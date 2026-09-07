import assert from 'node:assert/strict';
import http from 'node:http';
import { once } from 'node:events';
import { test } from 'node:test';
import { createServer } from './server.mjs';

const policy = { clients: ['airs-terminal-pilot'], workspace: 'ws-prisma-ff3d74', paths: ['/v1/responses', '/v1/chat/completions'] };
const token = `header.${Buffer.from(JSON.stringify({ azp: 'airs-terminal-pilot', portkey_workspace: policy.workspace })).toString('base64url')}.signature`;
async function fixture(t, respond) {
  const seen = [];
  const backend = http.createServer(async (req, res) => {
    const chunks = [];
    for await (const chunk of req) chunks.push(chunk);
    seen.push({ headers: req.headers, body: Buffer.concat(chunks).toString(), path: req.url });
    if (respond) return respond(req, res);
    res.writeHead(200, { 'content-type': 'application/json' }); res.end('{"ok":true}');
  });
  backend.listen(0, '127.0.0.1'); await once(backend, 'listening');
  const filter = createServer({ upstreamPort: backend.address().port, policy, maxBytes: 1024 });
  filter.listen(0, '127.0.0.1'); await once(filter, 'listening');
  t.after(() => { filter.closeAllConnections(); backend.closeAllConnections(); filter.close(); backend.close(); });
  return { seen, backend, filter, url: `http://127.0.0.1:${filter.address().port}` };
}
function headers(extra = {}) { return { 'content-type': 'application/json', 'x-portkey-api-key': token, ...extra }; }

test('preserves exact default/explicit request bytes and original user JWT', async t => {
  const f = await fixture(t);
  for (const body of ['{ "input": "hello", "stream":true }', '{"input":"hello","model":"@openai/gpt-4.1"}']) {
    const response = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers(), body });
    assert.equal(response.status, 200); await response.text();
    assert.equal(f.seen.at(-1).body, body);
    assert.equal(f.seen.at(-1).headers['x-portkey-api-key'], token);
  }
});
test('Bearer helper transport preserves JWT for native mandatory checks', async t => {
  const f = await fixture(t);
  const response = await fetch(f.url + '/v1/responses', { method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' }, body: '{}' });
  assert.equal(response.status, 200); await response.text();
  assert.equal(f.seen[0].headers['x-portkey-api-key'], token);
  assert.equal(f.seen[0].headers.authorization, undefined);
});
test('rejects routing, provider, metadata and policy override headers before backend', async t => {
  const f = await fixture(t);
  for (const header of ['x-portkey-config', 'x-portkey-config-version', 'x-portkey-provider',
    'x-portkey-custom-host', 'x-portkey-forward-headers', 'x-portkey-metadata',
    'x-portkey-input-guardrails', 'x-portkey-nitro-mode', 'x-api-key', 'api-key']) {
    const response = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers({ [header]: 'override' }), body: '{}' });
    assert.equal(response.status, 403, header); await response.text();
  }
  assert.equal(f.seen.length, 0);
});
test('rejects invalid model shapes, routing bodies, paths and ambiguous credentials', async t => {
  const f = await fixture(t);
  for (const body of [{ model: null }, { model: '' }, { model: 'ai-gateway' }, { model: 'gpt-4.1' },
    { model: 1 }, { provider: '@openai' }, { config: {} }, { input_guardrails: [] }, []]) {
    const r = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers(), body: JSON.stringify(body) });
    assert.equal(r.status, 400); await r.text();
  }
  for (const path of ['/v1/responses?config=evil', '/v1/proxy', '/ws-prisma-ff3d74/server/mcp']) {
    const r = await fetch(f.url + path, { method: 'POST', headers: headers(), body: '{}' });
    assert.equal(r.status, 403); await r.text();
  }
  const r = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers({ authorization: 'provider-key' }), body: '{}' });
  assert.equal(r.status, 400); await r.text();
  assert.equal(f.seen.length, 0);
});
test('bounds request memory and rejects compressed payloads', async t => {
  const f = await fixture(t);
  for (const [extra, body, status] of [[{}, JSON.stringify({ input: 'x'.repeat(1025) }), 413],
    [{ 'content-encoding': 'gzip' }, '{}', 415]]) {
    const r = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers(extra), body });
    assert.equal(r.status, status); await r.text();
  }
  assert.equal(f.seen.length, 0);
});
test('leaves existing opaque credentials and other stacks to native gateway enforcement', async t => {
  const f = await fixture(t);
  for (const credential of ['workspace-key', 'invalid-jwt', `header.${Buffer.from('{"azp":"another-stack"}').toString('base64url')}.signature`]) {
    const r = await fetch(f.url + '/v1/chat/completions', { method: 'POST', headers: headers({ 'x-portkey-api-key': credential, 'x-portkey-config': '{}' }), body: '{"model":"existing-model"}' });
    assert.equal(r.status, 200); await r.text();
    assert.equal(f.seen.at(-1).headers['x-portkey-config'], '{}');
  }
});
test('streams SSE promptly and cancels upstream when the caller disconnects', async t => {
  let closed;
  const close = new Promise(resolve => { closed = resolve; });
  const f = await fixture(t, (_req, res) => {
    res.writeHead(200, { 'content-type': 'text/event-stream' });
    res.write('data: first\n\n');
    res.once('close', closed);
  });
  const abort = new AbortController();
  const response = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers(), body: '{}', signal: abort.signal });
  const reader = response.body.getReader();
  assert.equal(new TextDecoder().decode((await reader.read()).value), 'data: first\n\n');
  abort.abort();
  await Promise.race([close, new Promise((_, reject) => { const timer = setTimeout(() => reject(new Error('upstream was not canceled')), 1000); timer.unref(); })]);
});
test('duplicate credentials are rejected and nested model/tool data is preserved', async t => {
  const f = await fixture(t);
  const response = await new Promise(resolve => {
    const r = http.request(f.url + '/v1/responses', { method: 'POST', headers: [
      'x-portkey-api-key', token, 'x-portkey-api-key', 'another-token', 'content-type', 'application/json',
    ] }, res => { res.resume(); resolve(res.statusCode); });
    r.end('{}');
  });
  assert.equal(response, 400);
  assert.equal(f.seen.length, 0);
  const raw = '{"input":[{"type":"function_call_output","output":"model=gpt-4.1"}],"tools":[{"type":"function","parameters":{"properties":{"model":{"type":"string"},"provider":{"type":"string"}}}}]}';
  const r = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers(), body: raw });
  assert.equal(r.status, 200); await r.text();
  assert.equal(f.seen[0].body, raw);
});
test('large signed-claim payloads cannot skip the Terminal policy', async t => {
  const f = await fixture(t);
  const large = `header.${Buffer.from(JSON.stringify({ azp: policy.clients[0], extra: 'x'.repeat(14000) })).toString('base64url')}.signature`;
  const r = await fetch(f.url + '/v1/responses', { method: 'POST', headers: headers({ 'x-portkey-api-key': large, 'x-portkey-config': '{}' }), body: '{}' });
  assert.equal(r.status, 403); await r.text();
  assert.equal(f.seen.length, 0);
});
test('shared-stack websocket upgrades retain bidirectional traffic', async t => {
  const f = await fixture(t);
  f.backend.on('upgrade', (_req, socket, head) => {
    socket.write('HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\n\r\n');
    if (head.length) socket.write(head);
    socket.pipe(socket);
  });
  await new Promise((resolve, reject) => {
    const request = http.request(f.url + '/v1/realtime', { headers: {
      connection: 'Upgrade', upgrade: 'websocket', 'x-portkey-api-key': 'existing-workspace-key',
    } });
    request.on('error', reject);
    request.on('upgrade', (_response, socket) => {
      socket.once('data', bytes => { assert.equal(bytes.toString(), 'ping'); socket.destroy(); resolve(); });
      socket.write('ping');
    });
    request.end();
  });
});
