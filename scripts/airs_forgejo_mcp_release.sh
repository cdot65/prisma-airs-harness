#!/usr/bin/env bash
set -euo pipefail
export PATH="/opt/homebrew/opt/python@3.13/libexec/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
: "${RUNNER_TEMP:?}" "${AIRS_RUNTIME_SOURCE:?}" "${AIRS_ARCHIVE_SHA256:?}" "${AIRS_BINARY_SHA256:?}" "${AIRS_FIXTURE_SHA256:?}"
test "$(uname -m)" = arm64
staged="$HOME/.local/share/airs-forgejo-runner/mcp-alpha13"
work="$RUNNER_TEMP/airs-mcp-release"
mkdir -p "$work/evidence"
test "$(git -C runtime rev-parse HEAD)" = "$AIRS_RUNTIME_SOURCE"
python3 scripts/airs_macos_artifact.py restore --archive "$staged/intake.tar.gz" --directory "$work/compiled" --source-commit "$AIRS_RUNTIME_SOURCE" --archive-sha256 "$AIRS_ARCHIVE_SHA256" --binary-sha256 "$AIRS_BINARY_SHA256" --fixture-sha256 "$AIRS_FIXTURE_SHA256"
cp "$work/compiled/ARTIFACT-VERIFIED.json" "$work/evidence/NATIVE-BUILD.json"
mkdir "$work/prisma-airs-harness-signing"
cp "$work/compiled/airs-harness" "$work/prisma-airs-harness-signing/airs-harness"
binary="$work/prisma-airs-harness-signing/airs-harness"
codesign --force --timestamp --options runtime --keychain "$HOME/Library/Keychains/login.keychain-db" --sign 9E9F6E0D92526D725FABC1BBDC2F1757049F1805 "$binary"
(cd "$work" && zip -X signed.zip prisma-airs-harness-signing/airs-harness)
xcrun notarytool submit "$work/signed.zip" --keychain-profile prisma-airs-harness-notary --keychain "$HOME/Library/Keychains/login.keychain-db" --wait --timeout 30m --output-format json > "$work/evidence/NOTARIZATION.json"
submission="$(python3 - "$work/evidence/NOTARIZATION.json" <<'PY'
import json,sys
r=json.load(open(sys.argv[1]));assert r['status']=='Accepted';print(r['id'])
PY
)"
archive_sha="$(shasum -a 256 "$work/signed.zip" | cut -d ' ' -f 1)"
binary_sha="$(shasum -a 256 "$binary" | cut -d ' ' -f 1)"
python3 scripts/airs_signed_macos_artifact.py verify --asset-id "forgejo-run-$GITHUB_RUN_ID" --binary "$binary" --receipt "$work/evidence/SIGNING.json" --archive-sha256 "$archive_sha" --binary-sha256 "$binary_sha" --source-commit "$AIRS_RUNTIME_SOURCE" --submission "$submission"
python3 scripts/validate_airs_macos_keychain.py --binary "$binary" --receipt "$work/evidence/NATIVE-KEYCHAIN.json"
python3 scripts/validate_builtin_mcp.py --binary "$binary" --fixture "$staged/acceptance-user.json" --state "$work/oauth-state" --output "$work/evidence/MCP-E2E.json" --refresh-cycles 2
(cd runtime/codex-rs && cargo metadata --locked --filter-platform aarch64-apple-darwin --format-version 1) > "$work/metadata.json"
python3 scripts/package_airs_harness.py --source-directory runtime --binary "$binary" --metadata "$work/metadata.json" --target aarch64-apple-darwin --unvalidated-candidate --signing-receipt "$work/evidence/SIGNING.json" --output-directory "$work/mac-release" --profile 'release; CLI opt-level=1; lto=false; codegen-units=16; debug=0' --build-command 'cargo --config profile.release.package.codex-cli.opt-level=1 build --locked --release -p codex-cli --bin airs-harness'
# Retain exact signed intake and package for the subsequent combined npm checks.
mkdir -p "$staged/signed-output"
cp "$work/mac-release/"*.tar.gz "$work/signed.zip" "$staged/signed-output/"
cp "$work/evidence/"*.json "$staged/signed-output/"
