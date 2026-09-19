import { runChild } from "./child.js";
import { runCompletion } from "./completion.js";
import { inspectCommands } from "./migration-check.js";
import { managedCliDirectory, resolvePrismaCli, runPrismaCli } from "./prisma-cli.js";
import { checkNodeRuntime } from "./runtime.js";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

export function platformPackage(platform, arch, manifest) {
  if (platform === "darwin" && arch === "x64") {
    throw new Error("Prisma AIRS Harness supports Apple Silicon Macs only. " +
      "On Apple Silicon, use an arm64 Node.js installation outside Rosetta.");
  }
  if (!["linux", "darwin", "win32"].includes(platform) ||
      !["arm64", "x64"].includes(arch)) {
    throw new Error(`Prisma AIRS Harness does not support ${platform}/${arch}.`);
  }
  const legacy = `airs-harness-${platform}-${arch}`;
  if (manifest === undefined) return legacy;
  const scoped = `@cdot65/prisma-${legacy}`;
  const dependencies = manifest.optionalDependencies;
  const declared = [legacy, scoped].filter((name) =>
    dependencies && Object.hasOwn(dependencies, name));
  if (declared.length === 0) {
    throw new Error(`This airs-harness release does not include a native package for ${platform}/${arch}. Install a release that supports this platform.`);
  }
  if (declared.length !== 1) {
    throw new Error(`Multiple native packages are declared for ${platform}/${arch}; reinstall a valid airs-harness release.`);
  }
  const name = declared[0];
  if (typeof dependencies[name] !== "string" || !dependencies[name].trim() ||
      (name === scoped && dependencies[name] !== manifest.version)) {
    throw new Error("Invalid native dependency version; reinstall airs-harness.");
  }
  return name;
}

export function managedEnvironment(environment, platform = process.platform) {
  const env = { ...environment };
  let inheritedPath = env.PATH || "";
  if (platform === "win32") {
    // Match Node's selection when Windows receives duplicate case variants,
    // then pass one key so a stale variant cannot override the managed path.
    const pathKeys = Object.keys(env).filter((key) => key.toUpperCase() === "PATH").sort();
    inheritedPath = env[pathKeys[0]] || "";
    for (const key of pathKeys) delete env[key];
  }
  env.AIRS_MANAGED_CLI = path.join(managedCliDirectory, platform === "win32" ? "airs-cli.cmd" : "airs-cli");
  env.PATH = inheritedPath
    ? `${managedCliDirectory}${platform === "win32" ? ";" : ":"}${inheritedPath}`
    : managedCliDirectory;
  return env;
}

export function run({ legacy = false } = {}) {
  if (!checkNodeRuntime()) return;
  if (process.argv.length === 3 && process.argv[2] === "--migration-check") {
    process.stdout.write(JSON.stringify(inspectCommands(), null, 2) + "\n");
    return;
  }
  if (legacy && process.stderr.isTTY) {
    process.stderr.write("airs-harness is now airs; this compatibility command will be removed in alpha.23.\n");
  }
  if (process.argv[2] === "cli" || (legacy && process.argv[2] === "airs")) {
    runPrismaCli(process.argv.slice(3));
    return;
  }
  let binary;
  try {
    const launcherManifest = require("../package.json");
    const name = platformPackage(process.platform, process.arch, launcherManifest);
    let manifest;
    try {
      manifest = require.resolve(`${name}/package.json`);
    } catch {
      throw new Error(
        `The native package ${name} is unavailable. Reinstall ` +
        `${JSON.stringify(`${launcherManifest.name}@${launcherManifest.version}`)} with ` +
        `--include=optional, using the same registry and installation scope as the original install.`,
      );
    }
    const nativeManifest = require(manifest);
    const allowedNames = name.startsWith("@") ? [name] : [name, `@cdot65/prisma-${name}`];
    if (!allowedNames.includes(nativeManifest.name) || nativeManifest.version !== launcherManifest.version) {
      throw new Error("Native package identity/version mismatch; reinstall airs-harness.");
    }
    binary = path.join(path.dirname(manifest), "bin",
      process.platform === "win32" ? "airs-harness.exe" : "airs-harness");
    if (!existsSync(binary)) {
      throw new Error(`The native executable is missing from ${name}; reinstall airs-harness.`);
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
    return;
  }

  try {
    const cli = resolvePrismaCli();
    if (process.argv[2] === "completion" && process.argv.length === 4 &&
        !process.argv[3].startsWith("-")) {
      runCompletion(binary, cli, process.argv[3], managedEnvironment(process.env));
      return;
    }
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
    return;
  }
  runChild(binary, process.argv.slice(2), managedEnvironment(process.env));
}
