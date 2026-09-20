#!/usr/bin/env node
// The managed TypeScript CLI owns ingestion, provider calls, replay and metrics.
import { spawn } from 'node:child_process';
import { existsSync, realpathSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

try {
  const configured = process.env.AIRS_MANAGED_CLI;
  if (!configured) throw new Error('The bundled AIRS CLI is unavailable. Run this skill inside airs; no judgment was performed.');
  // Invoke the Node entrypoint directly on every OS, never a .cmd shell wrapper.
  const entry = configured.endsWith('.cmd') ? configured.slice(0, -4) : configured;
  if (!path.isAbsolute(entry) || !existsSync(entry)) throw new Error('The bundled AIRS CLI is missing; reinstall airs-harness.');
  const script = realpathSync(fileURLToPath(import.meta.url));
  const home = path.resolve(path.dirname(script), '../../../..');
  const expected = path.join(home, 'skills', '.system', 'prisma-airs-asr-judge', 'scripts', 'asr_judge.mjs');
  if (script !== expected || !existsSync(path.join(home, 'config.toml'))) {
    throw new Error('Run the installed judge skill from its AIRS environment; its owning environment could not be resolved.');
  }
  const child = spawn(process.execPath, [entry, '--harness-skill-home', home,
    'redteam', 'judge', ...process.argv.slice(2)], { stdio: 'inherit', env: process.env });
  const handlers = new Map();
  for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
    const handler = () => { if (child.exitCode === null && child.signalCode === null) child.kill(signal); };
    handlers.set(signal, handler);
    process.on(signal, handler);
  }
  const cleanup = () => { for (const [signal, handler] of handlers) process.removeListener(signal, handler); };
  child.on('error', () => { cleanup(); console.error('Cannot start the bundled judge; no judgment was performed.'); process.exitCode = 1; });
  child.on('exit', (code, signal) => { cleanup(); if (signal) process.kill(process.pid, signal); else process.exitCode = code ?? 1; });
} catch (error) {
  console.error(error.message);
  process.exitCode = 1;
}
