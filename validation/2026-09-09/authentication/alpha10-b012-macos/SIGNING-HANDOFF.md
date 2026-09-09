# Sign the alpha10 Apple Silicon candidate

This is a maintainer signing input, not a published user installer. Compiled
source: `b012ba55e1404b498ad513cd15efa52a9a60530f`. GitHub run34298115142,
immutable artifact10084962634. The downloaded candidate retains its original
ad-hoc provenance; signing changes the executable bytes and requires new hashes.
Do not edit the old candidate metadata to claim that it was Developer ID signed.

## Download and verify

Use an authenticated GitHub CLI (`brew install gh`, then `gh auth login --web`
if needed). All commands run on the prepared Apple Silicon Mac.

```bash
mkdir -p ~/Downloads/prisma-airs-harness-alpha10-signing
cd ~/Downloads/prisma-airs-harness-alpha10-signing
gh run download 34298115142 --repo cdot65/airs-harness \
  --name airs-harness-darwin-arm64-unvalidated-executables-1
```

Alternatively download the [exact artifact](https://github.com/cdot65/airs-harness/actions/runs/34298115142/artifacts/10084962634)
in your signed-in browser, unzip it, and place the inner
`unvalidated-macos-executables.tar.gz` in that folder.

Run this block only after download completes. A checksum or architecture failure
stops the block before signing. The certificate stays in your Mac's Keychain.

```bash
bash <<'BASH'
set -euo pipefail
cd ~/Downloads/prisma-airs-harness-alpha10-signing
test "$(uname -m)" = arm64
printf '%s  %s\n' \
  a8c889f89e99d8a9dd43d801d24ee33bf8c19c3e4ddb81a06ba95ecd64ec4665 \
  unvalidated-macos-executables.tar.gz | shasum -a 256 -c -
test ! -e airs-harness
tar -xzf unvalidated-macos-executables.tar.gz airs-harness
printf '%s  %s\n' \
  3393285d9230f53cfd63702881b253e56f2f864e58acd7c407266167c875c5f0 \
  airs-harness | shasum -a 256 -c -
test "$(lipo -archs ./airs-harness)" = arm64
codesign --force \
  --sign 'Developer ID Application: Calvin Remsburg (G5QLZ5A8TA)' \
  --options runtime --timestamp ./airs-harness
codesign --verify --strict --verbose=2 ./airs-harness
codesign --display --verbose=4 ./airs-harness 2> signing-details.txt
ditto -c -k --keepParent ./airs-harness ./prisma-airs-harness-darwin-arm64-signed.zip
shasum -a 256 ./airs-harness ./prisma-airs-harness-darwin-arm64-signed.zip > SIGNED-SHA256.txt
xcrun notarytool submit ./prisma-airs-harness-darwin-arm64-signed.zip \
  --keychain-profile prisma-airs-harness-notary --wait --timeout 20m
BASH
```

Success is `status: Accepted`. If Apple is still processing after the wait
timeout, retain the submission ID and check it without re-signing or re-uploading:

```bash
xcrun notarytool info 'SUBMISSION-ID' --keychain-profile prisma-airs-harness-notary
```

If rejected, retrieve `notarytool log` for that ID before changing the artifact.
Do not staple the ZIP or raw command-line executable. Preserve the signed ZIP,
`SIGNED-SHA256.txt`, `signing-details.txt`, and accepted submission ID for intake.
The final npm package must preserve these signed executable bytes, add correct
license/provenance material, and pass installed Keychain/gateway/MCP/upgrade
acceptance before release readiness or publication is claimed.

Apple references: [Developer ID](https://developer.apple.com/developer-id/) and
[notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow).
