# Alpha.19 Linux DNS repair — September 15, 2026

The owner could fetch OIDC discovery with curl on an Omarchy Linux ARM64 VM,
but alpha.18 reported `issuer discovery unavailable`. The alpha.18 native HTTP
library diagnostic failed in DNS, before TLS. Node resolved the A record to
`192.168.1.0` and returned `ENOTFOUND` for AAAA. A local DNS fixture reproduced
musl discarding the usable A answer when AAAA returned NXDOMAIN.

Alpha.19 enables reqwest's Hickory resolver for Linux musl in both the identity
crate and shared HTTP client. It retains configured DNS servers and resolves
address families independently. The native inference client, MCP transport and
MCP OAuth adapters use that shared HTTP client. macOS retains its existing
resolver. Discovery and JWKS request errors preserve their causes without the
request URL. Runtime source is `20f37e5e31c832f34123caaf7a255154987fbe33`.

The 30-minute idle policy, gateway destinations and authentication recovery
behavior are unchanged. Docusaurus remains under the owner's separate review.

## Verification

Launcher tests: 16 passed. Packaging tests: 13 passed. Scoped Rust tests:
119 passed, six failed. The unchanged alpha.18 baseline has the same six TLS
fallback/certificate-classification failures (118 passed). The new discovery
error regression passes; `SCOPED-REGRESSION.json` lists the exact comparison.
Bazel lock update and drift check passed on Apple Silicon, with no lockfile drift.

The final Cargo-built default-client HTTPS probe passes three cases on Linux
x64 and ARM64 under QEMU: A succeeds with AAAA NXDOMAIN; A succeeds with AAAA
NODATA; both NXDOMAIN fails without an HTTPS request. Successful cases validate
TLS against an isolated fixture CA. The validator gives only the emulated child
a test resolv.conf and never changes host DNS. These checks do not claim native
ARM64 host acceptance. The original alpha.18 DNS reproduction is preserved.

The validation script was corrected after freezing the runtime: use the reserved
`.test` domain because Hickory rejects `.invalid` locally; create a separate CA
and server leaf certificate; allow distinct loopback DNS fixture addresses for
concurrent architecture checks. These are test-tooling changes, not changes to
the shipped Rust binary. The final validator hash is recorded separately.

## Distribution

All four packages are published under `gateway-validation` at
`https://npm.cdot.io`: launcher, Linux x64, Linux ARM64 and Apple Silicon.
Install the exact version:

```sh
npm install -g airs-harness@0.1.0-alpha.19 --registry=https://npm.cdot.io
airs-harness -V
```

The `latest` and `alpha` launcher tags remain alpha.14. The owner explicitly
requested candidate publication and will test the ARM64 VM personally. Native
ARM64 installed acceptance and full production OAuth lifecycle acceptance remain
pending; package metadata preserves those limits. No VM SSH session or browser
consent was required for this repair.

Linux x64 and Mac upgrades from alpha.14 and alpha.18 passed, followed by fresh
anonymous registry installs and native executable checks. The Mac binary is
Developer ID signed and notarized. The ARM64 archive was downloaded anonymously;
its integrity, embedded binary hash and launcher dependency were verified.

Receipts distinguish original private candidates from the published candidate
manifests. Native bytes and runtime source are unchanged by publication.
