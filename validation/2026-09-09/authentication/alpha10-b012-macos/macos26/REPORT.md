# Alpha.10 hosted macOS 26 acceptance

[Run 34301272838](https://github.com/cdot65/airs-harness/actions/runs/34301272838)
completed successfully on hosted macOS **26.6.2 ARM64**. The acceptance job took
616 seconds and reused the preserved source `b012ba55e` binary from
build run `34298115142`, executable artifact `10084962634`. It did not compile
Rust. Validation/packaging tooling was `9e21c6bc1`.

| Check | Observed result |
| --- | --- |
| Native application tests | 37 passed; 2 explicit skips out of 39 methods |
| Installed scoped npm application tests | 37 passed; 1 Linux-only skip out of 38 methods |
| Raw Keychain fixture | 7 separate-process phases passed, including 16 KiB workspace key storage |
| Native CLI and installed CLI Keychain | Both passed hidden login, private helper readback, local tool loop, logout rejection and plaintext absence |
| Managed CLI 5.2.0 | Configuration/doctor, 20 command contracts and actual PDF/PNG/JPEG/SVG/DOCX generation passed |
| Scoped bundled npm install | npm 10.9.8 / Node 22.23.2; exactly 4 staged registry requests; no unexpected requests or public dependency redirects |
| Installed bundle inventory | 68 packages, 3,080 files and 66 license files verified; only the two Darwin ARM64 Sharp payload packages |

The native skip for the npm-managed CLI is covered by the installed suite. The
other native and installed skip is Linux session-bus recovery, which is not a
macOS check. Exact logs and original receipts are retained beside this report.

The installed executable SHA256 is
`3393285d9230f53cfd63702881b253e56f2f864e58acd7c407266167c875c5f0`,
matching the independently supplied build identity. The selected inner native
archive SHA256 is
`a8c889f89e99d8a9dd43d801d24ee33bf8c19c3e4ddb81a06ba95ecd64ec4665`.
The original evidence ZIP was downloaded by immutable artifact ID `10085235552`;
its SHA256 matched GitHub's digest before parsing:
`bb606c4bb09eb45fc60966d898af0dbd585e31b88fbe4e1c144ee4190c870c85`.
All 17 recorded packaging source-file hashes independently match Git
objects at the recorded tooling revision. `ACCEPTANCE.json` records identities,
counts and retained file hashes.

This is an **unpublished private candidate with ad hoc signing**. The ordinary
scoped package-name install used the rejecting loopback fixture registry, not a
teammate's authenticated GitHub Packages installation. CLI gateway traffic used
a deterministic local server; this run does not prove live Mac Keycloak/AIRS
access, owner-device recovery, signed/notarized distribution or a complete Mac
upgrade/downgrade workflow. Those release and owner-review gates remain separate.
