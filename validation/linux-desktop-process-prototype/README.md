# Linux Desktop process guard prototype

This isolated prototype starts from engine `9331a5839bc6a763d94dee7584746f58bbbe1181`.
The initial process-only proof used an unregistered module. The branch now
contains experimental Desktop-only helper/loader wiring; no compatible Linux
artifact is yet registered in the desktop app. The Mac artifact/pin, source
remotes and published releases remain unchanged.

The existing Linux backend already combines Bubblewrap filesystem/PID/network
namespaces, dropped capabilities, no-new-privileges and network seccomp. Its
ordinary command semantics allow process forks; Desktop's accepted Mac contract
allows one command process, threads and exec replacement. The additional guard
denies process creation, session/group escape and namespace changes, preserves
pthread fallback through clone, and refuses x32/foreign syscall ABIs. All filter
installation failures must abort the command before any user-controlled code.

`check.py /absolute/path/to/airs-linux-desktop-process-prototype` runs the actual
module in disposable children with a cleared environment. Its unguarded control
must fail the Desktop assertion, while guarded and exec-replaced controls pass.
This is not an engine, filesystem/network or complete lifetime qualification.

The branch's strict marker, Linux loader-only defaults, private-store exclusions,
bundled-helper checks and PIDFD supervision still require the pinned complete
helper-chain qualification. Do not use parent-death signal alone as a
lifetime guarantee: Linux clears it on certain credential-changing execs. No
`/run/user`, D-Bus/Secret Service, Wayland/X11 or private-home grants are allowed.
The existing restricted network filter must continue denying Unix socket
connections as well as internet access. No generic/alpha engine fallback is
compatible with `airs-desktop/1`.

Source engine toolchain is Rust1.95.0; any prototype run with Enterprise's host
Rust1.99.1 must be labeled as such, never relabeled an exact engine build.
Full pinned engine compilation and boundary review are required before any new
Linux artifact/pin can be proposed.
