# Terminal.app over SSH — completed source integration

| Criterion | Score | Evidence |
| --- | --- | --- |
| Implementation | 9/10 | Complete DA1/DA2 matching, bounded unknown fallback, preserved typeahead/paste, and real TUI/CLI screen-policy checks on Linux and Apple Silicon. |
| Code quality | 9/10 | Small private parser, existing startup/replay ownership, generated Cargo/Bazel locks, zero-warning scoped lint on both hosts, and 7,182 matching Rust source files. |
| Design | 9/10 | Detection applies only to SSH without a detected multiplexer. Explicit screen overrides win. Compatibility applies before startup dialogs/pickers and after configuration reload. |
| Feature completeness | 9/10 | Full local/native TUI suites, both dependency input implementations, seven PTY scenarios per host, actual CLI resume picker checks and built debug binaries. Signed release acceptance remains part of the broader delivery phase. |

These are bounded, evidence-based self-review scores, not independent certification or completion of the broader PRD.

## Behavior and scope

Adapted upstream `2925d06f5a91703548575e6b52f17591e8af85b2`. On Unix SSH sessions without a detected multiplexer, a complete DA1 `1;2` plus DA2 `1;95;0` pair identifies Terminal.app. Missing replies remain unknown; pasted signatures cannot select screen policy. Automatic mode preserves native scrollback for Terminal.app. Always, never and `--no-alt-screen` retain their explicit precedence.

AIRS's initial composer already renders inline. Its probe result now survives terminal handoff and applies to startup dialogs, the resume/fork picker and final configuration. The change adds no provider, credential, OAuth, gateway routing or fullscreen-default behavior. Existing startup and overlay snapshots pass unchanged.

The pinned crossterm change (`45fecb9..efa1778`) consumes secondary replies in both Unix input sources. Its regression tests exercise every byte split and preserve reply-like text inside bracketed paste. Real Cargo regeneration also reconciled fifteen Windows dependency edges without changing package versions; the generated resolution is retained. Real native Bazel lock update and strict lock check passed.

## Verified checks

- Local TUI: **4,456/4,456** passed, six skips, retries disabled.
- Apple Silicon TUI: **4,461/4,461** passed, six skips, retries disabled; 324.732 seconds of test execution.
- The six skips are four existing manual tmux resize cases, the PTY child invoked by its parent test, and a Windows-only AltGr case. Real tmux resize/fullscreen ownership belongs to its later accepted slice.
- Seven new real-PTY scenarios pass on each host: Apple auto/always/CLI override, non-Apple auto, missing secondary reply, local query exclusion, and tmux query exclusion.
- Fresh `codex` and `airs-harness` debug CLI binaries built on both hosts. Actual `resume` picker checks pass four mode/override combinations on each host.
- Dependency tests: 2/2 for mio and 2/2 for tty on Linux. The initial invocation selected this repository's absent `local` profile in the dependency checkout; its `default` profile runs the same tests successfully.
- Local/native scoped lint: zero warnings. Formatting passed. Post-lint source audit: all 7,182 tracked Rust files match.
- Local coverage includes 72 AIRS-named cases, seven TypeSafe cases, eight routing cases, 28 terminal-probe cases and 164 startup cases; these groups overlap.

## Adversarial findings and retained failures

1. Initial adaptation applied compatibility only after session selection. Review moved initial policy application before startup dialogs and the picker, with a refresh after onboarding reload. The CLI checks exercise that early consumer.
2. The initial seven PTY cases sent action keys to the provisional composer, which intentionally ignores them. Waiting for the loaded model header fixes the fixture; every original failure is retained.
3. The first native suite filled disk at 664/4,461, leaving 3,797 cases unrun. It is not a validation pass. Cleanup preserved linked binaries/libraries and diagnostics. A scheduling mistake started a rebuild before cleanup finished, producing a missing-intermediate-object failure before tests. Both attempts are retained. The final complete run uses per-test temporary roots, cleans successful cases immediately and retains failures; disk remained stable. No Rust process was killed.
4. The native CLI fixture used `/var` while the runtime correctly canonicalized it to `/private/var`, so trust did not match. Canonicalizing the disposable fixture root closes all four cases on the same binaries. The real trust prompt was not bypassed.

## Remaining delivery boundary

This feature clears its four 9/10 source-integration gates. No new signed/notarized or npm preview is claimed. Actual Terminal.app-over-SSH behavior is represented by controlled protocol replies; live owner acceptance and the remaining Unicode, math, fullscreen/search, tmux ownership and signed Mac release gates remain in the broader PRD.
