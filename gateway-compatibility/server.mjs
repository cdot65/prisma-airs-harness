import http from 'node:http';
import { createHash, timingSafeEqual } from 'node:crypto';
import { pathToFileURL } from 'node:url';

export const RULE_VERSION = 'openai-gpt41-reasoning-v1';
const object = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);

export function transform(context) {
  if (!object(context) || context.eventType !== 'beforeRequestHook' ||
      typeof context.provider !== 'string' || typeof context.requestType !== 'string' ||
      !object(context.request) || !object(context.request.json)) {
    throw new Error('invalid hook context');
  }
  const body = context.request.json;
  const removed = context.provider === 'openai' && body.model === 'gpt-4.1' &&
    context.requestType === 'createModelResponse' && Object.hasOwn(body, 'reasoning');
  const result = {
    verdict: true,
    data: { rule_version: RULE_VERSION, removed_fields: removed ? ['reasoning'] : [] },
  };
  if (removed) {
    const replacement = { ...body };
    delete replacement.reasoning;
    result.transformedData = { request: { json: replacement } };
  }
  return result;
}

export function createServer(secret, { maxBytes = 16 * 1024 * 1024, maxConcurrent = 2 } = {}) {
  if (typeof secret !== 'string' || secret.length < 32 || !/^[\x21-\x7e]+$/.test(secret)) {
    throw new Error('a dedicated compatibility credential of at least 32 characters is required');
  }
  const hash = (value) => createHash('sha256').update(value).digest();
  const expected = hash(secret);
  let active = 0;
  const server = http.createServer({ maxHeaderSize: 8192 }, (req, res) => {
    const reply = (status, body) => {
      res.writeHead(status, { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' });
      res.end(JSON.stringify(body));
    };
    if (req.method === 'GET' && req.url === '/health') return reply(200, { status: 'ok', rule_version: RULE_VERSION });
    if (req.method !== 'POST' || req.url !== '/terminal-compatibility') return reply(404, { error: 'not found' });
    const credential = req.headers['x-airs-compatibility-key'];
    if (typeof credential !== 'string' || !timingSafeEqual(hash(credential), expected)) return reply(401, { error: 'unauthorized' });
    if (req.headers['content-type']?.split(';')[0].trim() !== 'application/json' || req.headers['content-encoding']) {
      return reply(415, { error: 'uncompressed application/json required' });
    }
    if (active >= maxConcurrent) return reply(503, { error: 'capacity reached' });
    if (Number(req.headers['content-length'] || 0) > maxBytes) return reply(413, { error: 'body too large' });
    active++;
    let released = false;
    const release = () => { if (!released) { released = true; active--; } };
    res.once('close', release);
    res.once('finish', release);
    let size = 0;
    let oversized = false;
    const chunks = [];
    req.on('data', (chunk) => {
      size += chunk.length;
      if (size > maxBytes) {
        if (!oversized) { oversized = true; chunks.length = 0; reply(413, { error: 'body too large' }); }
        return;
      }
      chunks.push(chunk);
    });
    req.on('end', () => {
      if (oversized) return;
      try { reply(200, transform(JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(Buffer.concat(chunks))))); }
      catch { reply(400, { error: 'invalid hook context' }); }
    });
    req.on('error', () => { release(); res.destroy(); });
  });
  server.maxConnections = 64;
  server.maxRequestsPerSocket = 100;
  server.requestTimeout = 10_000;
  server.headersTimeout = 10_000;
  server.keepAliveTimeout = 5_000;
  server.setTimeout(10_000, (socket) => socket.destroy());
  return server;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const server = createServer(process.env.AIRS_COMPATIBILITY_KEY);
  server.listen(8080, '0.0.0.0');
  for (const signal of ['SIGTERM', 'SIGINT']) {
    process.once(signal, () => {
      server.close(() => process.exit(0));
      setTimeout(() => process.exit(1), 12_000).unref();
    });
  }
}
