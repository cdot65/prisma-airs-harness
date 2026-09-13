# Codex 0.154 integration ledger

Candidate branch: `integrate/codex-0.154-candidate`. Original main is unchanged.
AIRS source: `1cdc152631d4c1e960435f1630373fef29d34a64`.
Stable import: `6b9826e3aa83b1a5947db50f4332cb9c65f1b340` (`rust-v0.154.0`).
Independent backports: `77abdbcfe` (macOS TIOCSTI), `5b4d18df0` (hook stdin timeout).

## Semantic resolutions

| Surface | Resolution | Required proof |
| --- | --- | --- |
| CLI startup | Retain AIRS home, origin, setup, login, environment and prohibited-command handlers; add upstream build initialization, allocator and worktree validation; remove obsolete McpServer match variant | Native CLI/PTY, authority tests, worktrees |
| Request client | Retain optional gateway model and supported-effort filter; apply upstream model effort resolution after filtering; retain serial tools and redirect refusal | Request assertions and live default/explicit/default, compaction |
| TUI history/status | Retain AIRS name/version/environment and hidden unsupported reasoning; retain upstream model display name/provider identification | TUI snapshots and native PTY |
| Session/token budget/guardian tests and scorer | Use stable coherent implementation for inherited release-branch hotfix conflicts, with no AIRS-specific baseline changes in those conflict blocks | Affected Rust suites |
| Models catalog | Stable upstream catalog; AIRS private gateway catalog remains selected | Provider/catalog isolation tests |
| App-server README/schema | Preserve credentialHelper API documentation; regenerate stable and experimental fixtures from merged protocol | Protocol schema suite |
| Cargo | Workspace 0.154.0 with AIRS member/dependencies; retain quinn-proto 0.11.15, event-listener 5.4.2, memmap2 0.9.11, spin 0.9.9 patches | Locked build, Bazel refresh and drift check |
| Bazel defs | Stable build-info env_files integration | Native GNU lock refresh; platform builds |

## Additional AIRS adaptation

Runtime-managed features pin unsupported apps/plugins/image generation/remote control/discovery/Code Mode/realtime off, including existing homes and later feature mutations. Realtime has a direct provider boundary rejection before endpoint/auth/network initialization. Native override regression and core no-request regression cover these distinct surfaces. Worktrees and inline questions remain in the selected acceptance scope.

AIRS product candidate version is 0.1.0-alpha.12; Prisma CLI 5.2.0 and SDK 0.28.0 remain pinned. No publication is part of this change.

## Evidence status

This ledger records implementation decisions, not completed independent review. Logs are candidate-specific except explicitly named alpha.11 control logs. Hosted dependency refresh and platform validation are pending. The local Alpine host cannot execute rules_rs GNU cargo; this must be resolved with a real GNU-host refresh, not a manually invented Bazel lock.

All eight mandatory gates, live service correlations, exact installed artifacts, independent review and owner acceptance must be evaluated before any 9/10 claim. Skips and unavailable tests receive no behavior credit.
