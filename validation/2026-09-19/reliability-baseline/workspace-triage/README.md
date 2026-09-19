# Historical workspace failure triage — September 19

The historical full suite remains **18,097 passed, 150 failed, 34 skipped**. This triage did not rerun the full workspace or a previous source commit and does not prove all failures are baseline. It establishes narrower reproducible causes using retained historical binaries and one current focused rebuild.

## Results

| Historical group | New evidence | Conclusion |
| --- | --- | --- |
| exec-server: 146 failures | Retained process executable still rejects `exec-server --listen invalid`; current rebuilt health test also fails before embedded server starts. A minimal `.init_array` reproducer reports `std::env::args() == []` before main and correct arguments in main on rustc 1.95 x86_64-musl. | The test helper's `#[ctor]` dispatch reads arguments before musl Rust initializes them. Strong demonstrated mechanism for the shared failure; not all 146 individually rerun. |
| skills-extension: 1 | Exact retained executable fails with TMPDIR=/tmp and passes with TMPDIR=/home/cdot/.cache/airs-workspace-triage-tmp. | Demonstrated environment-sensitive test contamination from ancestor /tmp/.git. Marker inspected and untouched. |
| linux-sandbox: 1 | Retained denied-/tmp test still fails with isolated TMPDIR; error identifies sandbox helper under /tmp as permission denied. | Runtime TMPDIR cannot relocate the compile-time `env!(CARGO_BIN_EXE_codex-linux-sandbox)` path. Test needs build/helper layout outside denied /tmp to validate intended condition. Passing relocated rerun not performed. |
| app-server skills warning: 1 | Exact retained executable still reports 15 omitted vs expected 7 with isolated TMPDIR and isolated home built into fixture. Source embeds 14 system skills, including 8 AIRS skills; fixture adds 2 skills. | Source strongly supports stale upstream hardcoded budget count after 8 added AIRS skills. Do not reduce product skills to satisfy a stale count. Test isolation/semantic expectation is the appropriate future fix. No source change made. |
| app-server zsh: 1 | First rerun self-skipped because dotslash absent from PATH and is not a pass. With dotslash PATH corrected, retained test reproduces process creation ENOENT then timeout. Selected vendored executable exists but requires /lib64/ld-linux-x86-64.so.2, which is absent on this musl host; direct --version raises ENOENT. | Demonstrated incompatible vendored interpreter on this runner, rather than an unexplained timing-only failure. No glibc installation or fixture replacement attempted. |

## Current bounded Rust check

Ran from `airs-reliability-20260919/codex-rs`:

```
just test --locked -p codex-exec-server --test health \
  -E 'test(exec_server_serves_readyz_alongside_websocket_endpoint)'
```

Used the existing `/tmp/airs-command-output-20260918/target`, debug info disabled, incremental disabled, four jobs, isolated TMPDIR, and the existing test-only certificate-scrubbing runner. Build took 79 seconds. Result: **0 passed, 1 failed, 2 filtered**, exit 100. The failure reproduces `Unrecognized option: listen` both attempts. Rust cache was released before the sibling agent's CLI regression run.

## Recommended next handling

1. Keep the workspace non-green disclosure. Do not block owner-authorized test release on a fabricated full-suite success.
2. Run future tests with an isolated TMPDIR whose ancestors have no project markers. Never delete unknown `/tmp/.git`.
3. Triage exec-server constructors as a separate test-portability change: initialize dispatch after Rust argv initialization, or use a dedicated helper main/test harness supported by musl. Do not weaken product execution policy or add a direct-upstream workaround.
4. Build the sandbox fixture/helper outside the denied `/tmp` tree when exercising this test. A symlink is insufficient because the helper is canonicalized.
5. Isolate the app-server skills-budget fixture from bundled skill count or derive its expected count from actual fixture inventory while still testing budget semantics.
6. Run patched zsh integration on its supported GNU host or provide a verified actual musl artifact; do not classify a self-skip as successful execution.

`SUMMARY.json` and per-probe JSON include exact retained binary hashes and results. Logs contain synthetic test traffic only. No application code, credential state, `/tmp/.git`, security environment flags, or external services were modified by this triage agent. The source file modification visible at completion belongs to the sibling credential regression effort and was preserved.
