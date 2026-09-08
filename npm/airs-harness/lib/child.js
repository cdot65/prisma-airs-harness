import { spawn } from "node:child_process";

// Preserve argument boundaries and terminal ownership without shell interpolation.
export function runChild(executable, args, env = process.env) {
  const child = spawn(executable, args, { stdio: "inherit", env });
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
    process.stderr.write(`Cannot start Prisma AIRS command: ${error.message}\n`);
    process.exitCode = 1;
  });
  child.on("exit", (code, signal) => {
    cleanup();
    if (signal) process.kill(process.pid, signal);
    else process.exitCode = code ?? 1;
  });
}
