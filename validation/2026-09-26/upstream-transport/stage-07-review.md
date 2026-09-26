# gRPC transport policy adaptation

Adapted the code-mode slice of upstream `22a3f6d5d8`. Remote gRPC calls now use the shared managed HTTP client, preserve response trailers, and bind cached clients to the active account. A revoked client cannot keep an RPC alive; the provider can build a replacement after the new account policy is installed. Local Unix channels retain their existing local transport semantics. No new remote-execution capability is enabled.

The upstream test delegates were adapted to this fork's existing session-level API. No protocol redesign was imported. The final remaining external raw reqwest factory caller is migrated, so that factory method is now crate-private.

Linux and Apple Silicon suites each pass 142 tests with zero skips, including before-connect denial, cancellation and replacement-account recovery. Initial default V8 downloads failed on both platforms: reruns use the repository's checksum-verified Codex artifact/archive pairs, with no sandbox feature removed. Linux client-only diagnostic suite also passed all 69 tests (overlaps the 142).

Local scoped lint and format pass; native scoped lint passes without warnings. Native Bazel regeneration and strict check pass with unchanged lockfile bytes. No connected-feature score until application ownership, AIRS-specific CAS boundaries and full workspace gates pass.
