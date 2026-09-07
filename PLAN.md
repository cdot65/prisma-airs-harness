# Prisma AIRS Harness delivery plan

## Next distribution release — naming contract

Owner direction, reaffirmed for full implementation: product name **Prisma AIRS Harness**, executable `airs-harness`.
npm package: `airs-harness` (unscoped, for Verdaccio; not yet published). Adapt the
inherited npm launcher to distribute tested native binaries, beginning with
macOS endpoint support alongside Linux, and provide guided first-run setup.
Use the selected names throughout new onboarding, CLI help, UI and package metadata.
Preserve existing environments, encrypted credential bindings and session history
during the rename; do not blindly rename persisted identifiers or Keycloak clients.
The installed alpha.7 release and historical receipts still use `airs-terminal`.
The checkout and hosted PAH remain separate projects.

## Historical Linux MVP scope

Owner authorization: 2026-09-07. Continue autonomously toward the design and
implementation plan in the Prisma AIRS Terminal vault effort. Linux is the
current platform target; macOS is explicitly deferred. Workspace API-key mode
is the priority if Keycloak work encounters a platform blocker. Do not claim
complete team identity acceptance from a shared workspace credential.

1. Prove live Responses routing, synchronous AIRS policy, local tools and SSE.
2. Resolve sandbox failures while preserving filesystem/network restrictions.
3. Deliver named environment setup/select/status/doctor with isolated state,
   endpoint-bound credential references, secure-store login and explicit file
   input for headless operation. Never silently persist plaintext credentials.
4. Finish product branding and show gateway/auth/routing context in the UI.
5. Configure and prove remote MCP with separate credentials and a read-only tool.
6. Validate local skills, cancellation, resume, model changes and compaction.
7. Implement/prove public Keycloak authentication where the environment supports
   it; report the owner's authorized deferral if it cannot be completed.
8. Build/install/package the verified Linux binary, publish reproducible receipts,
   and assess each acceptance gate. Improve remaining failures before handoff.

## Owner acceptance correction

Alpha.4 failed the owner's actual interactive `/model` switch. Its provisional
9/10 score is withdrawn. Alpha.5 corrects the picker, request capability filtering
and runtime self-description. The new live and deterministic PTY path is required
before its installation/publication; see RELEASE.md and VALIDATION.json. Gateway
JSON-normalization guardrails are a separate capability under review.

## Earlier alpha.4 evidence

- Alpha.4 runtime implementation is committed in reviewable stages. Private
  source repository: `cdot65/prisma-airs-terminal`; inherited Actions are disabled.
- Focused Rust suite: 5,151 passed, seven skips. Additional CLI/MCP suite:
  972 passed, seven skips (overlaps CLI). Seventeen executable/PTY fixtures passed.
- Scoped Clippy, formatter and Bazel dependency metadata refresh pass. The full
  upstream workspace build is unavailable because V8 150.4.0's musl archive is
  missing. Code mode/Node REPL are disabled in the supported runtime catalog.
- Both real routes completed local skill/read/edit/test work and remote scanning.
  Cancel/resume and live/manual plus deterministic automatic compaction pass.
- A separate blocking scanner profile protects both routes. Wrong keys, model/
  config/provider overrides and forbidden MCP tools are tested server-side.
- Named environment and credential/history bindings fail closed. Interactive
  logout stays attached to the running environment after default/binary changes.
- Infrastructure changes are committed and pushed: DNS `533b30a`; MCP connection
  isolation `31fadca`, followed by stateless scanner deployment `5df8d39`.
  Both gateway replicas are ready at Helm revision 32. The final scanner image
  runs two replicas; the previous hosted MCP failure is avoided through REST.
- Final optimized build, installed/extracted-artifact checks and private prerelease
  publication were completed. [Alpha.4](https://github.com/cdot65/prisma-airs-terminal/releases/tag/airs-terminal-v0.1.0-alpha.4) was submitted for review and subsequently failed the interactive model-switch case.
  [Independent CI](https://github.com/cdot65/prisma-airs-terminal/actions/runs/34100831631) passed against the exact published archive: 17 executable
  fixtures, four scanner tests and zero production npm audit findings.
- Withdrawn historical assessment: **9/10 for the authorized Linux workspace-key pilot**.
  Owner E2E revealed the missed failure. The historical notes and post-publication
  `validation/2026-09-07/release-verification.json` record evidence and limitations.
  The full original team MVP remains unfinished.

## Authentication release progress

The installed alpha.6 binary passed the 27-check user-authentication flow, including
real OIDC model switching and refresh in one running TUI. The original user JWT
passes through the existing realm/JWKS to AIRS; native stores passed on Linux,
macOS and Windows. Twenty executable fixtures, seven workspace-key turns, 3,929
supported core tests, scoped Clippy and live MCP authorization checks passed.
Persisted security and management telemetry map to the signed user, including
spoof rejection. Only the owner has standing Terminal role grants.

Publication verification is recorded separately in `RELEASE-VERIFICATION.json`.
Owner hands-on review is the remaining acceptance step after artifact publication.
Full Windows/macOS terminal packaging and Conjur workload-secret activation are
separate milestones. The full upstream V8-dependent suite cannot build on this
musl host; the feature is disabled in the supported runtime.

No credential values belong in this document, source control, artifacts or vault.


## Authentication workflow polish (alpha.7)

Owner-confirmed alpha.6 E2E acceptance exposed recovery/status presentation gaps.
Alpha.7 adds platform-specific native-store recovery guidance, explicit MCP
credential-helper inventory, idempotent logout messaging and accurate doctor
identity wording. Existing gateway and Keycloak policy are unchanged. Current
artifact validation and publication evidence belong in VALIDATION.json and the
release verification receipt; alpha.6 test counts remain historical evidence.
