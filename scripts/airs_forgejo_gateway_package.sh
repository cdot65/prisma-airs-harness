#!/usr/bin/env bash
set -euo pipefail
umask 077
export PATH="/opt/homebrew/opt/python@3.13/libexec/bin:/opt/homebrew/bin:$HOME/.cargo/bin:$PATH"
: "${RUNNER_TEMP:?}" "${AIRS_RUNTIME_SOURCE:?}" "${AIRS_ARCHIVE_SHA256:?}" "${AIRS_BINARY_SHA256:?}" "${AIRS_FIXTURE_SHA256:?}"
test "$(uname -m)" = arm64
test "$(node -p 'require("./npm/airs-harness/package.json").version')" = 0.1.0-alpha.14
staged="$HOME/.local/share/airs-forgejo-runner/gateway-alpha14"
work="$RUNNER_TEMP/airs-gateway-package"
mkdir -p "$work/evidence"
retain_diagnostics() {
  outcome="$1"
  if [ "$outcome" -ne 0 ]; then
    private="$staged/private-run-$GITHUB_RUN_ID"
    mkdir -p -m 700 "$private"
    find "$work/evidence" -name '*.private.log' -exec cp {} "$private/" \;
    find "$private" -type f -exec chmod 600 {} \;
  fi
}
trap 'retain_diagnostics "$?"' EXIT
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
AIRS_HARNESS_BIN="$binary" python3 -m unittest discover -s scripts -p 'test_airs_harness*.py' -v > "$work/evidence/native-tests.log" 2>&1
# Interactive gateway E2E runs against the subsequently installed signed npm bytes.
# This job only produces an unvalidated candidate; it does not authorize promotion.
(cd runtime/codex-rs && cargo metadata --locked --filter-platform aarch64-apple-darwin --format-version 1) > "$work/metadata.json"
python3 scripts/package_airs_harness.py --source-directory runtime --binary "$binary" --metadata "$work/metadata.json" --target aarch64-apple-darwin --unvalidated-candidate --signing-receipt "$work/evidence/SIGNING.json" --output-directory "$work/mac-release" --profile 'release; CLI opt-level=1; lto=false; codegen-units=16; debug=0' --build-command 'cargo --config profile.release.package.codex-cli.opt-level=1 build --locked --release -p codex-cli --bin airs-harness'
# Retain exact signed intake and package for the subsequent combined npm checks.
mkdir -p "$staged/signed-output"
cp "$work/mac-release/"*.tar.gz "$work/signed.zip" "$staged/signed-output/"
cp "$work/evidence/"*.json "$staged/signed-output/"
