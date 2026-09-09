# Signed alpha.10 gated review-package acceptance

[Run 34336173227](https://github.com/cdot65/airs-harness/actions/runs/34336173227) completed successfully on macOS **26.6.2 ARM64**, using runtime `b012ba55e` and validation tooling `fcfb37141`. No Rust compilation, re-signing or stripping occurred.

| Check | Observed result |
| --- | --- |
| Native and installed Apple trust | Strict Developer ID/team G5QLZ5A8TA, hardened runtime and explicit notarized requirement passed with online ticket lookup |
| Native application tests | 37 passed, 2 explicit skips of 39 |
| Installed scoped npm application tests | 37 passed, 1 explicit skip of 38 |
| Native and installed actual CLI Keychain | Hidden login, exact private helper readback in another process, local tool loop, logout rejection and plaintext absence passed |
| Managed CLI 5.2.0 | Configuration/doctor, 20 command contracts, PDF/PNG/JPEG/SVG/DOCX generation passed |
| Fresh bundled scoped installation | npm 10.9.8, Node 22.23.2; four staged registry requests, zero unexpected requests or public redirects |
| Bundle inventory | 68 packages, 3,080 files, 66 license files; only two Darwin ARM64 Sharp payload packages |

The native npm-managed CLI skip is covered by the installed suite. The other skip is explicitly Linux session-bus recovery. These runs use the signed application's actual Keychain flows, not the separate unsigned raw-store fixture.

The signed native SHA256 remains `2099b66324eef6d1c63a6ba06da681163f60031767081833888e34626a4dee2b` in intake, native archive, native npm package and installed executable. All eight signature-log hashes and all 17 packaging source hashes were independently checked. The native archive and npm package preserve the exact `SIGNING.json` bytes and contain the bound review-release validation receipt plus all 11 exact referenced evidence files. Original evidence ZIP and receipt hashes are recorded in `ACCEPTANCE.json`; the large original package ZIP is retained privately and remains available by immutable GitHub artifact ID.

Apple's online explicit notarization requirement passed before execution and again after npm installation. The earlier failed offline requirement and non-app `spctl` assessment remain historical evidence; this report does not relabel either as passing. The user-reported submission ID was not independently queried through the notary submission API.

This is evidence-gated prerelease package acceptance, not registry publication or complete product readiness. The source remains b012; only package metadata and bundled documentation use the newer tooling. The workflow artifact name retains its historical `private-packages` label; actual archive contents carry the approved scoped review-release receipt and omit candidate markers. Installation used an isolated rejecting registry fixture, not a teammate's authenticated GitHub Packages account. Gateway traffic was deterministic loopback traffic. Affected-owner Mac recovery, live Mac Keycloak/AIRS use and final registry/distribution gates remain separate.
