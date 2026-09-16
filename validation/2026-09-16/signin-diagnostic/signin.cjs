#!/usr/bin/env node
const {spawnSync} = require('node:child_process');
const path = require('node:path');
const platform = process.platform + '-' + process.arch;
if (!['linux-x64', 'linux-arm64', 'darwin-arm64'].includes(platform)) {
  console.error('Unsupported diagnostic platform: ' + platform);
  process.exit(1);
}
const result = spawnSync(path.join(__dirname, 'bin', platform), process.argv.slice(2), {stdio: 'inherit'});
if (result.error) console.error('Could not start the sign-in diagnostic: ' + result.error.code);
process.exit(result.status ?? 1);
