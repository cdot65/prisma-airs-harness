#!/usr/bin/env bash
set -euo pipefail
: "${RUNNER_TEMP:?}"
staged="$HOME/.local/share/airs-forgejo-runner/frozen-alpha12"
work="$RUNNER_TEMP/airs-signed-package"
mkdir -p "$work/evidence"
exec > >(tee "$work/evidence/packaging.log") 2>&1
source=19ce13981e92079ec29889896836d58d070cf106
archive_sha=${AIRS_SIGNED_ARCHIVE_SHA256:-5e368838f5f8ed43d2185679e8149931ef47c57473a35a24fabeb1b656decce9}
binary_sha=${AIRS_SIGNED_BINARY_SHA256:-bf9f36b5919502d2922550ab903556151762502bd3404a6e76a5ea935c060d66}
submission=${AIRS_NOTARY_SUBMISSION:-fe77f685-7323-4c55-b62f-9f2b1a868c72}
signed_archive=${AIRS_SIGNED_ARCHIVE_FILE:-$staged/prisma-airs-harness-alpha12-signed.zip}
signing_run=${AIRS_SIGNING_RUN:-forgejo-run-3230}
test "$(git -C runtime rev-parse HEAD)" = "$source"
python3 scripts/airs_signed_macos_artifact.py restore --asset-id "$signing_run" --archive "$signed_archive" --directory "$work/signed" --archive-sha256 "$archive_sha" --binary-sha256 "$binary_sha" --source-commit "$source"
python3 scripts/airs_signed_macos_artifact.py verify --asset-id "$signing_run" --binary "$work/signed/airs-harness" --receipt "$work/evidence/SIGNING.json" --archive-sha256 "$archive_sha" --binary-sha256 "$binary_sha" --source-commit "$source" --submission "$submission"
(cd runtime/codex-rs && cargo +1.95.0 metadata --locked --filter-platform aarch64-apple-darwin --format-version 1) > "$work/metadata.json"
python3 scripts/package_airs_harness.py --source-directory runtime --binary "$work/signed/airs-harness" --metadata "$work/metadata.json" --target aarch64-apple-darwin --unvalidated-candidate --signing-receipt "$work/evidence/SIGNING.json" --binary-processing none --output-directory "$work/mac-release" --profile 'release; CLI opt-level=1; lto=false; codegen-units=16; debug=0' --build-command 'cargo --config profile.release.package.codex-cli.opt-level=1 build --locked --release -p codex-cli --bin airs-harness'
tar -xzf "$work/mac-release/airs-harness-0.1.0-alpha.12-darwin-arm64.tar.gz" -C "$work/mac-release"
python3 scripts/verify_airs_release.py --directory "$work/mac-release" --receipt "$work/evidence/native-package-integrity.json"
printf '%s  %s\n' c3c7ef876dca148f9cedc19b83e4ab11b70cc65992f2fea59b6eee506adbb0a5 "$staged/airs-harness-0.1.0-alpha.12-linux-x86_64-musl.tar.gz" | shasum -a 256 -c -
mkdir "$work/linux-release"
tar -xzf "$staged/airs-harness-0.1.0-alpha.12-linux-x86_64-musl.tar.gz" -C "$work/linux-release"
python3 scripts/package_airs_npm.py --release-directory "$work/mac-release/airs-harness-0.1.0-alpha.12-darwin-arm64" --release-directory "$work/linux-release/airs-harness-0.1.0-alpha.12-linux-x86_64-musl" --output-directory "$work/npm" --registry https://npm.cdot.io --bundle-cli
prefix="$work/Signed Candidate Install"
python3 scripts/validate_airs_npm.py --packages "$work/npm" --prefix "$prefix"
cp "$prefix/INSTALL-VERIFICATION.json" "$prefix/INSTALL-NETWORK.json" "$work/evidence/"
# npm may hoist the optional platform package; resolve using Node's module lookup.
installed="$(node -e 'const p=require("path");const m=require.resolve("airs-harness-darwin-arm64/package.json",{paths:[process.argv[1]]});process.stdout.write(p.join(p.dirname(m),"bin/airs-harness"))' "$prefix/lib/node_modules/airs-harness")"
python3 scripts/airs_signed_macos_artifact.py verify --asset-id "$signing_run" --binary "$installed" --receipt "$work/evidence/INSTALLED-SIGNING.json" --archive-sha256 "$archive_sha" --binary-sha256 "$binary_sha" --source-commit "$source" --submission "$submission"
python3 scripts/validate_prisma_cli.py --launcher "$prefix/lib/node_modules/airs-harness/bin/airs-harness.js" --receipt "$work/evidence/managed-prisma-cli.json"
AIRS_MANAGED_CLI_ACCEPTANCE=1 AIRS_HARNESS_BIN="$prefix/bin/airs-harness" python3 -m unittest discover -s scripts -p test_airs_harness.py -v > "$work/evidence/installed-tests.log" 2>&1
python3 scripts/validate_airs_macos_keychain.py --binary "$prefix/bin/airs-harness" --receipt "$work/evidence/installed-keychain.json"
python3 - "$work" "$binary_sha" "$signing_run" <<'PY'
import hashlib, json, sys
from pathlib import Path
root=Path(sys.argv[1])
files={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((root/'npm/tarballs').glob('*.tgz'))}
receipt={'passed':True,'scope':'combined signed Mac and Linux npm tarballs; fresh Mac install, signatures, notarization, managed CLI and Keychain acceptance','tarball_sha256':files,'signed_binary_sha256':sys.argv[2],'signing_run':sys.argv[3],'runtime_source':'19ce13981e92079ec29889896836d58d070cf106','published':False,'linux_installed_acceptance':'separate dependent job','release_ready':False}
(root/'evidence/PACKAGE-ACCEPTANCE.json').write_text(json.dumps(receipt,indent=2)+'\n')
PY
