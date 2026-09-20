# Verbatim model output: CLI and embedded judge

Harness **0.1.2-alpha.3.mcp.1** is published under `mcp` at https://npm.cdot.io,
bundling CLI **7.1.4**. Harness stable `latest` remains **0.1.1**. Public CLI
`next` is **7.1.4**, while its `latest` remains **7.0.1**.

The owner clarified that `output` is the model response string, including nested
JSON or serialized messages. Both implementations preserve it verbatim. Prompt
normalization is separate and bounded to recognized message envelopes. Actual
non-string outputs except missing/null fail ingestion. No content-based echo
rejection, response unwrapping or forced verdict remains. Empty/missing outputs
retain their existing exclusion behavior.

The original read-only scan has 4,362 rows. Every response string and original
prompt is unchanged; response hashes match across Python and TypeScript. There
are 36 oversized exclusions and 4,326 eligible rows. The input hash is recorded
in implementation/PARITY.json; no real scan text is retained here. This preserves
the input contract, not a particular ASR. `unrelated_or_error` remains a valid
Jev classification; it is neither a provider failure nor an AIRS disagreement
flag. No paid Jev call or revised accuracy claim was made for this correction.

CLI tests: 2,026 passed across 133 files. Typecheck, lint (existing warnings),
formatting, build and release contracts passed. Python: 26 run with one optional
SDK skip. Rust codex-skills: 51 passed. Launcher/managed CLI: 27 passed; bundle:
26 passed; native packaging: 13 passed. Coverage and command logs are retained.
The first bundle/package pytest selectors selected zero tests; the corrected
selectors produced the passing counts above. They were not interpreted as passes.

Exact candidate and fresh anonymous registry acceptance passed on Ubuntu x64,
native Linux ARM64 and signed/notarized Apple Silicon. This includes installed
regression suites, onboarding, terminal behavior, MCP, doctor, managed CLI,
command output and upgrade/rollback from alpha.2. Separate candidate checks
exercise lifecycle and native TypeSafe key rotation, isolation and cleanup.
Installed verbatim-response dry-run/replay ran against both the bundled CLI and
embedded skill on all three platforms, for candidates and registry installs.
The synthetic four-row replay has three judgments, one empty-output exclusion,
one success and no provider errors. JSON-looking and serialized-message outputs
are asserted equal to their original strings. No owner credentials are used.

Runtime, acceptance tooling and packaging are frozen at
`5c2a3c01e10b8b37845333bcea56e5a829840082`. Linux native build runs are 271/272;
Mac build/acceptance is 273 and signing/notarization is 274. CLI runtime/tag source
is `5953a683e0eccca9ed398152d842edbc5de42319`. CLI source, public tarball integrity,
source freeze, native binary extraction, platform acceptance, publication and
live documentation receipts are retained. Docusaurus deployment receipts identify
their separate documentation revisions.

The first registry-test setup stopped on a shared ARM scratch-directory name
left by an older test. A task-specific directory resolved the collision without
changing the runtime or validators. The failed setup log and helper hashes are
retained under registry-setup-recovery; existing data was preserved.

The earlier response-extraction interpretation is superseded. CLI 7.1.3 was
published and is superseded by 7.1.4; its harness candidate was never published.
That retired candidate has a publication guard and cannot authorize this release.
No historical pass is substituted for these corrected runtime checks.

Completed alpha.2 Mac test copies were backed up with matching SHA256 and length
before relocation to satisfy the unchanged 100 GiB build gate. Backup receipts
are retained; the private archive stays outside Git. Compiler caches, owner
profiles and native credentials were preserved. Temporary publisher credentials
were removed after publication, before anonymous registry verification.

This directory contains selected safe receipts and synthetic tests, not raw scan
contents, credential files, package archives or owner profiles. Scratch and host
paths can occur in diagnostic metadata. SHA256SUMS covers all retained files; a
subsequent immutable Git audit checks the committed evidence and stage links.
