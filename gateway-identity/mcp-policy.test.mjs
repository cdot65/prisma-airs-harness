import assert from 'node:assert/strict';
import http from 'node:http';
import { once } from 'node:events';
import { test } from 'node:test';
import { createServer } from './server.mjs';

const policy = { clients: ['airs-terminal-pilot'], workspace: 'ws-terminal', paths: ['/v1/responses'],
  mcp: { client: 'airs-terminal-mcp', audience: 'airs-terminal-security', issuer: 'https://issuer.example/realm',
    organisation: 'org', role: 'scanner-user', path: '/ws-terminal/scanner/mcp', tools: ['pan_inline_scan'] } };
const claims = { azp: policy.mcp.client, aud: policy.mcp.audience, iss: policy.mcp.issuer,
  portkey_oid: 'org', portkey_workspace: policy.workspace, airs_roles: ['scanner-user'], scope: 'openid portkey.mcp.invoke', sub: 'user-1' };
const jwt = c => `header.${Buffer.from(JSON.stringify(c)).toString('base64url')}.signature`;

async function fixture(t) {
  const seen = [];
  const backend = http.createServer(async (req, res) => {
    const chunks = []; for await (const chunk of req) chunks.push(chunk);
    seen.push({ token: req.headers['x-portkey-api-key'], path: req.url, body: Buffer.concat(chunks).toString() });
    res.writeHead(200, { 'content-type': 'application/json', 'mcp-session-id': 'native-session' });
    res.end('{"jsonrpc":"2.0","id":1,"result":{}}');
  });
  backend.listen(0, '127.0.0.1'); await once(backend, 'listening');
  const filter = createServer({ upstreamPort: backend.address().port, policy, mode: 'mcp' });
  filter.listen(0, '127.0.0.1'); await once(filter, 'listening');
  t.after(() => { filter.closeAllConnections(); backend.closeAllConnections(); filter.close(); backend.close(); });
  return { seen, url: `http://127.0.0.1:${filter.address().port}` };
}

test('MCP preserves the original resource JWT and exact allowed tool body', async t => {
  const f = await fixture(t);
  const body = '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"pan_inline_scan","arguments":{"scan_request":{"response":"hello"}}}}';
  const response = await fetch(f.url + policy.mcp.path, { method: 'POST', body,
    headers: { authorization: `Bearer ${jwt(claims)}`, 'content-type': 'application/json' } });
  assert.equal(response.status, 200); assert.equal(response.headers.get('mcp-session-id'), 'native-session');
  await response.text(); assert.deepEqual(f.seen, [{ token: jwt(claims), path: policy.mcp.path, body }]);
  for (const method of ['GET', 'DELETE']) {
    const r = await fetch(f.url + policy.mcp.path, { method, headers: { 'x-portkey-api-key': jwt(claims) } });
    assert.equal(r.status, 200); await r.text();
  }
});

test('MCP denies inference identities and missing or wrong authorization claims before forwarding', async t => {
  const f = await fixture(t);
  for (const altered of [{ azp: 'airs-terminal-pilot' }, { aud: 'airs-terminal-inference' },
    { aud: ['airs-terminal-security', 'airs-terminal-inference'] }, { iss: 'https://other.example' },
    { portkey_workspace: 'another-workspace' }, { portkey_oid: 'another-org' },
    { airs_roles: [] }, { airs_roles: 'scanner-user' }, { scope: 'portkey.completions.write' }]) {
    const response = await fetch(f.url + policy.mcp.path, { method: 'POST', body: '{"jsonrpc":"2.0","id":1,"method":"tools/list"}',
      headers: { 'x-portkey-api-key': jwt({ ...claims, ...altered }), 'content-type': 'application/json' } });
    assert.equal(response.status, 403); await response.text();
  }
  assert.equal(f.seen.length, 0);
});

test('MCP rejects other tools, resources, paths and routing overrides', async t => {
  const f = await fixture(t);
  for (const [path, body, extra] of [
    [policy.mcp.path, { method: 'tools/call', params: { name: 'pan_batch_scan' } }, {}],
    [policy.mcp.path, { method: 'resources/read' }, {}],
    ['/other/scanner/mcp', { method: 'tools/list' }, {}],
    [policy.mcp.path, { method: 'tools/list' }, { 'x-portkey-config': '{}' }],
  ]) {
    const response = await fetch(f.url + path, { method: 'POST', body: JSON.stringify({ jsonrpc: '2.0', id: 1, ...body }),
      headers: { 'x-portkey-api-key': jwt(claims), 'content-type': 'application/json', ...extra } });
    assert.equal(response.status, 403); await response.text();
  }
  assert.equal(f.seen.length, 0);
});
