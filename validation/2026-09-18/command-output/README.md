# AIRS command-output consistency

The authorized full Rust workspace run completed on Alpine x86_64/musl:
**18,097 passed, 150 failed, 34 skipped**, across 273 test binaries.
The workspace is **not green**. After receiving these results, the owner explicitly
requested publication of the command-output update. Native release acceptance is
tracked separately in `../command-output-release/README.md`.

All eight changed packages passed in that run, including 933 CLI, 4,336 TUI,
and 4,159 core tests. The exact reported resume command, environment retention,
shell escaping, colored output, recursive help tree and recovery expectations
passed. Fourteen checks against the built native binary also passed: twelve help
commands and two worktree recovery messages. `just fix` for the eight affected
packages, `just fmt`, and `git diff --check` passed.

## Change

Generated resume, credential recovery, MCP login, worktree errors and help examples
use `airs` for the standalone application. Resume commands retain the selected
environment. Package names, engine identities, credential locations and
user-supplied session names are preserved. Eleven help/worktree snapshots were
reviewed and updated; two new exit-summary tests cover the reported command and
shell escaping.

## Full-suite failures

| Package | Failed | Observation |
| --- | ---: | --- |
| exec-server | 146 | The test executable rejects `--listen` instead of dispatching to its embedded server. A direct executable probe reproduces this without the Cargo runner. |
| linux-sandbox | 1 | The test denies `/tmp`, where this run's sandbox helper resides; the helper cannot be executed inside that policy. |
| skills-extension | 1 | An ancestor `/tmp/.git` marker is present, so a test expecting no project ancestry finds an extra skills root. |
| app-server | 2 | Skills-budget warning reports 15 omitted skills versus 7 expected; the zsh-fork integration case times out. |

These failures are in unchanged packages. They have **not** been proven to be
baseline failures by running the previous commit. Do not claim a green workspace. The owner authorized publication after these
failures were disclosed.

[RESULTS.json](RESULTS.json) records package counts and commands;
[workspace-failures.json](workspace-failures.json) names all 150 failures.
Compressed raw logs and the direct exec-server probe are retained alongside this
note.

## Test setup and earlier attempts

The final command was `just test --locked --features codex-v8-poc/sandbox`, using
the V8 setup and feature selection from `scripts/airs_forgejo_linux.sh`. The
repository's `scripts/codex_package/v8.py` resolved sandbox-enabled V8 artifacts
and checked their hashes against the pinned manifest. The CI certificate wrapper
was adapted to the musl runner variable so test processes do not inherit ambient
certificate overrides. No application security or TLS behavior was changed.
[V8-INPUTS.json](V8-INPUTS.json) records the artifact hashes.

The first unconfigured `just test` failed at the generic V8 download URL before
executing tests. This was resolved with the repository's existing artifact setup.
The earlier affected-package run passed 9,980 cases and failed 103 code-mode
cases; **all 103 pass in the final full workspace run**. Those intermediate logs
and failure names are retained for traceability, not presented as final results.

Uncompressed logs and the reused build cache are in
`/tmp/airs-command-output-20260918`. Jadzia was inspected as a fallback; no remote
suite or service was started, and its temporary nextest download was removed.
At completion of the full-suite run, the published version was
`0.1.0-alpha.22.onboarding.2`; subsequent release work is recorded separately.
