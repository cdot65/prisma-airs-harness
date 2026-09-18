# In-session connection doctor — development acceptance

Linux x86_64 development binary: `/tmp/airs-session-doctor-20260918/bin/airs`.
Launch it with your normal environment, then enter `/doctor`. The source is
`3812aa011572f47af06d133bd22af9a771dbbbac`; exact bytes are recorded in `build.json`.
This build retains the base `0.1.0-alpha.22.mcp.1` version metadata. It is a
development artifact, not a new npm package or platform release.

The affected CLI/TUI Rust suites passed 5,284 tests, with six skipped. Five UI
snapshots cover healthy/degraded diagnostics, narrow terminals, unavailable
diagnostics, explicit verification consent and cancellation. Scoped Clippy
passed without warnings, and `just fmt` completed. Unrelated preexisting
formatting changes were reverted. No core, shared protocol or dependency changes
were made, so a full workspace run was not required.

The exact copied binary passed 15 acceptance checks:

- Four doctor checks: slow-check cancellation/retry; environment binding after a
  default change; explicit inference confirmation, HTTP 403 reporting and MCP
  manager handoff; native-store failure and a stalled D-Bus service with bounded
  completion and preserved credential files.
- Seven existing CLI doctor regressions. One npm-managed CLI integration was
  skipped because this is a native development artifact.
- Four existing MCP manager checks covering browser callback entry, cancellation,
  failed discovery, duplicate names, add/login/verify/logout/remove and inference
  environment preservation.

Raw fixture summaries are adjacent. Initial failures exposed missing new
snapshots, narrow-screen clipping, an incorrect fixture scroll count, and a
plain-CLI credential-inspection regression; these were corrected before the final
runs. Four cursor-color tests also failed under inherited `NO_COLOR=1`; the final
suite passed with that variable unset and `TERM=xterm-256color`.

These fixtures do not establish production ServiceNow acceptance. The owner
still needs to complete gateway SSO/upstream consent, run a read-only incident
lookup, restart and confirm persistence. The original user's Linux credential
store failure has not been reproduced or declared fixed. Apple Silicon and Linux
ARM64 have not been built or accepted for this change, and no npm tags were moved.
