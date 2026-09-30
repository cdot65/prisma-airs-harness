import { existsSync, readFileSync, realpathSync, statSync } from "node:fs";
import path from "node:path";

// Inspect links and bounded package manifests only. Never execute a discovered
// command or change another installer's files to resolve command ownership.
export function inspectCommands(environment = process.env) {
  const suffixes = process.platform === "win32" ? [".cmd", ".exe", ""] : [""];
  const directories = [...new Set((environment.PATH || "").split(path.delimiter).filter(Boolean))];
  const commands = {};
  for (const name of ["airs", "airs-cli", "airs-harness"]) {
    commands[name] = [];
    for (const directory of directories) {
      for (const suffix of suffixes) {
        const executable = path.resolve(directory, name + suffix);
        if (!existsSync(executable)) continue;
        try {
          const target = realpathSync(executable);
          let parent = path.dirname(target);
          let owner = null;
          for (let depth = 0; depth < 7; depth++) {
            const manifest = path.join(parent, "package.json");
            if (existsSync(manifest) && statSync(manifest).size < 1024 * 1024) {
              const value = JSON.parse(readFileSync(manifest, "utf8"));
              if (["airs-harness", "prisma-airs-harness", "@cdot65/prisma-airs-cli"].includes(value.name)) {
                owner = { name: value.name, version: value.version };
                break;
              }
            }
            const next = path.dirname(parent);
            if (next === parent) break;
            parent = next;
          }
          commands[name].push({ executable, target, owner });
        } catch {
          commands[name].push({ executable, inspection: "unavailable" });
        }
      }
    }
  }
  return {
    commands,
    modified: false,
    guidance: "Upgrade an old standalone CLI to 7.0.0 with its owning package manager before installing the renamed harness. Unknown files require manual ownership review; do not use npm --force. Shell aliases/functions are not visible to this process: inspect type -a airs airs-cli airs-harness in your shell.",
  };
}
