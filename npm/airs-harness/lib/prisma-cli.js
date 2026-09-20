import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runChild } from "./child.js";
import { checkNodeRuntime } from "./runtime.js";
import { resolveNative } from "./native.js";

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
  const command = manifest.bin?.["airs-cli"];
  if (typeof command !== "string" || !command || path.isAbsolute(command)) {
    throw new Error("Bundled Prisma AIRS CLI does not declare airs-cli; reinstall airs-harness.");
  }
  const root = path.dirname(manifestPath);
  const entry = path.resolve(root, command);
  if (!entry.startsWith(root + path.sep)) throw new Error("Invalid bundled CLI entrypoint.");
  if (!existsSync(entry)) throw new Error("Prisma AIRS CLI executable is missing; reinstall airs-harness.");
  return entry;
}

export function runPrismaCli(args = process.argv.slice(2)) {
  if (!checkNodeRuntime()) return;
  try {
    let skillHome;
    if (args[0] === "--harness-skill-home") {
      skillHome = args[1];
      if (!skillHome || !path.isAbsolute(skillHome)) throw new Error("The judge requires its absolute installed skill home.");
      args = args.slice(2);
      if (args[0] !== "redteam" || args[1] !== "judge") throw new Error("Skill credentials are only available to redteam judge.");
    }
    const entry = resolvePrismaCli();
    // dotenv/config also accepts argv overrides. Keep repository files out of
    // credential and endpoint resolution for this managed invocation.
    if (args.some((arg) => /^dotenv_config_/i.test(arg))) {
      throw new Error("dotenv argument overrides are not supported by the managed Prisma AIRS CLI. Use the selected tenant JSON configuration.");
    }
    const env = Object.fromEntries(Object.entries(process.env)
      .filter(([key]) => !key.toUpperCase().startsWith("DOTENV_CONFIG_")));
    env.DOTENV_CONFIG_PATH = path.join(managedCliDirectory, "empty.env");
    env.DOTENV_CONFIG_QUIET = "true";
    env.AIRS_CLI_INVOKED_AS = "airs cli";
    const providerAt = args.indexOf("--provider");
    const provider = providerAt >= 0 ? args[providerAt + 1]
      : args.find((arg) => arg.startsWith("--provider="))?.slice("--provider=".length) ?? "typesafe";
    const offline = args.some((arg) => ["--dry-run", "--help", "-h"].includes(arg)) || provider !== "typesafe";
    if (skillHome && !offline) {
      runChild(resolveNative(), ["env", "typesafe", "exec", "--skill-home", skillHome,
        "--", process.execPath, entry, ...args, "--harness-credentials"], env);
      return;
    }
    runChild(process.execPath, [entry, ...args], env);
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}
