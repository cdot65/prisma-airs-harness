#!/usr/bin/env bash
set -euo pipefail
test "$(uname -m)" = arm64
test "$(uname -s)" = Darwin
: "${RUNNER_TEMP:?}"
staged="$HOME/.local/share/airs-forgejo-runner/frozen-alpha12"
evidence="$RUNNER_TEMP/airs-mac-acceptance"
mkdir -p "$evidence"
exec > >(tee "$evidence/validation.log") 2>&1
git rev-parse HEAD > "$evidence/tooling-source.txt"
python3 scripts/airs_macos_artifact.py restore \
  --archive "$staged/unvalidated-macos-executables.tar.gz" \
  --directory "$RUNNER_TEMP/airs-frozen-native" \
  --source-commit 19ce13981e92079ec29889896836d58d070cf106 \
  --archive-sha256 8bbb1ec231dddb8f6dbd397a98a753c8e7ff1461b6617bc760a550fee1dea447 \
  --binary-sha256 63500ffa8d3c7d47833242dc7dee908e626d85df2ebe88eec7d74f617a94138d \
  --fixture-sha256 f6533ba7d2514b6ca1766d8aa7b65493c965a4322a51d1a1937d667708f3fb24
cp "$RUNNER_TEMP/airs-frozen-native/ARTIFACT-VERIFIED.json" "$evidence/"
for binary in airs-harness store_acceptance; do
  test "$(lipo -archs "$RUNNER_TEMP/airs-frozen-native/$binary")" = arm64
  codesign --verify --strict --verbose=2 "$RUNNER_TEMP/airs-frozen-native/$binary"
done
python3 scripts/validate_airs_macos_keychain.py --binary "$RUNNER_TEMP/airs-frozen-native/airs-harness" --receipt "$evidence/native-keychain.json"
AIRS_HARNESS_TEST_EVIDENCE="$evidence/fixtures" AIRS_HARNESS_BIN="$RUNNER_TEMP/airs-frozen-native/airs-harness" python3 -m unittest discover -s scripts -p 'test_airs_harness*.py' -v
python3 scripts/validate_native_credentials.py --binary "$RUNNER_TEMP/airs-frozen-native/store_acceptance" --output "$evidence/native-store.json"
python3 scripts/validate_airs_npm.py --packages "$staged/npm-verdaccio" --prefix "$RUNNER_TEMP/AIRS Verdaccio Install"
cp "$RUNNER_TEMP/AIRS Verdaccio Install/INSTALL-VERIFICATION.json" "$evidence/"
cp "$RUNNER_TEMP/AIRS Verdaccio Install/INSTALL-NETWORK.json" "$evidence/"
AIRS_MANAGED_CLI_ACCEPTANCE=1 AIRS_HARNESS_BIN="$RUNNER_TEMP/AIRS Verdaccio Install/bin/airs-harness" python3 -m unittest discover -s scripts -p test_airs_harness.py -v
python3 scripts/validate_airs_macos_keychain.py --binary "$RUNNER_TEMP/AIRS Verdaccio Install/bin/airs-harness" --receipt "$evidence/installed-keychain.json"
