import assert from "node:assert/strict";
import { cpSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { platformPackage } from "./lib/launcher.js";

function fixture(t, native) {
  const root = mkdtempSync(path.join(os.tmpdir(), "airs-harness-launcher-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const module = path.join(root, "node_modules", "airs-harness");
  cpSync(path.dirname(fileURLToPath(import.meta.url)), module, { recursive: true });
  if (native !== undefined) {
    const target = path.join(root, "node_modules", platformPackage(process.platform, process.arch));
    mkdirSync(path.join(target, "bin"), { recursive: true });
    writeFileSync(path.join(target, "package.json"), JSON.stringify({ name: path.basename(target) }));
    if (native !== null) {
      writeFileSync(path.join(target, "bin", "airs-harness"), native, { mode: 0o755 });
    }
  }
  return path.join(module, "bin", "airs-harness.js");
}

test("unsupported platform is actionable", () => {
  assert.throws(() => platformPackage("freebsd", "x64"), /does not support/);
  assert.throws(() => platformPackage("darwin", "x64"), /Apple Silicon Macs only/);
});

test("missing platform dependency fails without fetching a replacement", (t) => {
  const result = spawnSync(process.execPath, [fixture(t)], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /optional dependencies enabled/);
});

test("incomplete native package reports a reinstall", (t) => {
  const result = spawnSync(process.execPath, [fixture(t, null)], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /native executable is missing/);
});

test("launcher preserves arguments, cwd, stdin and exit code", { skip: process.platform === "win32" }, (t) => {
  const entry = fixture(t, '#!/bin/sh\nprintf "%s\\n" "$PWD" "$1"\ncat\nexit 17\n');
  const argument = 'space and $(do-not-execute) ; literal';
  const result = spawnSync(process.execPath, [entry, argument], {
    encoding: "utf8", input: "stdin preserved\n", cwd: os.tmpdir(),
  });
  assert.equal(result.status, 17);
  assert.match(result.stdout, /stdin preserved/);
  assert.ok(result.stdout.includes(argument));
  assert.ok(result.stdout.includes(os.tmpdir()));
});

test("termination reaches the native process", { skip: process.platform === "win32", timeout: 5000 }, async (t) => {
  const entry = fixture(t, '#!/bin/sh\nprintf "ready\\n"\nexec sleep 30\n');
  const child = spawn(process.execPath, [entry], { stdio: ["ignore", "pipe", "pipe"] });
  t.after(() => { if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL"); });
  await once(child.stdout, "data");
  const closed = once(child, "close");
  child.kill("SIGTERM");
  assert.deepEqual(await closed, [null, "SIGTERM"]);
});


test("termination still forwards after the native process handles an interrupt", { skip: process.platform === "win32", timeout: 5000 }, async (t) => {
  const entry = fixture(t, `#!${process.execPath}
process.on("SIGINT", () => process.stdout.write("interrupted\\n"));
process.stdout.write("ready\\n");
setInterval(() => {}, 1000);
`);
  const child = spawn(process.execPath, [entry], {
    detached: true, stdio: ["ignore", "pipe", "pipe"],
  });
  t.after(() => {
    try { process.kill(-child.pid, "SIGKILL"); }
    catch (error) { if (error.code !== "ESRCH") throw error; }
  });
  assert.match(String((await once(child.stdout, "data"))[0]), /ready/);
  const interrupted = once(child.stdout, "data");
  child.kill("SIGINT");
  assert.match(String((await interrupted)[0]), /interrupted/);
  const closed = once(child, "close");
  child.kill("SIGTERM");
  assert.deepEqual(await closed, [null, "SIGTERM"]);
});
