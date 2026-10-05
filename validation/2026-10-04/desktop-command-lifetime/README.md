# Desktop command lifetime correction

This correction follows the exact `b8d1e9e0b03418468d9060d32e255c3af7fd4460`
candidate. Independent macOS execution proved that killing the engine process
group could leave an approved command in a different process group alive. A
subsequent rapid double-fork probe also proved that descendant polling alone
cannot retain a child that reparents before an ancestry snapshot.

The operator explicitly chose approved commands that cannot spawn child
processes, with pipelines and worker-spawning build/test runners unavailable.
Desktop therefore adds `deny process-fork` to its existing dedicated Seatbelt
policy. Single-process execution and `exec` replacement remain available.
Ordinary Harness process policy and command behavior remain unchanged.

## Reviewable owner stages

The utility stage provides one opt-in macOS `DesktopSupervisor` API. Its private
modules own process birth/audit-token checks, fixed private pipe registration,
independent helper supervision, an engine backup monitor, and fresh cancellation
acknowledgements. Both pipe and raw PTY spawn boundaries register before user
code executes. The smallest callable API requires these cooperating parts;
splitting identity or private protocol support into unused intermediate APIs
would obscure the admission invariant. Production modules remain below 500
lines. The CLI stage selects the API only for Desktop, integrates interruption
and retirement, adds the dedicated sandbox restriction, and documents the scope.

Major behaviors verified at the actual command boundary:

- A single external `exec` succeeds through native approval with pipes and PTYs.
- `fork`, `vfork`, ordinary `posix_spawn`, the first double fork, and a worker
  runner return literal `EPERM`, with no child execution marker or delayed write.
- An actual shell pipeline returns a nonzero process result and a denied-fork
  error, with no worker execution. The fixture checks the actual tool result;
  this error path does not emit a separate command item with numeric exit code.
- Engine group `SIGKILL`, supervisor `SIGKILL`, native stdin EOF, and turn
  interruption stop an already-admitted single process with pipes and PTYs.
- Helper loss retires the engine even while native stdout is not drained and
  the host's stdin writer remains open.
- A sandboxed approved command cannot signal the helper or an unrelated process.
- Cancellation uses fresh correlated acknowledgements after cleanup, permits
  later admitted commands, and rejects stale/malformed control records.
- Darwin process birth checks prevent PID reuse from changing cleanup targets;
  audit tokens are refreshed across `exec` and checked by the signal syscall.
- The 33 KiB native tool-reply fixture remains below the outer 64 KiB frame cap
  and proves the separate 32 KiB tool-output admission limit.

## Local checks

All checks used the owned target `/tmp/airs-harness-engine-target-20261004`, Rust
1.95, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, and
`CARGO_INCREMENTAL=0`. Offline HTTP fixtures use synthetic credentials only.

- `just test -p codex-sandboxing -p codex-utils-pty --test-threads 4`: 157 passed,
  10 platform skips; `/tmp/airs-desktop-engine-sandbox-supervisor-full-20261004.log`.
- `just test -p codex-cli --test-threads 4`: 1102 passed, no skips;
  `/tmp/airs-desktop-engine-cli-restricted-full-20261004.log`.
- The unchanged Worktree test failed before preserving the existing boxed CLI
  dispatcher in the new branch, then passed alone and in the full CLI suite.
  The async-frame explanation remains a source-backed inference until an
  independent mutation establishes causality. Earlier failure logs are retained.
- Scoped `just fix` for CLI, sandboxing and PTY, repository `just fmt`, and
  `just bazel-lock-update` complete the source gates. The dependency owner changed
  an existing PTY edge from development to production; the Bazel lock was unchanged.

The earlier `d7e04502` exploratory binary and its 8 passing/1 failing lifetime
probes remain attributed to that pre-restriction binary. Its double-fork failure
is not a passing final acceptance result. Exact source/binary/package hashes and
independent round-two evidence are recorded separately. These local source and
offline checks do not establish native UI or live Gateway acceptance, production
signing, notarization, registry publication, or stable promotion.
