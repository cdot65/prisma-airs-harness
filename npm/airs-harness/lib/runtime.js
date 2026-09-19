import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const range = require("../package.json").engines?.node;

function stableVersion(value) {
  if (typeof value !== "string" || value.length > 128) return undefined;
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:\+[\da-zA-Z-]+(?:\.[\da-zA-Z-]+)*)?$/.exec(value);
  if (!match || match[0] !== value) return undefined;
  const parts = match.slice(1, 4).map(Number);
  return parts.every(Number.isSafeInteger) ? parts : undefined;
}

function atLeast(version, floor) {
  for (let index = 0; index < 3; index++) {
    if (version[index] !== floor[index]) return version[index] > floor[index];
  }
  return true;
}

export function checkNodeRuntime() {
  // Support the manifest's stable caret-major / minimum-version union directly.
  // An unrecognized declaration fails closed rather than drifting from npm.
  const declaration = typeof range === "string" && /^\^([^ ]+) \|\| >=([^ ]+)$/.exec(range);
  const lower = declaration && stableVersion(declaration[1]);
  const higher = declaration && stableVersion(declaration[2]);
  if (!declaration || declaration[0] !== range || !lower || lower[0] === 0 || !higher) {
    process.stderr.write("The Prisma AIRS Harness Node.js engine declaration is invalid. Reinstall a valid release.\n");
    process.exitCode = 1;
    return false;
  }
  const version = stableVersion(process.versions.node);
  if (version && ((version[0] === lower[0] && atLeast(version, lower)) || atLeast(version, higher))) {
    return true;
  }
  const found = JSON.stringify(String(process.versions.node).slice(0, 128));
  process.stderr.write(`Prisma AIRS Harness requires Node.js ${range}; found ${found}. Upgrade Node.js to match this range, check node --version in this terminal, then retry.\n`);
  process.exitCode = 1;
  return false;
}
