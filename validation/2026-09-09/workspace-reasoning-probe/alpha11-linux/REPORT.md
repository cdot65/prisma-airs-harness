# Alpha.11 Linux acceptance

Runtime source: `ff5e337e4250771933ad012f0b89109ee9a22568`.
Native SHA256: `eb919c76b3fe4129bff2a389b6dcf2a77952914b68f32091bfe64a4ab67079bb`.
The optimized build completed in 19m48s using the existing release dependency cache.

- Native executable suite: 39 passed, one managed-CLI-only check deferred to packaging.
- Fresh scoped npm candidate installation: passed; all 39 installed executable checks passed.
- Bundled Prisma AIRS CLI validation: passed (local capabilities and synthetic document generation; not live scanner detection).
- Live workspace API-key workflow: guided hidden login, native Secret Service, successful bounded gateway probe, default-route inference, local file effect, new-process resume, logout and post-logout denial passed. No plaintext credential appeared in fixture state or transcripts.
- Alpha.10 native-store upgrade: the existing credential and session were preserved through upgrade and executable relocation. Every captured default-route request omitted model. The old alpha.9-specific rollback scenario was explicitly excluded; no downgrade claim is made.

The new reasoning-only executable regression was also run against the published alpha.10 binary: both completed and incomplete cases failed as expected. The new runtime passes both. The probe still uses 16 output tokens and does not require a final message after valid reasoning output.

## Scope and remaining gates

This is Linux candidate acceptance, not publication or macOS acceptance. The native candidate archive and npm packages remain marked unpublishable until release attestation is assembled. Apple Silicon build run34378437580 is still pending. New Mac bytes must be signed, notarized and tested before the GitHub Packages review channel is updated. No gateway configuration was changed.

The independent 9/10 assessment covers reviewed tooling only. It does not claim the pending release is ready.

## Corrected operator checks

An initial upgrade invocation accidentally supplied the npm JavaScript launcher with the native binary hash; the hash guard rejected it before execution. Retrying with the actual native executable exposed the fixture's alpha.9-only keyring assertion. A reviewed `--upgrade-only` scenario now requires native keyring-v2 and retains strict identity/history preservation checks. Its actual alpha.10-to-alpha.11 run passed. An initial packaging invocation lacked rustc in PATH; the final candidate was generated with the Rust toolchain available. Neither issue changed the executable bytes.
