import { runChild } from "./child.js";
import { runCompletion } from "./completion.js";
import { inspectCommands } from "./migration-check.js";
import { managedCliDirectory, resolvePrismaCli, runPrismaCli } from "./prisma-cli.js";
import { checkNodeRuntime } from "./runtime.js";
import path from "node:path";

import { resolveNative } from "./native.js";
export { platformPackage } from "./native.js";

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
    binary = resolveNative();
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
