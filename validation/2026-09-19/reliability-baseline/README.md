# Focus 1 evidence: published baseline and reliability inventory

Source checkout: `a826d22a63d9d6889157477a0109e242871796f5`; exact published Linux native runtime is mcp.2 source `19e5bcee52f6b21e22441a870db0a610fa8b3d4b`, SHA-256 `f40aa35dac2834d2c32edd261e799f6414e0227eb002c00fbdd5d26b3b7aab81`. See BASELINE.json for commands, source/host identity, and log hashes. These are isolated fixtures, not the owner's session.

## Observations and next actions

| Observation | Evidence class | Evidence | Decision |
|---|---|---|---|
| Explicit staging login with saved default work displays unscoped doctor/restart commands after rejected probe | Reproduced on published binary | onboarding-baseline/RESULT.json; reproduce.py; terminal screens | Focus 3: scoped, copyable recovery commands |
| Kernel credential syscalls denied while Secret Service works does not break doctor | Disproven failure hypothesis; positive executed fixture | doctor-backend-reproduction.json; reproduce-doctor-backend.py | Preserve existing read-only bounded probe; retain regression coverage |
| Secondary credential deletion failure replaces original save/read failure | Source-confirmed; targeted failing regression pending | auth-baseline/primary-cleanup-masking.md | Focus 2: preserve safe contexts and typed recovery |
| Generic native-service error says to unlock even though lock state is unknown | Existing reported behavior and source-confirmed | login/src/auth/credential_recovery.rs; auth plan | Focus 2: neutral session/service guidance |
| npm launcher lacks unsupported-Node preflight | Source-confirmed; owner reported Node 18 engine warning | npm/airs-harness/lib/launcher.js; onboarding plan | Focus 3: engine-range guard before child dispatch |
| Old installed-candidate resume skips stages without receipts; release asserts disabled under Python optimization | Source-confirmed tooling gaps | delivery plan; previous scratch release scripts | Focus 4: explicit validated checkpoints and deterministic guards |

## Fresh checks

Published mcp.2 doctor: four passed, no failures/skips. Published mcp.2 MCP manager: four passed, no failures/skips. Full installed regression fixtures: 46 passed, one macOS-only Seatbelt skip, no failures. Existing 54 authentication fixture references are inventoried in auth-baseline/coverage.json and are **not** represented as newly executed Rust tests.

## Historical full workspace

The September 18 receipt is 18,097 passed, 150 failed, 34 skipped. Current representative reruns are in workspace-triage. A source file being unchanged is not proof a failure was preexisting. Keep each triage finding classified as reproduced, environment demonstrated, historical-only or unresolved. No full-workspace green result is claimed.

## Explicit limits

The owner deferred the Ubuntu credential-session investigation and attended production SSO/ServiceNow acceptance until tomorrow. No real identity, key, callback, owner credential session or gateway configuration was used or changed. Fixture results cannot authorize stable/default tag promotion. Native installed platform checks remain required before the later npm test release.

## Sequential gate

This baseline is assessed with completeness3 + capability3 + best practices2 + optimization2. No score is assigned until independent review; meaningful new defect regressions must demonstrate failure before runtime fixes. Runtime implementation begins only after baseline review reaches at least9/10 with mandatory isolation/provenance/truthfulness gates met.
