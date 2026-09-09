# Apple Silicon alpha10 compilation

Run34298115142 compiled runtime b012ba55e1404b498ad513cd15efa52a9a60530f
on macOS15 ARM64. The release compiler completed in 42m39s with CLI opt-level1,
LTO disabled, 16 codegen units and debug information disabled. The original
SHA-verified build-evidence ZIP is retained here, including Cargo timing and
fingerprint logs. Final native/installed acceptance is separate.

Native SHA-256:3393285d9230f53cfd63702881b253e56f2f864e58acd7c407266167c875c5f0.
Store fixture SHA-256:1f2dcce342d01a6de6698bc94e09d95155cb6ec59bed649df72aa6539c465757.
The candidate is ad-hoc signed; Developer ID signing and notarization are not
claimed by these build receipts.

This run predates the new sccache workflow. Its fingerprint log records142 dirty
reasons:31 changed-file timestamps,108 stale dependency fingerprints,2 changed
unit dependency records and1 other reason. For example, keyring-store's
diagnostics.rs was marked stale by its newer checkout timestamp. That file has
the identical Git object bf88afe9ba5bb3ff72a7926c67768db40fe43e33 in source154
and b012. This supports the need for content-based compiler caching alongside
Cargo target restoration; it is not proof of a future release-build speedup.
The high-fanout home-dir version change also exists in this release. The logs
do not justify assigning all rebuild time to a single cause.
