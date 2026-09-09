# Independent alpha.10 Linux checkpoint review

Date: 2026-09-09. Reviewer: `/root/guided_login`.

**Recorded Linux candidate checkpoint: PASS. Full release: NOT READY. Formal
weighted delivery score: NOT ESTABLISHED; no >=9/10 award.** The frozen,
populated platform scorecard and remaining mandatory gates are not established
by these successful subsets. A numerical impression must not substitute for the
plan's scoring method.

## Identity and evidence verified

Runtime source is `b012ba55e1404b498ad513cd15efa52a9a60530f`, version
`0.1.0-alpha.10`, target `x86_64-unknown-linux-musl`. The reviewer independently
rehashed both the retained executable and the native executable inside the fresh
npm installation to:

`a9166568623ce602da606a668359003e1f0a1380a202790da77f0a83dd1dc6ff`

The installed bundle inventory independently hashes to
`04c0a59076b241397e2a1b760a1cdcf3e01c0ea43af5cb29c9d702dff26bd743`, matching
[the installed receipt](installed.json). All 17 recorded packaging source-file
hashes match their files at source `b012ba55`. All six recorded Rust source-file
hashes in [the scoped unit receipt](../helper-relocation-unit/final-result.json)
also match that commit. The retained decompressed JUnit XML independently matches
its recorded hash and reports 388 tests, zero failures, errors or skips.

| Evidence | Verified result | Scope |
| --- | --- | --- |
| Scoped Rust JUnit | 388 passed | CLI library/binary and home-directory crates |
| Native executable log | 38 passed, one skip / 39 methods | The skip requires the npm-managed CLI |
| Installed executable log | 38 passed / 38 methods | Includes the managed CLI case skipped above |
| Managed-helper relocation | 4 passed | Exact saved helper execution, controlled path migration, pin/tamper checks |
| Published alpha.9 upgrade | Nine checks passed | Synthetic workspace credential, actual history resume, absent old executables |
| Live workspace authentication | 11 checks passed | One native-store login/probe/inference/resume/logout lifecycle |
| Live Keycloak/MCP | 29 checks passed | Browser/device flows, separate resource identities, local tools, real scans, two expiry cycles |
| Scoped bundled install | Passed with npm 12.0.2 / Node 22.23.2 | Four staged requests; zero unexpected requests or public dependency redirects |

The install verified 70 exact-version packages, 3,091 retained installed files and
67 licenses. CLI 5.2.0 and SDK 0.28.0 remain pinned. Actual PDF/PNG/JPEG/SVG/DOCX
generation exercises the managed CLI and its native image dependency. Its Linux
payload includes glibc and musl variants; this run exercises the recorded Linux
host, not every packaged libc variant independently.

The [live OIDC receipt](live-oidc.json) identifies the exact native invocation and
records successful local commands and distinct scanner results after two
inference-and-MCP expiry boundaries in one terminal process. Its null source field
is resolved through the separately verified [build provenance](BUILD.json), not
rewritten or treated as an embedded source attestation. [Upgrade evidence](upgrade.json)
and [workspace evidence](live-workspace.json) preserve their narrower limitations.
The earlier failed relocation baseline remains in [its own record](../helper-relocation-before/README.md).

## Findings and limits

The only concrete reporting error found was the native-suite skip being called
macOS-only. Its actual reason is `requires npm-managed CLI`; the installed suite
covers it. Root corrected the report. No additional executable defect emerged
from this bounded audit. No tests were rerun; this review inspected existing
receipts/logs, fixture behavior and actual retained bytes.

These results do not establish the entire authentication plan:

- The workspace live fixture runs one lifecycle and does not test MCP or
  live-process logout invalidation. Live default-route success alone does not
  prove model-key omission; captured loopback requests establish that behavior.
- Live OIDC used the native executable, not the npm wrapper. Two-expiry continuity
  is proven for this fixture; concurrent/crash-safe refresh, exact server rotation
  counts and the complete persisted audit/negative-policy matrix are not.
- The published-alpha.9 upgrade covers Linux workspace credentials, not OIDC
  migration or signed published-package upgrades. Relocation reuses candidate
  bytes. An old client failing because its configured helper is absent does not
  establish downgrade compatibility or a version-enforcement policy.
- Managed MCP relocation executes the actual saved helper with synthetic file
  credentials while remote MCP transport is disabled. Revision-first fault-state
  replay is not an injected process crash or power-loss test. The reviewer authored
  this four-method fixture; root separately reviewed it, so these tests are not
  an independent review of all self-authored work.
- A disposable Secret Service fixture is not ordinary GNOME/KDE onboarding or a
  completed harness-owned headless persistence flow. Repeatability, native desktop
  consent, unassisted user trials and the remaining selected-session gates remain.
- No package was published. Actual authenticated teammate installation, final
  Developer ID signing/notarization, affected-owner Mac acceptance and applicable
  migration tests still require evidence from the final distributed bytes.

Native Windows is explicitly owner-deferred in the current plan and is not a
blocking platform for this Mac/Linux release. It receives no inferred pass;
container delivery likewise requires its own acceptance before being advertised.
Pending Apple Silicon builds or tests receive no credit in this Linux audit.

The current outcome is a substantively verified Linux candidate with repaired
helper relocation and working live authentication integrations. Release approval
must wait for the remaining in-scope gates and a defensible scorecard; this review
does not declare the project's full specification complete.
