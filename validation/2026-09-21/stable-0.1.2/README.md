# Stable 0.1.2 release evidence

Native source and frozen validation tooling: `aee13a272b5e79fe0ea6fe62485ca7c164927028`.
Packaging and guide/helper version alignment: `8a1dd5324c4c6c2345b8d8737e0400308fe67cab`.
Promotion tooling: `9402e7ec7543a44210883950c5f338903472fd43`.
The stable harness bundles CLI 7.1.5. All four packages are published and promoted
to `latest`; candidate, anonymous registry and unversioned default installs were
verified on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon.

Both stable 0.1.1 and alpha.5 upgrade/rollback paths are checked. The owner
confirmed alpha.5 inference, ServiceNow through `/mcp`, restart/reuse and Jev.
Only the version stamp changes the stable native runtime relative to that accepted build.
The Ubuntu preparation helper now defaults to 0.1.2; its six isolated checks ran
against the published package without a version override. An earlier attempt
correctly failed its registry-availability check before publication.

An initial package set passed installed checks but contained stale guide versions.
It was never published. The corrected set received fresh installed acceptance.
PACKAGING-CORRECTION.json records the retained superseded evidence and package
identity boundary. The collector was corrected to exclude disposable npm links
from portable reports; the original collection error remains in the logs.

The actual agent tool executor and approval UI are exercised with saved native
keys, inherited-key precedence, denied approval and redacted outputs. TypeSafe
responses and inference decisions in automated tests are fixtures. They establish
execution and credential handling, not Jev model accuracy or calibrated ASR.

Full GNU workspace counts: 18374 passed,
3 failed, 34 skipped.
The original log and source receipts are retained. Read READINESS.json for any
individually reviewed failures; this record does not turn failures into passes.

Completed runner artifacts were backed up and verified before relocation.
The archives remain outside Git; verification receipts are included here.
