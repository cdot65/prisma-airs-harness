import assert from "node:assert/strict";
import { cpSync, mkdtempSync, mkdirSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

function fixture(t, version = "7.1.2") {
  const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), "airs-cli-integration-")));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const harness = path.join(root, "node_modules", "airs-harness");
  cpSync(path.dirname(fileURLToPath(import.meta.url)), harness, { recursive: true, filter: (source) => path.basename(source) !== "node_modules" });
  if (version !== null) {
    const cli = path.join(root, "node_modules", "@cdot65", "prisma-airs-cli");
    mkdirSync(cli, { recursive: true });
    writeFileSync(path.join(cli, "package.json"), JSON.stringify({
      version, bin: { "airs-cli": "index.js" },
    }));
    writeFileSync(path.join(cli, "index.js"), `
console.log(JSON.stringify({args:process.argv.slice(2),cwd:process.cwd(),
 dotenv:Object.fromEntries(Object.entries(process.env).filter(([k])=>k.startsWith('DOTENV_CONFIG_'))),
 tenant:process.env.PANW_MGMT_TSG_ID}));
process.exitCode=7;
`);
  }
  return { root, entry: path.join(harness, "bin", "airs-harness.js") };
}

test("managed CLI preserves arguments and exit status while excluding project dotenv loading", (t) => {
  const { root, entry } = fixture(t);
  const literal = "spaces ; $(literal)";
  const result = spawnSync(process.execPath, [entry, "airs", "doctor", literal], {
    cwd: root, encoding: "utf8", env: { ...process.env,
      DOTENV_CONFIG_PATH: path.join(root, ".env"), DOTENV_CONFIG_OVERRIDE: "true",
      PANW_MGMT_TSG_ID: "fixture-tenant",
    },
  });
  assert.equal(result.status, 7, result.stderr);
  const output = JSON.parse(result.stdout);
  assert.deepEqual(output.args, ["doctor", literal]);
  assert.equal(output.cwd, root);
  assert.equal(output.tenant, "fixture-tenant");
  assert.ok(output.dotenv.DOTENV_CONFIG_PATH.endsWith(path.join("managed-cli", "empty.env")));
  assert.equal(output.dotenv.DOTENV_CONFIG_OVERRIDE, undefined);
});

test("missing or mismatched managed CLI fails instead of selecting a global airs", (t) => {
  for (const version of [null, "3.3.0"]) {
    const { entry } = fixture(t, version);
    const result = spawnSync(process.execPath, [entry, "airs", "--version"], { encoding: "utf8" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /missing|version mismatch/);
  }
});

test("dotenv argv cannot redirect managed credential discovery", (t) => {
  const { entry } = fixture(t);
  const result = spawnSync(process.execPath, [entry, "airs", "dotenv_config_path=untrusted.env"], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /dotenv argument overrides/);
});


test("public airs cli dispatches without a native package, login or environment", (t) => {
  const { entry } = fixture(t);
  const primary = path.join(path.dirname(entry), "airs.js");
  const result = spawnSync(process.execPath, [primary, "cli", "--", "literal --environment", "{\"x\":1}"], { encoding: "utf8" });
  assert.equal(result.status, 7, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout).args, ["--", "literal --environment", "{\"x\":1}"]);
});

test("only the compatibility launcher accepts the old airs forwarding action", (t) => {
  const { entry } = fixture(t);
  const legacy = spawnSync(process.execPath, [entry, "airs", "--version"], { encoding: "utf8" });
  assert.equal(legacy.status, 7, legacy.stderr);
  const primary = spawnSync(process.execPath, [path.join(path.dirname(entry), "airs.js"), "airs", "--version"], { encoding: "utf8" });
  assert.equal(primary.status, 1);
  assert.match(primary.stderr, /native package|does not include/);
});
