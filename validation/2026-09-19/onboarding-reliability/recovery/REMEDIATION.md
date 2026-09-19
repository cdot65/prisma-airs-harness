# Selected-environment recovery implementation

Implemented in AIRS-owned CLI modules only. Recovery command rendering preserves
named selection, including valid leading-hyphen names via `--environment=NAME`;
legacy unnamed configuration continues using `airs`. Plain login and doctor derive
the selected public name from the bound home, never the saved default. Welcome
already holds its Selection and now carries it through cancellation and probes.
Environment creation output and help distinguish configuration from guided login.

Loaded registry names now pass the existing validation rule before rendering.
The new security regression first failed against old production code, then passed
with a three-line read validation loop; no schema or persistence semantics changed.

Validation: **955 CLI tests passed, zero skipped**, plus the maintained native
recovery fixture passed under Python optimized mode with two environment subcases.
See RECOVERY-RESULT.json for exact native bytes and log checksums. Earlier logs
include intentionally failing snapshots, reviewed and accepted only for the changed
access/help copy, and the expected RED behavior against published mcp.2.

The maintained fixture is `scripts/test_airs_harness_recovery.py`; it uses existing
HTTPS/PTY fixtures inside a disposable private D-Bus/Secret Service, isolated XDG
state, bounded subprocesses and explicit assertions that remain active with `-O`.
It is Linux-only and explicitly skips on other platforms. Parser and recovery
snapshot Rust tests cover the shared rendering on supported platforms. Root will
run broader installed validation and native package acceptance before publication.

No application commit was made by this subagent. Parent owns final formatting,
linting, review, version bump, package publication and vault updates. The old
`setup never overwrites it` duplicate-create wording was left for a later nearby
copy edit because the request arrived after final source freeze. It is not a
functional gate failure. Owner Ubuntu and live ServiceNow acceptance remain deferred.
