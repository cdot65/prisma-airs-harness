import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runChild } from "./child.js";

const require = createRequire(import.meta.url);
const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const managedCliDirectory = path.join(packageRoot, "managed-cli");

export function resolvePrismaCli() {
  const expected = JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8"))
    .dependencies["@cdot65/prisma-airs-cli"];
  let manifestPath;
  try {
    manifestPath = require.resolve("@cdot65/prisma-airs-cli/package.json");
  } catch {
    throw new Error(`Required Prisma AIRS CLI ${expected} is missing; reinstall airs-harness with dependencies enabled.`);
  }
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.version !== expected) {
    throw new Error(`Prisma AIRS CLI version mismatch: expected ${expected}, found ${manifest.version}. Reinstall airs-harness.`);
  }
  const entry = path.resolve(path.dirname(manifestPath), manifest.bin.airs);
  if (!existsSync(entry)) throw new Error("Prisma AIRS CLI executable is missing; reinstall airs-harness.");
  return entry;
}

export function runPrismaCli(args = process.argv.slice(2)) {
  try {
    const entry = resolvePrismaCli();
    // dotenv/config also accepts argv overrides. Keep repository files out of
    // credential and endpoint resolution for this managed invocation.
    if (args.some((arg) => /^dotenv_config_/i.test(arg))) {
      throw new Error("dotenv argument overrides are not supported by the managed Prisma AIRS CLI. Use explicit environment variables or PRISMA_AIRS_CONFIG_PATH.");
    }
    const env = Object.fromEntries(Object.entries(process.env)
      .filter(([key]) => !key.toUpperCase().startsWith("DOTENV_CONFIG_")));
    env.DOTENV_CONFIG_PATH = path.join(managedCliDirectory, "empty.env");
    env.DOTENV_CONFIG_QUIET = "true";
    runChild(process.execPath, [entry, ...args], env);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
