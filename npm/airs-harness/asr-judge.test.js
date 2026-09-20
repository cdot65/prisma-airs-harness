import assert from 'node:assert/strict';
import { cpSync, realpathSync, mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
const sample = fileURLToPath(new URL('../../codex-rs/skills/src/assets/samples/prisma-airs-asr-judge/scripts/asr_judge.mjs', import.meta.url));
for (const suffix of ['', '.cmd']) {
  test(`installed skill delegates to Node without shell or Python (${suffix || 'POSIX'})`, t => {
    const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'airs-node-judge-')));
    t.after(() => rmSync(root, { recursive: true, force: true }));
    const home = path.join(root, 'home with spaces');
    const script = path.join(home, 'skills/.system/prisma-airs-asr-judge/scripts/asr_judge.mjs');
    mkdirSync(path.dirname(script), { recursive: true });
    cpSync(sample, script); writeFileSync(path.join(home, 'config.toml'), '');
    const entry = path.join(root, 'airs-cli');
    writeFileSync(entry, 'console.log(JSON.stringify(process.argv.slice(2)));process.exitCode=7;');
    const args = ['scan ; $(literal).json', '--out', 'output with spaces', '--dry-run'];
    const result = spawnSync(process.execPath, [script, ...args], { encoding: 'utf8',
      env: { ...process.env, AIRS_MANAGED_CLI: entry + suffix } });
    assert.equal(result.status, 7, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), ['--harness-skill-home', home, 'redteam', 'judge', ...args]);
  });
}
