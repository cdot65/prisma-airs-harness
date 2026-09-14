#!/usr/bin/env bash
set -euo pipefail
export PATH="/opt/homebrew/opt/python@3.13/libexec/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
: "${RUNNER_TEMP:?}"
test "$(uname -m)" = arm64
staged="$HOME/.local/share/airs-forgejo-runner/mcp-alpha13"
work="$RUNNER_TEMP/airs-mcp-npm"
mkdir -p "$work/evidence" "$work/native"
tar -xzf "$staged/linux-release.tar.gz" -C "$work/native"
tar -xzf "$staged/signed-output/airs-harness-0.1.0-alpha.13-darwin-arm64.tar.gz" -C "$work/native"
python3 scripts/package_airs_npm.py --release-directory "$work/native/airs-harness-0.1.0-alpha.13-linux-x86_64-musl" --release-directory "$work/native/airs-harness-0.1.0-alpha.13-darwin-arm64" --output-directory "$work/candidate-npm" --registry https://npm.cdot.io --bundle-cli
prefix="$work/installed"
python3 scripts/validate_airs_npm.py --packages "$work/candidate-npm" --prefix "$prefix"
cp "$prefix/INSTALL-VERIFICATION.json" "$prefix/INSTALL-NETWORK.json" "$work/evidence/"
python3 scripts/validate_airs_npm_upgrade.py --packages "$work/candidate-npm" --output "$work/upgrade"
cp "$work/upgrade/UPGRADE.json" "$work/evidence/"
AIRS_MANAGED_CLI_ACCEPTANCE=1 AIRS_HARNESS_BIN="$prefix/bin/airs-harness" python3 -m unittest discover -s scripts -p 'test_airs_harness*.py' -v > "$work/evidence/installed-tests.log" 2>&1
python3 scripts/validate_airs_macos_keychain.py --binary "$prefix/bin/airs-harness" --receipt "$work/evidence/INSTALLED-KEYCHAIN.json"
python3 scripts/validate_builtin_mcp.py --binary "$prefix/bin/airs-harness" --fixture "$staged/acceptance-user.json" --state "$work/oauth-state" --output "$work/evidence/INSTALLED-MCP-E2E.json"
# The next Linux validation uses these exact combined archives, never repacks them.
mkdir -p "$staged/npm-output"
tar -czf "$staged/npm-output/candidate-npm.tar.gz" -C "$work" candidate-npm
cp "$work/evidence/"*.json "$work/evidence/installed-tests.log" "$staged/npm-output/"
