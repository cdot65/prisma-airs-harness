# Sign Prisma AIRS Harness alpha.11

This is the maintainer signing step. The runtime source is
`ff5e337e4250771933ad012f0b89109ee9a22568`; it contains the workspace-key probe fix.
Use the Apple Silicon Mac where you previously configured the Developer ID identity
and the `prisma-airs-harness-notary` Keychain profile. No rebuild is required.

Download into a new directory:

```sh
mkdir -p ~/Downloads/prisma-airs-harness-alpha11
cd ~/Downloads/prisma-airs-harness-alpha11
gh release download airs-harness-alpha11-signing-intake --repo cdot65/airs-harness --pattern 'prisma-airs-harness-alpha11-unsigned.zip*'
shasum -a 256 -c prisma-airs-harness-alpha11-unsigned.zip.sha256
unzip prisma-airs-harness-alpha11-unsigned.zip
```

The checksum must report `OK`. Expected ZIP SHA256:

`5269e015c3b595de18bb7f694c2d89835743b887a21186cb2f8434ea29794ab5`

Then sign and submit:

```sh
codesign --force --timestamp --options runtime --sign 'Developer ID Application: Calvin Remsburg (G5QLZ5A8TA)' prisma-airs-harness-signing/airs-harness
codesign --verify --strict --verbose=2 prisma-airs-harness-signing/airs-harness
zip -X prisma-airs-harness-alpha11-signed.zip prisma-airs-harness-signing/airs-harness
xcrun notarytool submit prisma-airs-harness-alpha11-signed.zip --keychain-profile prisma-airs-harness-notary --wait --output-format json > notarization-alpha11.json
cat notarization-alpha11.json
```

Continue only when notarization reports `Accepted`. Verify Apple's ticket and upload:

```sh
codesign --verify --strict --verbose=4 --check-notarization -R '=notarized' prisma-airs-harness-signing/airs-harness
shasum -a 256 prisma-airs-harness-signing/airs-harness prisma-airs-harness-alpha11-signed.zip > signed-alpha11.sha256
gh release upload airs-harness-alpha11-signing-intake --repo cdot65/airs-harness prisma-airs-harness-alpha11-signed.zip signed-alpha11.sha256 notarization-alpha11.json
```

Report that the upload is complete. Release validation will independently check the
signed bytes, Apple trust, Keychain, the reasoning-only probe, and fresh npm installation.
Signing this artifact does not itself publish an npm update.
