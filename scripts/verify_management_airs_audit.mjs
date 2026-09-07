#!/usr/bin/env node
// Read only known synthetic traces. The supplied operator module exports an
// authenticated Prisma AIRS SDK client as `gw`; credentials stay in that module.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { pathToFileURL } from 'node:url';

async function main() {
  const { values } = parseArgs({ options: {
    'management-module': { type: 'string' },
    expectations: { type: 'string' },
    workspace: { type: 'string' },
    output: { type: 'string' },
  } });
  for (const name of ['management-module', 'expectations', 'workspace', 'output']) {
    assert(values[name], `--${name} is required`);
  }
  const file = path.resolve(values.expectations);
  assert(fs.statSync(file).size <= 8192, 'Expectations are too large');
  const expected = JSON.parse(fs.readFileSync(file, 'utf8'));
  assert(Array.isArray(expected) && expected.length >= 2 && expected.length <= 5);
  const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  assert(expected.every(e => uuid.test(e.subject) && uuid.test(e.trace_id)));
  assert(new Set(expected.map(e => e.subject)).size >= 2);
  assert(new Set(expected.map(e => e.trace_id)).size === expected.length);
  const { gw } = await import(pathToFileURL(path.resolve(values['management-module'])));
  const checks = [];
  for (const e of expected) {
    const result = await gw.telemetry.logs({
      workspaceSlug: values.workspace, traceId: e.trace_id, pageSize: 10,
    });
    const records = result.data.records;
    checks.push({
      subject: e.subject, trace_id: e.trace_id, records: records.length,
      passed: records.length === 1 && records.every(r =>
        r._user === e.subject && r.trace_id === e.trace_id &&
        r.is_success && r.response_status_code === 200),
      cost_recorded: records.length === 1 && records.every(r => typeof r.cost === 'number'),
      usage_recorded: records.length === 1 && records.every(r => typeof r.total_units === 'number'),
    });
  }
  const receipt = {
    checked_at: new Date().toISOString(),
    passed: checks.every(r => r.passed && r.cost_recorded && r.usage_recorded),
    scope: 'SCM persisted request telemetry: signed subject, trace, success, cost and usage fields',
    checks,
  };
  fs.writeFileSync(path.resolve(values.output), JSON.stringify(receipt, null, 2) + '\n', { mode: 0o600 });
  console.log(JSON.stringify({ passed: receipt.passed, checked_traces: checks.length }));
  if (!receipt.passed) process.exitCode = 1;
}

main().catch(error => {
  // SDK errors can carry request details; do not print arbitrary error messages.
  console.error(JSON.stringify({ passed: false, error_name: error.name }));
  process.exitCode = 1;
});
