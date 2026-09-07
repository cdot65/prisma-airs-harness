import http from 'node:http';
import { pathToFileURL } from 'node:url';

const hop = new Set(['connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization',
  'te', 'trailer', 'transfer-encoding', 'upgrade']);
const allowedPortkey = new Set(['x-portkey-api-key', 'x-portkey-trace-id']);
const forbiddenBody = ['provider', 'api_key', 'custom_host', 'config', 'override_params',
  'input_guardrails', 'output_guardrails'];

// Classification only: AIRS remains responsible for cryptographic authentication.
// Changing signed claims to avoid this filter makes the token invalid at AIRS.
export function terminalCredential(headers, policy) {
  const token = headers['x-portkey-api-key'] ?? headers.authorization?.replace(/^Bearer /, '');
  if (typeof token !== 'string') return null;
  const parts = token.split('.');
  if (parts.length !== 3) return null;
  try {
    const claims = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
    return claims && (policy.clients.includes(claims.azp) ||
      claims.portkey_workspace === policy.workspace) ? token : null;
  } catch { return null; }
}

function cleanHeaders(headers) {
  const excluded = new Set([...hop, ...(headers.connection ?? '').toLowerCase().split(',').map(s => s.trim())]);
  return Object.fromEntries(Object.entries(headers).filter(([key]) => !excluded.has(key)));
}

function failure(res, status, message) {
  res.writeHead(status, { 'content-type': 'application/json', 'cache-control': 'no-store', connection: 'close' });
  res.end(JSON.stringify({ error: { message, type: 'terminal_policy_error' } }));
}

function validateRequest(req, policy) {
  if (req.method !== 'POST' || !policy.paths.includes(req.url)) return [403, 'Operation is not authorized for Terminal'];
  if (Object.keys(req.headers).some(key =>
    (key.startsWith('x-portkey-') && !allowedPortkey.has(key)) || ['api-key', 'x-api-key'].includes(key))) {
    return [403, 'Caller-supplied gateway routing and policy headers are not permitted'];
  }
  if (req.headers['x-portkey-api-key'] && req.headers.authorization) return [400, 'Use one gateway credential header'];
  if (req.headers['content-encoding'] || req.headers['content-type']?.split(';')[0].trim() !== 'application/json') {
    return [415, 'Uncompressed application/json is required'];
  }
  return null;
}

async function body(req, limit) {
  let size = 0;
  const chunks = [];
  for await (const chunk of req) {
    size += chunk.length;
    if (size > limit) throw new Error('size');
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

export function createServer({ upstreamPort, policy, maxBytes = 16 * 1024 * 1024, maxActive = 8 }) {
  if (!Number.isInteger(upstreamPort) || upstreamPort < 1 || upstreamPort > 65535 ||
      !policy?.clients?.length || !policy?.workspace || !policy?.paths?.length) throw new Error('Invalid identity filter configuration');
  let active = 0;
  const agent = new http.Agent({ keepAlive: true, maxSockets: 128, maxFreeSockets: 16 });
  const server = http.createServer({ maxHeaderSize: 32768 }, async (req, res) => {
    if (!req.url.startsWith('/') || req.url.startsWith('//')) return failure(res, 400, 'Origin-form request target required');
    const names = req.rawHeaders.filter((_, index) => index % 2 === 0).map(name => name.toLowerCase());
    if (['authorization', 'x-portkey-api-key'].some(name => names.filter(n => n === name).length > 1)) {
      return failure(res, 400, 'Duplicate credential headers');
    }
    const credential = terminalCredential(req.headers, policy);
    let bytes;
    if (credential) {
      const invalid = validateRequest(req, policy);
      if (invalid) return failure(res, ...invalid);
      if (Number(req.headers['content-length'] ?? 0) > maxBytes) return failure(res, 413, 'Request body exceeds the Terminal limit');
      if (active >= maxActive) return failure(res, 503, 'Terminal capacity reached');
      active++;
      res.once('close', () => { active--; });
      try {
        bytes = await body(req, maxBytes);
        const parsed = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
        if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed) ||
            forbiddenBody.some(key => Object.hasOwn(parsed, key))) throw new Error('body');
        if (Object.hasOwn(parsed, 'model') && (typeof parsed.model !== 'string' ||
            !/^@[A-Za-z0-9][A-Za-z0-9._-]*\/[A-Za-z0-9][A-Za-z0-9._:/-]{0,255}$/.test(parsed.model))) {
          return failure(res, 400, 'Select the gateway default or an explicit @provider/model');
        }
      } catch (error) {
        if (!res.destroyed) failure(res, error.message === 'size' ? 413 : 400, 'Invalid Terminal request body');
        return;
      }
    }
    const headers = cleanHeaders(req.headers);
    if (credential) {
      // The helper emits Bearer; native mandatory JWT checks use x-portkey-api-key.
      // Forward exactly the original user token, never a shared service credential.
      headers['x-portkey-api-key'] = credential;
      delete headers.authorization;
      headers['content-length'] = String(bytes.length);
    }
    const upstream = http.request({ hostname: '127.0.0.1', port: upstreamPort, method: req.method,
      path: req.url, headers, agent }, response => {
      res.writeHead(response.statusCode, cleanHeaders(response.headers));
      response.on('error', () => res.destroy());
      response.pipe(res);
      res.once('close', () => response.destroy());
    });
    upstream.on('error', () => {
      if (!res.headersSent && !res.destroyed) failure(res, 502, 'Gateway unavailable');
      else res.destroy();
    });
    upstream.setTimeout(240_000, () => upstream.destroy());
    req.once('aborted', () => upstream.destroy());
    res.once('close', () => upstream.destroy());
    req.on('error', () => upstream.destroy());
    if (bytes) upstream.end(bytes);
    else req.pipe(upstream);
  });
  // Preserve existing non-Terminal websocket integrations on the shared listener.
  server.on('upgrade', (req, socket, head) => {
    if (terminalCredential(req.headers, policy) || !req.url.startsWith('/') || req.url.startsWith('//')) {
      socket.end('HTTP/1.1 403 Forbidden\r\nConnection: close\r\nContent-Length: 0\r\n\r\n');
      return;
    }
    const upstream = http.request({ hostname: '127.0.0.1', port: upstreamPort,
      path: req.url, headers: req.headers, method: req.method });
    upstream.once('upgrade', (response, remote, upstreamHead) => {
      socket.write(`HTTP/1.1 ${response.statusCode} Switching Protocols\r\n` +
        response.rawHeaders.reduce((text, value, index, all) => index % 2 ? text : `${text}${value}: ${all[index + 1]}\r\n`, '') + '\r\n');
      if (upstreamHead.length) socket.write(upstreamHead);
      if (head.length) remote.write(head);
      remote.pipe(socket); socket.pipe(remote);
      socket.once('close', () => remote.destroy());
      remote.on('error', () => socket.destroy());
      remote.once('close', () => socket.destroy());
    });
    upstream.on('response', response => { response.resume(); socket.end('HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n'); });
    upstream.on('error', () => socket.destroy());
    socket.on('error', () => upstream.destroy());
    socket.once('close', () => upstream.destroy());
    upstream.end();
  });
  server.headersTimeout = 15_000;
  server.requestTimeout = 30_000;
  server.keepAliveTimeout = 5_000;
  server.maxConnections = 256;
  server.on('close', () => agent.destroy());
  return server;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const policy = JSON.parse(process.env.AIRS_TERMINAL_POLICY);
  const servers = [[8887, 8787], [8888, 8788]].map(([port, upstreamPort]) => {
    const server = createServer({ upstreamPort, policy });
    server.listen(port, '0.0.0.0');
    return server;
  });
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
    for (const server of servers) server.close();
    setTimeout(() => process.exit(0), 250_000).unref();
  });
}
