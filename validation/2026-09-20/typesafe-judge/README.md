# TypeSafe Jev harness test release

Version **0.1.2-alpha.2.mcp.1** is published under `mcp` at https://npm.cdot.io.
The launcher bundles CLI **7.1.2**. All four packages preserve `latest=0.1.1`
and every other non-mcp tag. Public standalone CLI 7.1.2 is available under
`next`; its public latest remains 7.0.1.

Candidate and fresh anonymous registry acceptance passed on Ubuntu x64,
native Linux ARM64 and signed/notarized Apple Silicon. Each installed suite
ran 53 tests, with 3 platform-specific skips on Mac and 1 on each Linux host.
Onboarding, terminals, MCP, doctor, managed CLI, command output and actual
stable upgrade/rollback preservation passed. Mac installed signature and
notarization checks passed. Extra lifecycle and native TypeSafe key rotation,
child scope, environment isolation and cleanup checks passed on all three.

The actual `airs cli redteam judge` command was exercised from both candidate
and registry installations on every platform. A synthetic 11-unit input produced
9 judgments, 1 error-output exclusion, 1 oversized exclusion and 0 provider
errors. Dry run and replay needed no tenant credential and made no Jev calls.
The original representative 4,362-row file remains unchanged; separate bounded
integration checks exercised 4,326 eligible rows and 36 oversized exclusions.
No live model accuracy, calibration or production SSO/ServiceNow claim is made.

Runtime, validator and packaging revisions are separate and pinned in SPEC.json.
SOURCE-OBJECTS.json in source/ checks 189 validator/packager inputs against Git.
The focused Rust suite recorded 5,416 passed and 6 skipped; native Python judge
fixtures recorded 22 passed; the CLI suite recorded 1,997 passed. The final
corrections changed test scripts, a dependency download URL and documentation,
not the compiled Rust runtime. Earlier failures remain visible:

- Assembly initially refused the private CLI archive URL. Public npm publication
  and an integrity-verified URL-only lock correction resolved this.
- Four loopback fixture addresses had been accidentally changed during a prior
  version edit. They were restored before accepted native checks.
- The first installed onboarding checks missed the new optional TypeSafe screen.
  Corrected validators select Continue to AIRS. failed-onboarding-v2/ preserves
  the original logs, successful install-stage receipts and failed completions.
- Mac packaging hit historical SOURCE/source case collisions. A case-sensitive
  checkout produced the same signed binary. The historical path correction then
  passed an immutable 349-file/56-stage Git audit. The temporary volume is detached.
- The first ARM validator-transfer restart used an unavailable Python tarfile
  keyword. Bounded regular-file extraction restored compatibility without any
  binary change; the failed stage was not counted as passed.

Public CLI docs deployment c49a892 passed; five live pages match the local build.
The private publishing config was removed and task-owned GUI jobs unloaded.
Compiler caches, owner credentials and existing production environments were
preserved. Metadata can reference scratch/host paths retained for diagnostics;
no production credentials, real scan content or npm archives are committed here.

SHA256SUMS covers the retained evidence. The final immutable Git audit is stored
in a subsequent commit so the evidence inventory does not refer to its own hash.
