import test from 'node:test';
import http from 'node:http';
import assert from 'node:assert/strict';
import { createServer, transform, RULE_VERSION } from './server.mjs';

const context = () => ({
  provider: 'openai', requestType: 'createModelResponse', eventType: 'beforeRequestHook',
  request: { json: { model: 'gpt-4.1', reasoning: { effort: 'low' }, input: 'reasoning.effort',
    tools: [{ parameters: { properties: { reasoning: { type: 'string' } } } }] } },
});

test('only the unsupported root field changes; original context is intact', () => {
  const input = context(); const original = structuredClone(input);
  const body = structuredClone(input.request.json); delete body.reasoning;
  assert.deepEqual(transform(input), {
    verdict: true, data: { rule_version: RULE_VERSION, removed_fields: ['reasoning'] },
    transformedData: { request: { json: body } },
  });
  assert.deepEqual(input, original);
});

test('other providers, models, APIs and absent reasoning pass through unchanged', () => {
  const variants = [context(), context(), context(), context(), context()];
  variants[0].provider = 'vertex-ai'; variants[1].request.json.model = 'gpt-5-mini';
  variants[2].requestType = 'chatComplete'; delete variants[3].request.json.reasoning;
  delete variants[4].request.json.model;
  for (const input of variants) assert.deepEqual(transform(input), {
    verdict: true, data: { rule_version: RULE_VERSION, removed_fields: [] },
  });
});

test('invalid contexts fail instead of modifying arbitrary JSON', () => {
  for (const input of [null, [], {}, { ...context(), eventType: 'afterRequestHook' },
    { ...context(), request: { json: [] } }]) assert.throws(() => transform(input));
});

test('HTTP authentication, parser, size limit and transformation', async (t) => {
  const secret = 'test-only-compatibility-secret-00000000';
  const server = createServer(secret, { maxBytes: 1024 });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));
  const url = `http://127.0.0.1:${server.address().port}/terminal-compatibility`;
  const send = (body, headers = {}) => fetch(url, { method: 'POST', body,
    headers: { 'Content-Type': 'application/json', 'x-airs-compatibility-key': secret, ...headers } });
  assert.equal((await send('{}', { 'x-airs-compatibility-key': 'wrong' })).status, 401);
  assert.equal((await send('{}', { 'Content-Type': 'text/plain' })).status, 415);
  assert.equal((await send('x'.repeat(1025))).status, 413);
  assert.equal((await send('{')).status, 400);
  assert.equal((await send('{}')).status, 400);
  const response = await send(JSON.stringify(context()));
  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), transform(context()));
});


test('an incomplete authenticated request consumes capacity until disconnected', async (t) => {
  const secret = 'test-only-compatibility-secret-00000000';
  const server = createServer(secret, { maxConcurrent: 1 });
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));
  const url = `http://127.0.0.1:${server.address().port}/terminal-compatibility`;
  const headers = { 'Content-Type': 'application/json', 'x-airs-compatibility-key': secret };
  const held = http.request(url, { method: 'POST', headers });
  held.on('error', () => {});
  t.after(() => held.destroy());
  const arrived = new Promise((resolve) => server.once('request', resolve));
  held.write('{');
  const incoming = await arrived;
  assert.equal((await fetch(url, { method: 'POST', headers, body: '{}' })).status, 503);
  const disconnected = new Promise((resolve) => incoming.once('close', resolve));
  held.destroy(); await disconnected;
  assert.equal((await fetch(url, { method: 'POST', headers, body: JSON.stringify(context()) })).status, 200);
});
