import assert from "node:assert/strict";
import { cpSync, existsSync, mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { managedEnvironment, platformPackage } from "./lib/launcher.js";
import { managedCliDirectory } from "./lib/prisma-cli.js";

function fixture(t, native, layout = "legacy") {
  const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), "airs-harness-launcher-")));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const module = path.join(root, "node_modules", "airs-harness");
  cpSync(path.dirname(fileURLToPath(import.meta.url)), module, { recursive: true, filter: (source) => path.basename(source) !== "node_modules" });
  const launcherManifestPath = path.join(module, "package.json");
  const manifest = JSON.parse(readFileSync(launcherManifestPath, "utf8"));
  const legacy = platformPackage(process.platform, process.arch);
  const scoped = `@cdot65/prisma-${legacy}`;
  const dependency = layout === "scoped" ? scoped : legacy;
  if (layout !== "legacy") manifest.name = "@cdot65/prisma-airs-harness";
  manifest.optionalDependencies = { [dependency]: layout === "legacy-alias" ? "https://npm.pkg.github.com/download/native" : manifest.version };
  writeFileSync(launcherManifestPath, JSON.stringify(manifest));
  const cli = path.join(root, "node_modules", "@cdot65", "prisma-airs-cli");
  mkdirSync(cli, { recursive: true });
  writeFileSync(path.join(cli, "package.json"), JSON.stringify({
    name: "@cdot65/prisma-airs-cli", version: "7.1.2", bin: { "airs-cli": "index.js" },
  }));
  writeFileSync(path.join(cli, "index.js"), "console.log('7.1.2')");
  if (native !== undefined) {
    const target = path.join(root, "node_modules", dependency);
    mkdirSync(path.join(target, "bin"), { recursive: true });
    writeFileSync(path.join(target, "package.json"), JSON.stringify({ name: layout === "legacy" ? legacy : scoped, version: manifest.version }));
    if (native !== null) {
      writeFileSync(path.join(target, "bin", process.platform === "win32" ? "airs-harness.exe" : "airs-harness"), native, { mode: 0o755 });
    }
  }
  return path.join(module, "bin", "airs-harness.js");
}

test("unsupported platform is actionable", () => {
  assert.throws(() => platformPackage("freebsd", "x64"), /does not support/);
  assert.throws(() => platformPackage("darwin", "x64"), /Apple Silicon Macs only/);
});

test("missing platform dependency fails without fetching a replacement", (t) => {
  const entry = fixture(t);
  const manifest = JSON.parse(readFileSync(path.join(path.dirname(entry), "../package.json"), "utf8"));
  const result = spawnSync(process.execPath, [entry], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.ok(result.stderr.includes(`airs-harness@${manifest.version}`));
  assert.match(result.stderr, /--include=optional/);
  assert.match(result.stderr, /same registry and installation scope/);
});

test("incomplete native package reports a reinstall", (t) => {
  const result = spawnSync(process.execPath, [fixture(t, null)], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /native executable is missing/);
});

test("Windows PATH variants cannot override the managed CLI directory", () => {
  const original = { Path: "stale-path", PATH: "selected-path", path: "another-path", KEEP: "value" };
  const env = managedEnvironment(original, "win32");
  assert.equal(env.PATH, `${managedCliDirectory};selected-path`);
  assert.equal(env.AIRS_MANAGED_CLI, path.join(managedCliDirectory, "airs-cli.cmd"));
  assert.equal(env.KEEP, "value");
  assert.deepEqual(Object.keys(env).filter((key) => key.toUpperCase() === "PATH"), ["PATH"]);
  assert.equal(original.Path, "stale-path");
  const mixedCase = managedEnvironment({ Path: "windows-path" }, "win32");
  assert.equal(mixedCase.PATH, `${managedCliDirectory};windows-path`);
});

test("native launch requires the pinned dependency before executing anything", (t) => {
  for (const version of [null, "3.3.0"]) {
    const entry = fixture(t, '#!/bin/sh\nprintf "native must not run\\n"\n');
    const root = path.resolve(path.dirname(entry), "../../..");
    const cli = path.join(root, "node_modules", "@cdot65", "prisma-airs-cli");
    if (version === null) rmSync(cli, { recursive: true });
    else writeFileSync(path.join(cli, "package.json"), JSON.stringify({ version, bin: { "airs-cli": "index.js" } }));
    const result = spawnSync(process.execPath, [entry, "--version"], { encoding: "utf8" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /missing|version mismatch/);
    assert.equal(result.stdout, "");
  }
});

test("native login-shell tools use the pinned CLI even after PATH is replaced", { skip: process.platform === "win32" }, (t) => {
  const entry = fixture(t, `#!${process.execPath}
const {spawnSync}=require("node:child_process");
const result=spawnSync("/bin/sh", ["-lc", 'PATH="$AIRS_TEST_RESET_PATH"; export PATH; exec "$AIRS_MANAGED_CLI" "$@"', "fixture", ...process.argv.slice(2)], {encoding:"utf8"});
if(result.error) throw result.error;
process.stdout.write(JSON.stringify({managed:process.env.AIRS_MANAGED_CLI,nativePath:process.env.PATH,decoyPath:process.env.Path,nested:JSON.parse(result.stdout)}));
process.stderr.write(result.stderr);
process.exitCode=result.status;
`);
  const root = path.resolve(path.dirname(entry), "../../..");
  const cliEntry = path.join(root, "node_modules", "@cdot65", "prisma-airs-cli", "index.js");
  writeFileSync(cliEntry, "console.log(JSON.stringify({args:process.argv.slice(2),cwd:process.cwd(),entry:__filename}));process.exitCode=7;\n");
  const resetPath = path.join(root, "global-bin");
  mkdirSync(resetPath);
  symlinkSync(process.execPath, path.join(resetPath, "node"));
  writeFileSync(path.join(resetPath, "airs"), '#!/bin/sh\nprintf "unmanaged CLI must not run\\n"\nexit 99\n', { mode: 0o755 });
  const literal = "spaces ; $(literal)";
  const inherited = Object.fromEntries(Object.entries(process.env).filter(([key]) => key.toUpperCase() !== "PATH"));
  const result = spawnSync(process.execPath, [entry, "doctor", literal], {
    cwd: root, encoding: "utf8", env: {
      ...inherited, Path: "unix-decoy", PATH: process.env.PATH,
      AIRS_TEST_RESET_PATH: resetPath, AIRS_MANAGED_CLI: path.join(resetPath, "airs"),
    },
  });
  assert.equal(result.status, 7, result.stderr);
  const output = JSON.parse(result.stdout);
  const managed = path.resolve(path.dirname(entry), "../managed-cli");
  assert.equal(output.managed, path.join(managed, "airs-cli"));
  assert.equal(output.nativePath, `${managed}:${process.env.PATH}`);
  assert.equal(output.decoyPath, "unix-decoy");
  assert.deepEqual(output.nested, { args: ["doctor", literal], cwd: root, entry: cliEntry });
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
  // Announce readiness from the final process, without a shell-to-sleep exec race.
  const entry = fixture(t, `#!${process.execPath}
process.stdout.write("ready\\n");
setInterval(() => {}, 1000);
`);
  const child = spawn(process.execPath, [entry], { detached: true, stdio: ["ignore", "pipe", "pipe"] });
  t.after(() => {
    try { process.kill(-child.pid, "SIGKILL"); }
    catch (error) { if (error.code !== "ESRCH") throw error; }
  });
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


test("declared keys select scoped natives without inferring from the launcher name", () => {
  const version = "0.1.0-alpha.9";
  for (const [platform, arch] of [["linux", "x64"], ["linux", "arm64"], ["darwin", "arm64"], ["win32", "x64"]]) {
    const legacy = platformPackage(platform, arch);
    const scoped = `@cdot65/prisma-${legacy}`;
    assert.equal(platformPackage(platform, arch, { name: "airs-harness", version,
      optionalDependencies: { [scoped]: version } }), scoped);
    assert.equal(platformPackage(platform, arch, { name: "@cdot65/prisma-airs-harness", version,
      optionalDependencies: { [legacy]: "https://npm.pkg.github.com/download/native" } }), legacy);
    for (const dependencies of [{}, { [scoped]: false }, { [scoped]: "another-version" },
      { [scoped]: version, [legacy]: version }, { [`@other/prisma-${legacy}`]: version }]) {
      assert.throws(() => platformPackage(platform, arch, { version, optionalDependencies: dependencies }),
        /does not include a native package|Multiple native packages|Invalid native dependency/);
    }
  }
  assert.throws(() => platformPackage("darwin", "x64", { optionalDependencies: {
    "@cdot65/prisma-airs-harness-darwin-x64": version,
  }, version }), /Apple Silicon Macs only/);
});

test("scoped and historical alias layouts launch the declared native", { skip: process.platform === "win32" }, (t) => {
  for (const layout of ["scoped", "legacy-alias"]) {
    const entry = fixture(t, '#!/bin/sh\nprintf "declared native\\n"\n', layout);
    const result = spawnSync(process.execPath, [entry], { encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout.trim(), "declared native");
  }
});

test("native identity or version mismatch fails before execution", (t) => {
  for (const mutation of [{ name: "@other/native" }, { version: "0.0.0" }]) {
    const entry = fixture(t, '#!/bin/sh\nprintf "must not execute\\n"\n', "scoped");
    const root = path.resolve(path.dirname(entry), "../../..");
    const file = path.join(root, "node_modules", `@cdot65/prisma-${platformPackage(process.platform, process.arch)}`, "package.json");
    writeFileSync(file, JSON.stringify({ ...JSON.parse(readFileSync(file, "utf8")), ...mutation }));
    const result = spawnSync(process.execPath, [entry], { encoding: "utf8" });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /identity\/version mismatch/);
    assert.equal(result.stdout, "");
  }
});


test("a release without the host architecture reports unsupported packaging", () => {
  const manifest = {version: "0.1.0-alpha.17", optionalDependencies: {
    "airs-harness-linux-x64": "0.1.0-alpha.17",
    "airs-harness-darwin-arm64": "0.1.0-alpha.17",
  }};
  assert.throws(() => platformPackage("linux", "arm64", manifest), {
    message: "This airs-harness release does not include a native package for linux/arm64. Install a release that supports this platform.",
  });
});

// Simulate only process.versions.node in the launcher subprocess. This exercises
// the real entrypoints without a production override or claiming a Node 18 run.
function runtimeFixture(t, version) {
  const entry = fixture(t, "#!/bin/sh\nexit 99\n");
  const module = path.resolve(path.dirname(entry), "..");
  const root = path.resolve(module, "../..");
  const marker = path.join(root, "child-started");
  const child = `#!${process.execPath}\nrequire("node:fs").writeFileSync(${JSON.stringify(marker)}, "started");\nconsole.log(JSON.stringify(process.argv.slice(2)));\nprocess.exitCode=17;\n`;
  const native = path.join(root, "node_modules", platformPackage(process.platform, process.arch), "bin", "airs-harness");
  writeFileSync(native, child, { mode: 0o755 });
  writeFileSync(path.join(root, "node_modules", "@cdot65", "prisma-airs-cli", "index.js"), child);
  const preload = path.join(root, "runtime-fixture.cjs");
  writeFileSync(preload, `Object.defineProperty(process.versions, "node", {value:${JSON.stringify(version)}});\n`);
  return { entry, module, marker, preload };
}

const runtimeEntrypoints = [
  ["native", "bin/airs.js", ["--version"]],
  ["managed CLI", "bin/airs.js", ["cli", "doctor"]],
  ["completion", "bin/airs.js", ["completion", "bash"]],
  ["direct managed wrapper", "managed-cli/airs-cli", ["doctor"]],
  ["legacy launcher", "bin/airs-harness.js", ["--version"]],
];

for (const [name, relative, args] of runtimeEntrypoints) {
  test(`unsupported Node runtime stops ${name} before any child starts`, { skip: process.platform === "win32" }, (t) => {
    for (const version of ["18.19.1", "22.12.0", "23.4.0", "22.13.0-rc.1", "24.0.0-pre", "22.13", "garbage", "022.13.0", "22.13.0\n"]) {
      const { module, marker, preload } = runtimeFixture(t, version);
      const result = spawnSync(process.execPath, ["--require", preload, path.join(module, relative), ...args], { encoding: "utf8" });
      assert.equal(result.status, 1, `${version}: ${result.stderr}`);
      assert.equal(result.stdout, "", version);
      assert.match(result.stderr, /requires Node\.js.*\^22\.13\.0 \|\| >=23\.5\.0/);
      assert.match(result.stderr, /node --version/);
      assert.equal(existsSync(marker), false, `${name} spawned a child with ${version}`);
    }
  });
}


for (const [name, relative, prefix] of [
  ["native", "bin/airs.js", []],
  ["managed CLI", "bin/airs.js", ["cli"]],
  ["direct managed wrapper", "managed-cli/airs-cli", []],
]) {
  test(`supported Node runtime preserves ${name} argument forwarding and exit status`, { skip: process.platform === "win32" }, (t) => {
    for (const version of ["22.13.0", "22.13.1", "22.99.0", "23.5.0", "23.6.1", "24.0.0", "25.1.0", "22.13.0+vendor.1"]) {
      const { module, marker, preload } = runtimeFixture(t, version);
      const args = ["doctor", "--", "literal ; $(no-shell)"];
      const result = spawnSync(process.execPath, ["--require", preload, path.join(module, relative), ...prefix, ...args], { encoding: "utf8" });
      assert.equal(result.status, 17, `${version}: ${result.stderr}`);
      assert.equal(result.stderr, "");
      assert.equal(readFileSync(marker, "utf8"), "started");
      assert.deepEqual(JSON.parse(result.stdout), args);
    }
  });
}

test("runtime policy follows the package engine declaration and rejects an unknown range", { skip: process.platform === "win32" }, (t) => {
  for (const [range, version, allowed] of [
    ["^24.2.0 || >=25.1.0", "22.13.0", false],
    ["^24.2.0 || >=25.1.0", "24.1.9", false],
    ["^24.2.0 || >=25.1.0", "24.2.0", true],
    ["^24.2.0 || >=25.1.0", "25.0.9", false],
    ["^24.2.0 || >=25.1.0", "25.1.0", true],
    [">=18.0.0", "24.0.0", false],
  ]) {
    const { module, marker, preload } = runtimeFixture(t, version);
    const manifestPath = path.join(module, "package.json");
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    manifest.engines.node = range;
    writeFileSync(manifestPath, JSON.stringify(manifest));
    const result = spawnSync(process.execPath, ["--require", preload, path.join(module, "bin/airs.js"), "--version"], { encoding: "utf8" });
    assert.equal(result.status, allowed ? 17 : 1, `${range}, ${version}: ${result.stderr}`);
    assert.equal(existsSync(marker), allowed);
    if (!allowed) assert.match(result.stderr, /requires Node\.js|engine declaration is invalid/);
  }
});
