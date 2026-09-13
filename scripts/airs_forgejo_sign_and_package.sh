#!/usr/bin/env bash
set -euo pipefail
: "${RUNNER_TEMP:?}"
work="$RUNNER_TEMP/airs-new-signing"
evidence="$RUNNER_TEMP/airs-signed-package/evidence"
mkdir -p "$work" "$evidence"
python3 scripts/airs_macos_artifact.py restore \
  --archive "$HOME/.local/share/airs-forgejo-runner/frozen-alpha12/forgejo-3232-executables.tar.gz" \
  --directory "$work/compiled" \
  --source-commit 19ce13981e92079ec29889896836d58d070cf106 \
  --archive-sha256 937495028336975c0cabb2cb2c8693ec7fd5d16669dd35633e9996cf89194778 \
  --binary-sha256 8f39c9493423b3fd7dced061826d20f741a3b4c1a6fbe43d198394377625e502 \
  --fixture-sha256 f7220368daead37a86b0888ed3db6218100380b447864b1c25b8ccff2a23e7be
cp "$work/compiled/ARTIFACT-VERIFIED.json" "$evidence/FORGEJO-BUILD-3232.json"
mkdir "$work/prisma-airs-harness-signing"
cp "$work/compiled/airs-harness" "$work/prisma-airs-harness-signing/airs-harness"
python3 - "$work/prisma-airs-harness-signing/airs-harness" <<'PY'
import subprocess, sys
subprocess.run(['codesign','--force','--timestamp','--options','runtime','--keychain','/Users/cdot/Library/Keychains/login.keychain-db','--sign','9E9F6E0D92526D725FABC1BBDC2F1757049F1805',sys.argv[1]],check=True,timeout=120)
PY
(cd "$work" && zip -X signed.zip prisma-airs-harness-signing/airs-harness)
xcrun notarytool submit "$work/signed.zip" --keychain-profile prisma-airs-harness-notary --keychain "$HOME/Library/Keychains/login.keychain-db" --wait --timeout 30m --output-format json > "$evidence/NOTARIZATION.json"
AIRS_NOTARY_SUBMISSION="$(python3 - "$evidence/NOTARIZATION.json" <<'PY'
import json,sys
r=json.load(open(sys.argv[1])); assert r['status']=='Accepted',r
print(r['id'])
PY
)"
export AIRS_NOTARY_SUBMISSION
export AIRS_SIGNED_ARCHIVE_FILE="$work/signed.zip"
export AIRS_SIGNED_ARCHIVE_SHA256="$(shasum -a 256 "$work/signed.zip" | cut -d ' ' -f 1)"
export AIRS_SIGNED_BINARY_SHA256="$(shasum -a 256 "$work/prisma-airs-harness-signing/airs-harness" | cut -d ' ' -f 1)"
export AIRS_SIGNING_RUN="forgejo-run-$GITHUB_RUN_ID"
cp "$work/signed.zip" "$evidence/SIGNED-INTAKE.zip"
bash scripts/airs_forgejo_signed_package.sh
