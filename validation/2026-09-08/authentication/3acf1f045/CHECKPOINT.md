---
title: Authentication 3acf executable checkpoint and catalog regression baseline
date: 2026-09-08
status: failed-validation
source_commit: 3acf1f0455751734eb41772bf6a7b116a8d5857f
binary_sha256: 39d238e30af3ef0398abacfc3bac6efdde6404dca0cf49eb0a876c72f2c50eda
release_ready: false
tags: [authentication, validation, regression, private-candidate]
---

# Failed checkpoint; remediation verification pending

This checkpoint preserves results for immutable Linux executable **3acf1f045**.
It is not release approval. The binary SHA-256 was independently recomputed and
matched the retained build receipt. No evidence here establishes that the pending
154b4f0bc executable fixes the failures below.

| Evidence | Actual result | Retained record |
| --- | --- | --- |
| CLI unit/fault tests | 358 passed, 1 failed, 0 skipped; 359 total | [CLI receipt](cli-tests.json), [sanitized JUnit](cli-junit-sanitized.xml) |
| Executable acceptance | 36 methods passed, 1 skipped, 1 failed; 38 total | [Executable receipt](executable-tests.json) |
| Build provenance | Binary hash verified; 711 seconds wall time including Cargo lock wait | [Build receipt](binary-build.json) |

The CLI failure is `airs_status::tests::native_metadata_does_not_require_any_stored_secret`.
Both native-format cases execute one inline snapshot assertion; Insta rejects the
duplicate without `allow_duplicates`. Test-only commit bdd553b46 addresses that
framework requirement. A successful rerun is not credited by this checkpoint.

The executable suite uses test source **154b4f0bc** against the **older 3acf binary**.
The new `test_doctor_rejects_oversized_or_fifo_catalog_without_blocking` method
reproduces two defects: an oversized catalog returns success, and a FIFO catalog
blocks until the fixture's ten-second subprocess timeout. These are two failing
subcases within one method, not two additional methods. The managed Prisma AIRS
CLI skill method is skipped because this invocation is not npm-managed. Preserve
that coverage gap; the other 36 methods passed.

The Cargo timing report records the CLI library unit at **58.97 seconds** and
`airs-harness` binary unit at **502.84 seconds**. Its overall duration is rounded
to 712 seconds, while the operator build receipt reports 711 wall seconds. The
wall measurement includes lock wait, which was not separately isolated. These
numbers do not demonstrate a build-performance improvement. The report's local
path and SHA-256 are retained without copying the 661,516-byte HTML report.

Raw executable logs are not committed. The JUnit copy removes process output,
error streams, and properties while retaining test identities, durations, and
statuses plus a bounded framework failure reason. Original local artifact paths
and hashes remain available in the receipts for reconciliation.

See the separate [source review](INDEPENDENT-SOURCE-REVIEW.md). Owner macOS
reproduction, native platform acceptance, installed/live integration, signing,
and the full authentication plan remain independent gates. This checkpoint is
**not ready**, and no prior scan failure is reclassified by these local tests.
