import assert from 'node:assert/strict';
import { once } from 'node:events';
import { test } from 'node:test';
import type { AddressInfo } from 'node:net';
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import { createApp, type ScanInput, type ScanVerdict } from './server.js';

const key = 'test-only-gateway-key-with-at-least-32-characters';
const profile = 'Prisma AIRS Terminal';

async function fixture(scan: (input: ScanInput) => Promise<ScanVerdict>) {
  const server = createApp({ gatewayKey: key, profile, scan }).listen(0, '127.0.0.1');
  await once(server, 'listening');
  const url = new URL(`http://127.0.0.1:${(server.address() as AddressInfo).port}/terminal-scanner/mcp`);
  const clients: Client[] = [];
  return {
    url,
    async connect() {
      const client = new Client({ name: 'acceptance', version: '1' });
      clients.push(client);
      await client.connect(new StreamableHTTPClientTransport(url, { requestInit: { headers: { 'x-airs-terminal-mcp-key': key } } }));
      return client;
    },
    async close() {
      await Promise.all(clients.map(client => client.close()));
      server.closeAllConnections();
      await new Promise<void>(resolve => server.close(() => resolve()));
    },
  };
}

test('actual SDK discovery and scan are stateless, bounded and profile-pinned', async () => {
  const calls: ScanInput[] = [];
  const f = await fixture(async input => { calls.push(input); return { action: 'allow', scan_id: 'scan-123', category: 'benign' }; });
  try {
    const client = await f.connect();
    assert.deepEqual(await client.listResources(), { resources: [] });
    assert.deepEqual(await client.listResourceTemplates(), { resourceTemplates: [] });
    assert.equal(calls.length, 0);
    const catalog = await client.listTools();
    assert.deepEqual(catalog.tools.map(tool => tool.name), ['pan_inline_scan']);
    const result = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: 'hello', profile, app_name: 'test' } } });
    assert.deepEqual(result.structuredContent, { results: { action: 'allow', scan_id: 'scan-123', profile_name: profile, category: 'benign' } });
    assert.equal(calls.length, 1);
    assert.equal(calls[0].prompt, 'hello');
    assert.equal(calls[0].appName, 'test');
    const denied = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: 'hello', profile: 'weaker-policy' } } });
    assert.equal(denied.isError, true);
    assert.equal(calls.length, 1);
    const oversized = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: 'a'.repeat(200_001) } } });
    assert.equal(oversized.isError, true);
    assert.equal(calls.length, 1);
    const unknownField = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: 'hello', api_key: 'attempted-override' } } });
    assert.equal(unknownField.isError, true);
    assert.equal(calls.length, 1);
  } finally { await f.close(); }
});

test('unauthorized HTTP requests do not invoke the scanner or disclose keys', async () => {
  let calls = 0;
  const f = await fixture(async () => { calls++; throw new Error('not reached'); });
  try {
    for (const supplied of ['', 'wrong', key + 'x']) {
      const response = await fetch(f.url, { method: 'POST', headers: { 'Content-Type': 'application/json', 'x-airs-terminal-mcp-key': supplied }, body: '{}' });
      assert.equal(response.status, 401);
      assert.deepEqual(await response.json(), { error: 'Unauthorized' });
    }
    assert.equal(calls, 0);
  } finally { await f.close(); }
});

test('upstream errors, timeouts and incomplete verdicts fail without hidden retries', async () => {
  for (const outcome of ['exception', 'timeout', 'error', 'missing-id', 'unknown-action']) {
    let calls = 0;
    const f = await fixture(async () => {
      calls++;
      if (outcome === 'exception') throw new Error('sensitive-upstream-request');
      return { action: outcome === 'unknown-action' ? 'unknown' : 'allow', scan_id: outcome === 'missing-id' ? '' : 'scan', timeout: outcome === 'timeout', error: outcome === 'error' };
    });
    try {
      const client = await f.connect();
      const result = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: 'hello' } } });
      assert.equal(result.isError, true);
      assert.equal(calls, 1);
      assert.equal(JSON.stringify(result).includes('sensitive-upstream-request'), false);
      assert.equal(result.structuredContent, undefined);
    } finally { await f.close(); }
  }
});

test('independent concurrent clients preserve their own results', async () => {
  const f = await fixture(async input => ({ action: 'block', scan_id: input.prompt! }));
  try {
    await Promise.all(Array.from({ length: 12 }, async (_, index) => {
      const client = await f.connect();
      const result = await client.callTool({ name: 'pan_inline_scan', arguments: { scan_request: { prompt: `scan-${index}` } } });
      assert.deepEqual(result.structuredContent, { results: { action: 'block', scan_id: `scan-${index}`, profile_name: profile } });
    }));
  } finally { await f.close(); }
});
