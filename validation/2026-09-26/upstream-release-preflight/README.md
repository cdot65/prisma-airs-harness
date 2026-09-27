# Apple Silicon signing access preflight

The existing Developer ID identity is present. The owner unlocked the login
keychain, but direct SSH and SSH PTY notarization probes continued to report it
locked. The build SSH session identifies as Background; the owner desktop
session has a separate security context.

A temporary `launchctl bootstrap gui/501` job ran `xcrun notarytool history`
against the existing `prisma-airs-harness-notary` profile and explicit login
keychain. It exited successfully with a valid response containing 40 historical
submissions. The job was removed immediately with `launchctl bootout`. No
password, API credential or keychain item was read/exported, and no keychain
settings or persistent service were changed. Raw historical submission data
remains in the private Mac cache; only aggregate preflight facts are retained.

Reuse the alpha.5 GUI-session signing orchestration in
`validation/2026-09-25/mcp-routing-alpha5/orchestration/signing-runner.py.txt`.
Its source-bound intake and existing `scripts/airs_forgejo_gateway_package.sh`
provide signing, submission, native-store validation and candidate packaging.
The failed SSH probes alone did not establish that this release path was blocked.

This is profile-access evidence only. A new build, actual codesign/notarization,
exact candidate acceptance, publication and fresh registry acceptance remain.
