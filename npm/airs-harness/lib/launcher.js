import { runChild } from "./child.js";
import { managedCliDirectory, resolvePrismaCli, runPrismaCli } from "./prisma-cli.js";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

export function platformPackage(platform, arch) {
  if (platform === "darwin" && arch === "x64") {
    throw new Error("Prisma AIRS Harness supports Apple Silicon Macs only. " +
      "On Apple Silicon, use an arm64 Node.js installation outside Rosetta.");
  }
  if (!["linux", "darwin", "win32"].includes(platform) ||
      !["arm64", "x64"].includes(arch)) {
    throw new Error(`Prisma AIRS Harness does not support ${platform}/${arch}.`);
  }
  return `airs-harness-${platform}-${arch}`;
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
  env.AIRS_MANAGED_CLI = path.join(managedCliDirectory, platform === "win32" ? "airs.cmd" : "airs");
  env.PATH = inheritedPath
    ? `${managedCliDirectory}${platform === "win32" ? ";" : ":"}${inheritedPath}`
    : managedCliDirectory;
  return env;
}

export function run() {
  if (process.argv[2] === "airs") {
    runPrismaCli(process.argv.slice(3));
    return;
  }
  let binary;
  try {
    const name = platformPackage(process.platform, process.arch);
    let manifest;
    try {
      manifest = require.resolve(`${name}/package.json`);
    } catch {
      throw new Error(
        `The native package ${name} is unavailable. Install a release that supports ` +
        `this platform from your organization's registry with optional dependencies enabled.`,
      );
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
    resolvePrismaCli();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
    return;
  }
  runChild(binary, process.argv.slice(2), managedEnvironment(process.env));
}
