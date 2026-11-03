# Linux Desktop owned command lifetime

This change adds Linux equivalents of the existing Desktop command supervisor.
It applies after the standalone Desktop CLI starts supervision; ordinary Linux
pipe/PTY behavior and the existing Mac implementation remain unchanged.

Each owned pipe or raw-PTY root opens its own pidfd before exec. A private
`SOCK_SEQPACKET` socket transfers that capability to an independent supervisor,
which checks the root is a live child of its engine owner. An exact, monotonic
registration acknowledgement is required before user code can run. A separate
per-spawn socket transfers the same retained birth to the parent's command
handle. Every supervisor/registration descriptor is closed before user exec.
The wire carries only fixed-size control records and pidfds, never command
text, credentials, environment, project paths or conversation data.

Individual handles signal retained pidfds. Stop/revocation cancels all retained
roots before ordinary core interruption. Engine socket EOF or engine pidfd
exit makes the independent supervisor kill and wait for its roots. If the
supervisor dies while the engine lives, the existing health loop fails closed
and parent-held pidfds provide cleanup. A failed spawn or interrupted receipt
shuts down supervision before another command is admitted. Cleanup attempts
every root even if another descriptor reports an error. No late PID or PGID
lookup selects a signal target.

The root also arms parent-death SIGKILL before trusted helper setup. This is
additional interrupted-setup protection, **not** the complete descendant proof.
Strict Desktop admission must require the verified nonprivileged bundled
Bubblewrap chain, `--die-with-parent`, user/PID/network namespaces and the final
command no-fanout filter. The trusted helpers may fork during setup; user code
may use threads and exec replacement but cannot create another process or
session. Killing the supervised root must retire that complete namespace.
Missing pidfd/backend support is unavailable, never a generic fallback.

The standalone Desktop Linux sandbox alias is dispatched before account-home
initialization only when its internal flag precedes the command separator.
The typed filesystem helper requires its exact additional owned-helper marker
and three-argument shape. Ordinary helper invocations retain their startup.

## Bounded verification

The initial exact-source `codex-utils-pty` mirror ran on Enterprise's actual
Linux7.2.8-9-omarchy AMD64 host with Rust1.99.1 and nextest0.9.148:

- Original crate run: **34 passed**, two private subprocess entry points skipped.
- Follow-up Desktop-only run: **7 passed**; unchanged crate evidence was retained.
- Exact Linux early-dispatch selector: **2 passed**, including the owned typed
  filesystem-helper route. The complete CLI awaits the pinned engine compile.
- Scoped Clippy fix used the original engine workspace lint policy/config:
  passed with no source changes. No test repetition followed that no-op fix.

The pipe and owned raw-PTY matrix covers handle termination, repeated cancel,
guard drop, supervisor SIGKILL and engine SIGKILL. The disposable root clears
PDEATHSIG, ignores TERM, exec-replaces and starts a thread, so independent pidfd
cleanup is required. It asserts no supervisor socket survived exec, no delayed
side effect occurs after stop, and a separate control child stays alive. Other
focused checks cover wrong acknowledgement/no user exec, a stale retired birth
handle, every-root cleanup after one descriptor error, and interrupted setup in
both transports. The first attempt's missing-nextest PATH failure is preserved.

`probe-receipt.json` binds the copied sources, generated probe lock, toolchain,
commands and original raw receipt/log hashes. The isolated mirror pins direct
dependencies from the engine lock but resolves its own small transitive lock;
it is explicitly **not** the full engine workspace or a Rust1.95 build.

## Remaining integration evidence

The actual full owned pipe/PTY → verified bundled Bubblewrap → inner helper →
final exec chain still needs its focused fixture execution, including abrupt
engine death and interrupted setup. Then the pinned Rust1.95/Ubuntu22.04 engine
build, strict filesystem/network/descriptor probes and Desktop bootstrap and
revocation regressions must pass before a Linux engine artifact/pin is proposed.
These crate checks do not qualify a release or replace those boundaries. The
accepted Mac engine archive and application artifact are untouched.

Primary syscall references: [pidfd_open](https://man7.org/linux/man-pages/man2/pidfd_open.2.html),
[pidfd_send_signal](https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html),
[Unix descriptor passing](https://man7.org/linux/man-pages/man7/unix.7.html), and
[parent-death signal caveats](https://man7.org/linux/man-pages/man2/PR_SET_PDEATHSIG.2const.html).
