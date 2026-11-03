# Bounded Linux Desktop integration

Base: `9331a5839bc6a763d94dee7584746f58bbbe1181`, protocol `airs-desktop/1`,
runtime `0.1.7-desktop.2`. The original checkout and accepted Mac pin/artifact
remain unchanged. The initial unregistered process prototype now has
experimental Desktop-only wiring in this branch. It is not a qualified Linux
engine artifact and no desktop app pin selects it. Source toolchain is Rust1.95.0.

## Existing authority contract

The Desktop bootstrap owns tenant/workspace/credential UUIDs and epoch, pins
Gateway inference/MCP credentials in memory, disables saved identity discovery,
hooks/skills/remote executors, and keeps AIRS operations proposal-only. Local
commands get one explicit project scope (read-only or workspace-write), no
network and no account/private runtime state. The accepted Mac contract permits
one command process, threads and exec replacement; forks/pipelines/daemon
creation are unavailable. Cancellation/revocation/engine exit must retire owned
commands without touching operator shells or other native leases.

## Integration boundaries and actual evidence

| Boundary | Existing 933 baseline | Required Linux Desktop change / proof |
| --- | --- | --- |
| Bootstrap selector | `cli/src/airs_desktop_runtime.rs` calls Mac-only Seatbelt selector unconditionally | OS-specific strict Desktop selector; Linux must refuse bootstrap unless all backend prerequisites are present. Never replace with no-op or generic alpha engine. |
| Filesystem | Root-deny explicit Desktop permission profile; loader read list is Mac-specific | Linux loader/system-file read list only, same explicit project grant. No global temp writes, account home, runtime home or credential-directory grants. Verify private/symlink/descriptor denial and read-only/write modes. |
| Network / IPC | Existing Linux restricted seccomp denies connect/bind, including Unix sockets; Bubblewrap can isolate network | Preserve both layers; deny D-Bus/Secret Service, Wayland/X11 and abstract sockets. No broad `/run/user`, `/tmp`, display, proxy or host-socket exemption. Test descriptor closure too. |
| Command fanout | Ordinary Linux helper forks an inner command and ordinary filters permit further forks | Carry an internal Desktop-only marker through outer/inner helper; install prototype only in final command child. Never restrict helper setup or ordinary Harness jobs. Refuse legacy Landlock/full-disk/proxy bypass for Desktop. |
| Fanout proof | New `desktop_process.rs` compiled directly by isolated probe | Enterprise hostRust1.99.1: ordinary control passes; unguarded Desktop assertion fails; guarded fork/vfork/clone, session/group and namespace escapes denied; pthread works; guard survives exec. **Not** a full engine build. |
| Pipe lifetime | `utils/pty/src/pipe.rs` sets parent-death SIGTERM before exec; Linux helper forwards signals and uses Bubblewrap PID namespaces | Verify the complete trusted-helper chain, abrupt engine death, stop and revocation. SIGTERM/parent-death alone is insufficient against ignored signals or credential-changing exec. |
| PTY lifetime | Mac has Desktop supervisor; Linux raw PTY `pre_exec` does not set parent-death signal, and Desktop currently does not force the owned raw path | Linux Desktop must use an owned, auditable PTY spawn path with the same lifetime guarantee as pipes. No ordinary operator shell/PTY policy changes. Actual PTY + Bubblewrap + exec/exit tests required before registration. |
| Process identity | Mac dual supervisor uses audit identities; generic Linux uses process groups | Use owned child/pidfd identities or equivalent kernel-backed lifetime, not a late PID lookup that could target a reused/operator PID. Namespace death must cover all final-command threads and exec replacements. |

The current no-fanout guard adds no filesystem/network permission. It rejects
non-AMD64/x32 ABI escape, denies process clones and session/namespace changes,
and gives clone3 ENOSYS so libc can create threads through clone. Seccomp TSYNC
failure retires the command; partial setup must never admit user code.

## Filesystem and descriptor details to close

- Resolve Linux `/bin`, `/sbin`, `/lib`, `/lib64` aliases and exact loader files
  against the candidate's supported distribution. Existing `/usr` loader
  defaults may be retained only to the extent already admitted by the Mac
  contract; do not replace root-deny with whole-host read access.
- Public libc metadata may need explicit `/etc/ld.so.cache`, passwd/group,
  nsswitch and timezone files. Never admit `/etc` as a directory, shadow files,
  private auth configs or user credentials.
- Refuse projects overlapping the engine's runtime home, app data/credentials,
  `~/.local/share/keyrings`, `~/.config/airs-cli`, or other protected stores,
  including alternate XDG roots and symlink aliases. Coordinate the exact list
  with the desktop app's Linux project and Secret Service adapter.
- Existing Bubblewrap mounts a new `/proc` even for a root-deny filesystem
  profile. Desktop must bound that explicitly; prefer its existing `--no-proc`
  mode unless a specific safe prerequisite is demonstrated. Do not assume a
  root-deny permission object automatically denies an added `/proc` mount.
- Close every non-admitted descriptor before the command. Test attempts to use
  `/proc/*/fd`, `/dev/fd`, inherited directory handles and a planted D-Bus socket
  FD to reach private state. StdIO/PTY is an owned capability, not a general
  host-filesystem grant.

## Kernel / packaging prerequisites

Enterprise's actual host is Linux7.2.8-9-omarchy AMD64. Landlock is active;
unprivileged user namespaces are enabled. Actual disposable user+PID+network
Bubblewrap namespace setup passed with Bubblewrap0.13.1. This establishes host
capabilities, not Docker-job permissions or another distribution's runtime.
No kernel setting, service, credential store or global toolchain was changed.

The eventual artifact must be built under the pinned engine Rust1.95.0 and
the agreed Ubuntu22.04 compatibility baseline, then qualified natively on
Enterprise/Omarchy. Require active Bubblewrap/user/mount/PID/network namespace,
capability-drop and seccomp/TSYNC checks; missing support is Unavailable, never
an unsandboxed or legacy fallback. If pidfds are chosen, check them explicitly.

Ship one native Linux AMD64 engine executable with verified resource inventory
and required license/source notices (including bundled Bubblewrap if used).
Keep the desktop seal's no-Node/no-managed-CLI payload contract. Preserve
`airs-desktop/1`, proposal-only authority and all bootstrap/revocation tests.
Record a new reviewed source commit, exact executable/archive/resource hashes
and toolchain/environment evidence before proposing a Linux pin. There is no
new qualified executable hash or publishable engine pin in this prototype.

## Next focused checks

1. Internal Desktop marker propagation and fail-closed unsupported/legacy mode.
2. Strict loader/project/private-store filesystem fixtures and no D-Bus/display
   or network/descriptor escape in the **actual existing helper**.
3. Both pipe and owned raw PTY: stop, engine SIGKILL, interrupted spawn/setup,
   exec replacement, thread attempts, and command exit. Prove no surviving
   owned command and no change to an independent operator shell/native lease.
4. One pinned compatible engine compile plus Desktop bootstrap/protocol
   regressions. Keep generic Harness behavior and the Mac artifact unchanged.

Primary references: [kernel seccomp semantics](https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html),
[parent-death exec caveats](https://man7.org/linux/man-pages/man2/PR_SET_PDEATHSIG.2const.html),
[seccompiler0.5 architecture check](https://github.com/rust-vmm/seccompiler/blob/v0.5.0/src/backend/bpf.rs),
[seccompiler TSYNC](https://github.com/rust-vmm/seccompiler/blob/v0.5.0/src/lib.rs).
