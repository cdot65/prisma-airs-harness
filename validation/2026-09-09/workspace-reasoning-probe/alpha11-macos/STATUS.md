# Alpha.11 Apple Silicon release handoff

The optimized Mac build and compilation-cache save completed successfully in run `34378437580`. Native source is `ff5e337e4250771933ad012f0b89109ee9a22568`. Its immutable executable artifact is `10116550695`.

The original acceptance phase was cancelled only after build completion and cache preservation, because that run pinned obsolete alpha.10 fixture expectations. The same compiled bytes passed corrected artifact-only run `34382986822`; no Rust rebuild was scheduled.

The unsigned signing ZIP and checksum were uploaded to private draft intake `airs-harness-alpha11-signing-intake` (release `385691178`). Maintainer steps are in `administration/releases/alpha11-signing.md`. Developer ID signing must happen on the owner's provisioned Mac. The signature-only Mach-O comparison, Apple trust/notarization checks, signed native/installed acceptance, and npm publication remain pending.

This is a signing handoff, not a published alpha.11 Mac package or a completed authentication release.


## Completed unsigned artifact acceptance

- Native suite: 38 passed, two expected skips (npm-managed CLI and Linux D-Bus recovery).
- Installed scoped npm suite: 38 passed, one Linux-only recovery skip.
- Actual native and npm-installed Keychain lifecycle checks: passed.
- Bundled Prisma AIRS CLI 5.2.0 and local capability/document-generation checks: passed.
- Preserved native bytes, source commit and provenance: verified.

The exact acceptance evidence ZIP is retained here, with SHA256 in `ACCEPTANCE.json`. These are macOS26 ARM64 CI results, not owner-device or Developer ID/notarization evidence. The owner has been given the concrete checksum-bound signing workflow. No signed alpha.11 upload or npm publication exists yet.

Independent handoff review: 9/10, no blocking finding for owner signing. The review verifies the exact uploaded ZIP against the tested native executable and the signing guide. This score does not cover unsigned-to-signed acceptance or publication.
