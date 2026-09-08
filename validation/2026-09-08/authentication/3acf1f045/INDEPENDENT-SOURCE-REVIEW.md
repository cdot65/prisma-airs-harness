---
title: Diagnostic URL repair and private credential measurement review
status: source-reviewed-verification-pending
reviewed: 2026-09-08
source_commit: 3acf1f0455751734eb41772bf6a7b116a8d5857f
test_commits: [bdd553b465be73b00a11855231ff55a0d1a493e9, 5c2b9ce5a354f01eaf616b01a5f0bd3ae48f77fb]
reviewer: release_review
release_ready: false
tags: [authentication, source-review, diagnostics, private-candidate]
---

# Bounded review; executable verification pending

No additional concrete blocker was found in the diagnostic URL repair and its
private credential-readback fixtures. This is a read-only source assessment,
not release approval. The historical [f964 assessment](../f964e5246/INDEPENDENT-SOURCE-REVIEW.md)
remains 8/10 for its confirmed unsafe doctor path; this review does not silently
restore its superseded initial 9. Final native, installed, and live checks against
these candidate bytes are not credited here.

## Reviewed behavior

- Doctor obtains a bounded, regular-file configuration through the shared safe
  gateway reader before constructing configuration detail or a health request.
  Invalid configuration produces generic diagnostics. The optional access probe
  independently validates its destination before resolving credentials or
  sending the authenticated request.
- Saved OIDC metadata passes the existing public identity-configuration validator
  before rendering its issuer. The regression recomputes a matching identity
  fingerprint around each unsafe issuer, so the URL validator must reject it.
- The executable doctor fixture tests user information, query, and fragment
  canaries in both modes. It asserts no canary in output, zero health/inference/MCP
  requests, a failed configuration check, and unchanged credential-binding bytes.
- The test-only duplicate-snapshot allowance retains both V1 and V2 cases and
  requires the same rendered result. It does not remove a storage format or relax
  the expected output.
- Mac and Windows fixtures compare a fresh credential helper process's captured
  output privately with the expected secret and newline. Logout must then make
  that helper fail with empty stdout. These checks establish more than the new
  metadata-only status, when executed successfully on each native platform.
- The Linux performance fixture separately measures whole version, status, and
  trusted credential-helper processes. The helper bypasses transcript retention,
  verifies exact private output and empty stderr, and suppresses timeout exception
  chains. Receipts contain numeric timings, not captured secrets. No baseline
  subtraction presents the result as an isolated native-store latency.

## Evidence and limits

The reviewer independently ran `python3 -m unittest -v test_validate_workspace_login`
in `scripts`: seven tests passed. They cover private output, failure and timeout
redaction, paired timing samples, percentile calculation, and login disclosure.
They are fixture unit tests, not measured native latency or live authentication.

At review time the parent reported 358 passing CLI tests and one snapshot
framework failure caused by invoking the same inline assertion twice. The
reviewed test-only fix is present; the complete rerun and actual executable
regression are still pending. Do not count them as passed from source inspection.

The performance receipt's top-level `passed` describes functional completion.
The separate `warm_credential_resolution.target_met` must be checked for the
performance gate; a successful fixture can still record a slow percentile.

Doctor's catalog file read and whole-command deadline remain a separate known
limitation at this source. The pending bounded catalog repair is not included in
this review. The current native CI runs also use older source: Mac `34265276427`
uses f964 and Windows `34259921603` uses c10. Preserve their exact outcomes and
digests without transferring a pass to this runtime.

The full [authentication plan](../../../../AUTHENTICATION-PLAN.md) remains
**FAIL / not ready**. Owner-device validation, required desktop and terminal
coverage, signatures, upgrades, profile trust, final authorization/refresh,
teammate installation, and unfamiliar-user trials require their own evidence.
