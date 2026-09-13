# Apple Silicon signing handoff

This signs the already validated alpha.12 runtime from source
`19ce13981e92079ec29889896836d58d070cf106`; no rebuild is needed.
Use an Apple Silicon Mac with the project's Developer ID identity and the
`prisma-airs-harness-notary` Keychain profile configured. Their availability has
not been verified in this implementation environment. These commands have not
been run by the implementer.

Download the immutable private artifact into a new directory:

```sh
mkdir -p ~/Downloads/prisma-airs-harness-alpha12-signing
cd ~/Downloads/prisma-airs-harness-alpha12-signing
gh run download 34739344591 --repo cdot65/airs-harness --name airs-harness-darwin-arm64-unvalidated-executables-1
cat > ORIGINAL-SHA256SUMS <<'SUMS'
8bbb1ec231dddb8f6dbd397a98a753c8e7ff1461b6617bc760a550fee1dea447  unvalidated-macos-executables.tar.gz
SUMS
shasum -a 256 -c ORIGINAL-SHA256SUMS
tar -xzf unvalidated-macos-executables.tar.gz
cat > EXECUTABLE-SHA256SUMS <<'SUMS'
63500ffa8d3c7d47833242dc7dee908e626d85df2ebe88eec7d74f617a94138d  airs-harness
SUMS
shasum -a 256 -c EXECUTABLE-SHA256SUMS
```

Both checks must report `OK`. Then prepare the exact intake layout, sign and
submit to Apple:

```sh
mkdir prisma-airs-harness-signing
cp airs-harness prisma-airs-harness-signing/airs-harness
codesign --force --timestamp --options runtime --sign 'Developer ID Application: Calvin Remsburg (G5QLZ5A8TA)' prisma-airs-harness-signing/airs-harness
codesign --verify --strict --verbose=2 prisma-airs-harness-signing/airs-harness
zip -X prisma-airs-harness-alpha12-signed.zip prisma-airs-harness-signing/airs-harness
xcrun notarytool submit prisma-airs-harness-alpha12-signed.zip --keychain-profile prisma-airs-harness-notary --wait --output-format json > notarization-alpha12.json
```

Continue only when Apple's response is `Accepted`:

```sh
codesign --verify --strict --verbose=4 --check-notarization -R '=notarized' prisma-airs-harness-signing/airs-harness
shasum -a 256 prisma-airs-harness-signing/airs-harness prisma-airs-harness-alpha12-signed.zip > signed-alpha12.sha256
```

Retain the signed ZIP, checksum file and notarization receipt privately. The
existing owned `airs-harness-macos-signed.yml` workflow can independently verify
an immutable signed intake with its asset ID, both new hashes, Apple's accepted
submission UUID and the original runtime source above. It then repeats signed
artifact/Keychain/npm acceptance. This handoff does not upload or publish an
artifact, authorize publication, or turn an owner-reported submission UUID into
independent notarization proof.
