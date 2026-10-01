# October 1, 2026: approved selective adoption plan

Owner authorized implementation after the September 30 review. Qualify each
upstream patch against the fork before importing it; source readiness is separate
from packaged/native acceptance. The starting source is `f42eec46e1`.

1. Local sandbox boundaries and native macOS TLS trust access. First adapt
   nested metadata mount ordering, then qualify cross-root Git protection,
   explicit denial retention, `.aws` protection and macOS path aliases.
2. Command completion output and process-launch error reporting, including the
   actual agent/Jev execution path.
3. Markdown-preserving transcript copy and blank-session draft retention, with
   snapshots and real PTY checks for protected prompts and fullscreen behavior.
4. Opt-in steering, conditional on bounded cancellation, continuation and
   no-duplicate/no-replay acceptance.

Keep confidential-client MCP OAuth conditional on gateway support. Auto-review
requires a separate gateway-compatible design and credential/policy audit before
implementation. Existing Guardian/remote-executor deferrals still apply. TypeSafe
remains the optional bounded judge; deterministic code owns authorization and
execution. No model default or hosted service is imported incidentally.

| Candidate | Immutable upstream commit | Initial qualification |
| --- | --- | --- |
| Nested metadata mounts, #47623 | `8bbe8f8702471b90c75df20104bf91d20001b0d7` | Missing; adapt to existing synthetic mount registry, retaining writable-symlink rejection before deferral. |
| Cross-root Git protection, #47974 | `a92ccbde5328697894d09935a656f7c5f07a7d3f` | Pending source/dependency review. |
| Explicit denials after approval, #48155 | `645b683a9e712ae6d90a364e3cf597466e4146d0` | Pending; upstream spans nine files and requires splitting. |
| `.aws` protection, #48176 | `1d804e91b75454927fb4f51615b067ba6158c37c` | Pending policy/compatibility review. |
| macOS path aliases, #47879 | `6f51c65958ebdd5a57708e012d045f172fc9aab4` | Pending; upstream spans seventeen files, avoid incidental executor refactoring. |
| macOS TLS trust lookup, #48565 | `228ae3da8dda6edcff6a2078992e239729723a7d` | Missing; separate native macOS slice. |
| Codex 0.159.3 account reminders, #49744 | Not imported | Exclude: ChatGPT account setup is outside AIRS organizational gateway SSO. Audit future imports for account prompts. |

The first slice adds the upstream mount-order regression and four applied-policy
cases covering directory/symlink roots and parent/nested working directories.
The baseline `codex-sandboxing` suite passed 69 tests on Linux. The new regression
fails on original code and passes with the fix. Linux sandbox validation passed
111 unit cases, all four applied-policy regressions and 155 full-crate cases with
one explicit skip. Existing namespace-reaper checks printed four early-return
notices; the new cases ran fully. Both standard and strict scoped lint remain
blocked by the inherited static-header `expect_used` in rmcp-client. One earlier
redundant-closure lint in codex-client was corrected with the identical method
reference. Formatting passed; unrelated formatter rewrites were restored.
Evidence: `validation/2026-10-01/upstream-nested-metadata/ACCEPTANCE.json`.
No preview score, package publication or native acceptance is claimed. Follow-on
candidates remain planned, not implemented.

Use affected-crate `just test`, scoped lint and `just fmt`; preserve raw failures.
Before release, require exact candidate/fresh registry acceptance, gateway
denial/revocation, native credential access, actual-agent Jev approval and
upgrade/rollback. Linux ARM64 needs installed native-host evidence; Mac targets
Apple Silicon only with existing signing/notarization requirements. Retain owner
real-account acceptance as a separate gate.
