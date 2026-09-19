import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";

test("runtime version and manifest boundaries are portable without a native child", (t) => {
  const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), "airs-runtime-policy-")));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  mkdirSync(path.join(root, "lib"));
  const guard = path.join(root, "lib", "runtime.js");
  cpSync(fileURLToPath(new URL("./lib/runtime.js", import.meta.url)), guard);
  const declared = "^22.13.0 || >=23.5.0";
  const cases = [
    ...["18.19.1", "22.12.0", "23.4.0", "22.13.0-rc.1", "24.0.0-pre", "22.13", "garbage", "022.13.0", "22.13.0\n"].map((version) => [declared, version, false]),
    ...["22.13.0", "22.13.1", "22.99.0", "23.5.0", "23.6.1", "24.0.0", "25.1.0", "22.13.0+vendor.1"].map((version) => [declared, version, true]),
    ["^24.2.0 || >=25.1.0", "24.1.9", false],
    ["^24.2.0 || >=25.1.0", "24.2.0", true],
    ["^24.2.0 || >=25.1.0", "25.0.9", false],
    ["^24.2.0 || >=25.1.0", "25.1.0", true],
    [">=18.0.0", "24.0.0", false],
    [undefined, "24.0.0", false],
  ];
  for (const [range, version, expected] of cases) {
    writeFileSync(path.join(root, "package.json"), JSON.stringify({
      type: "module", engines: { node: range },
    }));
    // Metadata override exists only in this subprocess fixture; the production
    // guard has no runtime override and no native executable is needed here.
    const script = `
Object.defineProperty(process.versions, "node", {value:${JSON.stringify(version)}});
const {checkNodeRuntime} = await import(${JSON.stringify(pathToFileURL(guard).href)});
process.stdout.write(JSON.stringify({supported:checkNodeRuntime()}));
`;
    const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], { encoding: "utf8" });
    assert.equal(result.error, undefined);
    assert.equal(result.status, expected ? 0 : 1, `${range}, ${version}: ${result.stderr}`);
    assert.deepEqual(JSON.parse(result.stdout), { supported: expected });
    if (expected) assert.equal(result.stderr, "");
    else assert.match(result.stderr, /Prisma AIRS Harness/);
  }
});
