---
title: Independent Linux alpha10 owner-test distribution review
date: 2026-09-09
status: pass-for-bounded-prerelease-scope
tags: [prisma-airs-harness, authentication, linux, release-review]
---

**Score: 9/10. Verdict: PASS for `signed-prerelease-distribution`, limited to the
reviewed Linux x64 runtime entering the immutable P5 owner-test channel. Full
authentication release: NOT READY.** Linux uses provenance/checksums; the scope
name does not claim that this Linux ELF has an Apple or Windows code signature.

Reviewer: `/root/credential_review`. This is a scoped reviewer assessment, not the
authentication plan's final weighted platform score. It does not supersede the
[earlier independent review](INDEPENDENT-REVIEW.md), which correctly withheld a
whole-delivery score. Publication and final registry-download acceptance are
subsequent actions, not results invented by this review.

The exact runtime is `0.1.0-alpha.10`, source
`b012ba55e1404b498ad513cd15efa52a9a60530f`, target
`x86_64-unknown-linux-musl`, SHA-256
`a9166568623ce602da606a668359003e1f0a1380a202790da77f0a83dd1dc6ff`.
I freshly rehashed both the retained ELF and the installed native ELF to that
value. The installed inventory hash matches its receipt. All 17 recorded
packaging-source hashes match the recorded git source. The retained JUnit parses
as 388 tests, no failures, errors or skips. No live tests were rerun for this audit.

The scoped assessment uses five two-point review areas:

| Area | Points | Evidence and reason |
| --- | ---: | --- |
| Exact artifact provenance | 2/2 | Fresh native and installed hashes, source-bound build and packaging inputs, retained checksums/notices. |
| Installed runtime behavior | 2/2 | 388 scoped Rust checks; 38 installed executable cases; the native suite's npm-only skip is covered in the installed suite. |
| Live integrations | 2/2 | Workspace login/probe/inference/local effect/resume/logout; 29 Keycloak/MCP checks, including two resource-expiry continuations and real scans. |
| Managed dependency distribution | 2/2 | Ordinary scoped-name npm12 fixture; four staged requests and no unexpected/public fetches; 70 packages, 3,091 installed files, 67 licenses; CLI5.2.0/SDK0.28.0 and actual corpus generation. |
| Upgrade safety and limitations | 1/2 | Actual published-alpha9 workspace/history relocation passes with old executables absent; managed-helper pin/tamper cases pass. OIDC migration, signed-package upgrades and a full downgrade matrix remain unproven. |

These results are sufficient to invite owner testing of these immutable Linux
bytes. The withheld point reflects the incomplete migration matrix and prevents
calling this a perfect delivery. To earn that point, add source-bound OIDC
upgrade fixtures and explicit supported/unsupported downgrade behavior against
the final distributed package; retain failure and interruption evidence rather
than inferring compatibility from unchanged account identifiers.

The new review-release contract at source `e9f6b1224` requires exact
binary/source/version matching, retained evidence hashes, an independent scoped
score of at least nine, explicit Linux checksum disposition, and
`full_authentication_release_ready: false`. Those checks support bounded P5
distribution; they do not themselves prove the content of an attestation or
authorize a production-complete label. A newly assembled package still needs
its own metadata/inventory validation and final registry-download checks. Keep
the reviewed native bytes unchanged during packaging.

The receipt [PRERELEASE-DISTRIBUTION-REVIEW.json](PRERELEASE-DISTRIBUTION-REVIEW.json)
binds this assessment to hashes of the existing evidence. It preserves these
remaining gates:

- Actual nonowner package-read authorization and the published package install.
- Affected-owner signed macOS authentication/Keychain retest; no Mac result is
  inferred from this Linux audit or the separate Mach-O byte comparison.
- Ordinary Linux desktop onboarding, the approved headless workflow, unassisted
  user trials and the plan's remaining selected-session requirements.
- Complete refresh concurrency/crash/negative-policy and migration coverage,
  followed by the frozen full-platform scorecard, P6 acceptance and P7 promotion.

The Linux fixture uses a disposable Secret Service, and its live OIDC run used
the exact native executable rather than the npm wrapper. MCP relocation uses
real saved-helper execution with remote transport disabled; simulated
revision-first state is not a physical crash test. These limitations remain
visible in the original evidence. Windows and container delivery retain their
owner-deferred scope and receive no inferred pass. This review makes no claim
that the owner's complete specification is finished.
