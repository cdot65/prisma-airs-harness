# AIRS selective Codex adoption

Implementation ledger for the September 22 PRD. Decisions are based on behavior,
not ancestry counts. Baseline: `1a6df30b5654ca8b892440b21e4bf1b5442b1f38`
(AIRS 0.1.2 / bundled CLI 7.1.5). Reviewed upstream:
`a1f40f3f1326eff7c81b860a4f4e27c1be186e10`. The prior Codex 0.154 import is
`719b317241`; some later features were independently adapted into AIRS.

## Candidate ledger

| Upstream commit | Decision | AIRS implementation and validation boundary |
| --- | --- | --- |
| `064e701b0fa4f5c0dcaf195a565d088f99bb6af8` | Adapt lifecycle semantics; defer RPC | `airs_status`, `airs_access`, `airs_auth_lifecycle` and TUI `airs_recovery` already share CLI-owned operations. Upstream secondary gateway OAuth would change AIRS SSO-or-key semantics. Audit status and cancellation using existing private adapters; add no protocol or credential service unless a concrete gap requires it. |
| `f8ab57359dde6b6d5de1aee613c18fe60b661aeb` | Already present, audit later gaps | `mcp_login`, `oauth_callback_input` and their tests already implement hidden bounded callbacks. AIRS extends this with private TUI progress; retain gateway resource binding. |
| `b33199b1fbcf96ea1e4035729158f6ecd1760192` | Adapt | AIRS `airs_doctor` is a separate entry point from upstream `doctor`. Bound filesystem probes in the AIRS path rather than importing unrelated desktop/daemon checks. |
| `0ad18769ad8dfb7a6cf16b1bdd26b6e5f8773f64` | Existing AIRS redaction; audit remaining edges | `airs_status::configuration` already discards TOML error payloads and `airs_doctor` emits a fixed configuration failure. Canary tests must exercise AIRS, not only upstream doctor. |
| `ef9f3d022aff6bd087c640392dc8608743aa411d` | Adapt if gap reproduced | Inspect AIRS recovery dispatch when the displayed thread is unavailable; preserve drafts and never replay a failed turn. TUI tests and PTY acceptance own validation. |
| `9ed1b7469921e056b7cdbe2b98d1b3251d08e9f9` | Compare existing startup draft handling | `startup_draft` already exists. Adopt only missing exactly-once behavior, not upstream fullscreen defaults. |
| `c7828dd0109c75bcb5b07dc2d95c68947e56239a` | Assess narrow correctness delta | Requires changes across StepInputs, Guardian and execution environment ownership. Executor contexts are not AIRS environment UUIDs. Do not import the 40+ file refactor without a demonstrated active-product gap. |
| `568518357023708ecbeddb764ef8956ecaa076d7` | Defer daemon portion; assess active permission path | Depends on captured executor state. AIRS does not enable managed daemon recovery. Check existing origin-context behavior separately. |
| `2b842962883f2e01526b9c64e383cb2375123a5d` | Defer OpenAI refresh; verify AIRS isolation | Upstream refresh is OpenAI-specific and explicitly a no-op for static catalogs. AIRS uses its local capability catalog; its gateway remains the authority. Credential generation changes invalidate the active session rather than permit adopting another identity. |
| `d5b29951aca5205bc8415120b56dd4bc295f5c83` | Adapt only missing classification | AIRS already distinguishes 401, 403, 446 and blocked-200 probes. Test transport/runtime paths for policy denial and unsafe replay before changing retry architecture. |
| `3b50349e1a80930e6e9dcf6d9377d65ef55ef052` | Defer | Hosted app/account resource targets are not needed for gateway ServiceNow. No schema import. |

No dependency updates are currently selected. Each accepted patch carries only
its required code/tests; generated schemas and lockfiles follow owning changes.
Gateway-facing MCP OAuth uses upstream client machinery while the gateway retains
upstream OAuth credentials. CIE/CAS semantics, issuer/resource binding and workspace
keys remain AIRS-owned. TypeSafe remains an independent optional judge credential.

## Baseline and scoring

Baseline suite in progress: `just test -p codex-cli -E 'test(airs_)'` against
unmodified 0.1.2, using the existing local musl cache. Installed native acceptance
from `validation/2026-09-21/stable-0.1.2` is historical baseline evidence, not proof
that new code passes. Ubuntu acceptance host has only 431 MiB free; preserve it.

Each feature score uses correctness (3), boundary preservation (3), regression
evidence (2), maintainability (1) and user clarity (1). Below 9 requires revision;
a hard-fail condition overrides the arithmetic. A source/test score is not a
cross-platform release score. Pending evidence cannot receive passing points.

P0 and feature scores remain pending until their respective validation completes.
