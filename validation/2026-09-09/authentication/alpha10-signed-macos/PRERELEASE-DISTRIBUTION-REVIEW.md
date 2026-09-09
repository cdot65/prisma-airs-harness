---
title: Independent signed Apple Silicon alpha10 owner-test distribution review
date: 2026-09-09
status: pass-for-bounded-prerelease-scope
tags: [prisma-airs-harness, authentication, macos, signing, release-review]
---

**Score: 9/10. Verdict: PASS for `signed-prerelease-distribution`, limited to
immutable P5 owner-test distribution of the reviewed Apple Silicon runtime.
Full authentication release: NOT READY.** Reviewer: `/root/credential_review`.
This score is a bounded distribution assessment, not the plan's full-platform
weighted score and not confirmation that the affected owner's incident is fixed.

Both completed workflows tested signed native SHA-256
`2099b66324eef6d1c63a6ba06da681163f60031767081833888e34626a4dee2b`, runtime source
`b012ba55e1404b498ad513cd15efa52a9a60530f`, version `0.1.0-alpha.10`.
Workflow/tooling source was `3f9f4d30f636971454e8bded83b320ebc70644fc`.
No Rust rebuild occurred in either signed acceptance workflow.

| Actual environment | Run | Native suite | Installed suite |
| --- | --- | --- | --- |
| macOS 26.6.2 arm64 | [34334951573](https://github.com/cdot65/airs-harness/actions/runs/34334951573) | 37 passed, 2 skipped / 39; 70.339s | 37 passed, 1 skipped / 38; 72.917s |
| macOS 15.7.9 arm64 | [34335108763](https://github.com/cdot65/airs-harness/actions/runs/34335108763) | 37 passed, 2 skipped / 39; 68.855s | 37 passed, 1 skipped / 38; 73.536s |

The native skips are the npm-managed CLI case, covered by the installed suite,
and Linux session-bus recovery, which is not a Mac test. The installed skip is
only Linux session-bus recovery. These are successful subsets with disclosed
skips, not 39/39 or 38/38 passes.

On each host, both original and installed native paths passed strict Apple-anchored
Developer ID verification with team `G5QLZ5A8TA`, hardened runtime and a signing
timestamp. Each also passed `codesign --check-notarization -R '=notarized'`.
The exact binary hash is checked before and after verification; application code
is not run before the initial trust checks. The owner-reported submission ID
`dc835ddf-8841-49bc-a7ee-2f370dcb3457` remains API-unqueried. The actual native
online notarization requirement establishes acceptance of these code bytes;
it does not independently authenticate the submission ID or its archive history.
Raw-CLI acceptance is not presented as an app-bundle `spctl` result.

The scoped assessment uses five two-point review areas:

| Area | Points | Evidence and reason |
| --- | ---: | --- |
| Runtime provenance | 2/2 | Exact signed digest on both hosts; independently verified signing-only code/data transformation; Mac26 native tar and npm-native members freshly rehashed to the same digest. |
| Distribution trust | 2/2 | Actual native and installed Apple trust/team/runtime/timestamp checks and online explicit notarization checks on macOS 15 and 26. |
| Installed dependency integrity | 2/2 | Ordinary scoped-name npm10.9.8 install with Node22.23.2; four staged requests, zero unexpected/public dependency fetches; 68 packages, 3,080 installed files, 66 licenses; pinned CLI5.2.0/SDK0.28.0; actual PDF/PNG/JPEG/SVG/DOCX generation. |
| Installed credential/runtime behavior | 2/2 | Actual native and npm-invoked Keychain save/private separate-process readback, authenticated loopback tool round trip, plaintext-state absence and logout denial; installed terminal suite passes on both hosts. |
| End-user distribution assurance | 1/2 | Automated signed npm-path behavior passes. Affected-owner consent/Keychain, real Mac OIDC/MCP, signed upgrade and nonowner registry access remain unverified. |

I independently verified all eight signature-log hashes and all 17 recorded
packaging-source hashes for each run. Mac26's archived launcher inventory matches
the installed receipt, and both npm manifests remain `private: true`. Existing
candidate metadata has not been rewritten to manufacture release evidence.
The new review-release packaging gate must construct new, source-bound metadata
around the unchanged signed runtime and validate those packages before publishing
only to the intended prerelease channel. This assessment does not approve an
uninspected future archive or claim a completed registry install.

The separate [Mach-O comparison](../alpha10-signed-intake/MACHO-SIGNING-REVIEW.md)
establishes unchanged code/data. Its uninterpreted trailing signature-allocation
bytes are not called a notarization ticket; actual macOS strict verification now
provides separate evidence of acceptance. The machine-readable review
[receipt](PRERELEASE-DISTRIBUTION-REVIEW.json) retains exact original evidence
hashes and provenance for both runs.

The withheld point requires unassisted affected-owner testing from the final
registry package, signed upgrade coverage, and verified repository/package-read
installation without owner credentials. Current GitHub package association is
unresolved; repository access alone must not be advertised as sufficient.
The actual Keychain fixture's OIDC check reaches an intentionally unavailable
issuer after native-store preflight; it is not a completed Mac Keycloak login or
refresh test. Its inference server is deterministic loopback. Linux live OIDC/MCP
results cannot fill those Mac-specific gaps.

Full authentication acceptance still requires the affected-owner incident retest,
selected-session and refresh concurrency/crash matrix, broader migration and
downgrade evidence, user onboarding trials, a complete scorecard, and P6/P7 gates.
No owner-machine or nonowner-authorization pass is inferred from GitHub-hosted
runners. Windows and containers retain the owner's deferred scope; Intel Macs
remain excluded. These limitations do not prevent distributing a clearly labeled
owner-test prerelease, but they prevent declaring the full specification complete.
