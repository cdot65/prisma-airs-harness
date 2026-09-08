import { spawn } from "node:child_process";
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

export function run() {
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

  // Preserve the terminal, argument boundaries, credentials and working directory.
  // Never interpolate user arguments into a shell command.
  const child = spawn(binary, process.argv.slice(2), { stdio: "inherit" });
  const handlers = new Map();
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    const handler = () => {
      if (child.exitCode === null && child.signalCode === null) child.kill(signal);
    };
    handlers.set(signal, handler);
    process.on(signal, handler);
  }
  const cleanup = () => {
    for (const [signal, handler] of handlers) process.removeListener(signal, handler);
  };
  child.on("error", (error) => {
    cleanup();
    process.stderr.write(`Cannot start Prisma AIRS Harness: ${error.message}\n`);
    process.exitCode = 1;
  });
  child.on("exit", (code, signal) => {
    cleanup();
    if (signal) process.kill(process.pid, signal);
    else process.exitCode = code ?? 1;
  });
}
