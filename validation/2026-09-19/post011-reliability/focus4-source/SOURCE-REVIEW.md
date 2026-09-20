# Focus 4 independent source review

Status: source slices approved for freeze and actual release acceptance. No numeric PRD4 release score yet. This review is read-only and did not execute Rust, alter application code, or touch owner credentials.

## Product gate

Reviewed airs_product_gate.py, airs_product_workspace.py and test_airs_product_gate.py against the existing verify_acceptance_set authority. The gate preserves that authority rather than replacing it with aggregate booleans, requires the diagnostic-report case and both refresh-fault cases to execute on every native target, and consumes stable native/history/credential round-trip evidence verified by the shared contract. Rollback summaries are allowlisted. Stable0.1.1 is the explicit baseline, and successful output does not authorize stable promotion or claim production SSO/ServiceNow acceptance.

The retained GNU result stays a separate full-workspace diagnostic: current source/tooling/scope artifacts and their hashes are required, exactly one complete nextest summary is parsed, actual count/status/failure-name consistency is checked, and each failed test needs explicit retained nonblocking review evidence. Pending/focused/stale runs cannot substitute. Historical nextest output was inspected to verify the real summary/failure format; current-source evidence remains required.

Review correction: the initial gate omitted the existing registry-only verification_tooling_commit path. The final version forwards that optional identity and tests phase restrictions. Duplicate rollback policy checks were removed in favor of the shared validator. The retained product-gate log shows seven meaningful receipt-chain tests passing, including tampering, missing/skipped required IDs, rollback mismatch, phase/target identity and incomplete or unreviewed GNU diagnostics.

## Git-object audit

Reviewed airs_release_git_audit.py and its adversarial tests. The audit uses pinned committed object bytes, disables replacement refs and inherited Git redirection, prevents transport/lazy fetch, bounds member/blob/aggregate reads, and rejects linked/nonregular members. SHA256SUMS must describe the exact committed inventory without duplicates or self-reference. Known stage receipts, output bytes/sizes, dependency hashes and ACCEPTANCE stage closure are checked. Worktree/index bytes cannot rescue absent or wrong committed evidence.

The retained log shows 13 tests passing, including bare-clone operation, dirty/deleted local files, staged/ignored rescue attempts, malformed manifests, symlinks/gitlinks, reference corruption, read limits, replacement/environment attacks and failure preserving prior output. The historical stable audit at 5c7086341e50757168a8a04a5d56401c52cd0170 passed 242 manifest members and 56 stage receipts. This proves audit behavior against historical committed evidence; it does not establish current candidate acceptance, secret safety, or permission to publish.

## Stable upgrade and rollback

Reviewed airs_npm_versions.py, airs_npm_roundtrip.py, validate_airs_npm_upgrade.py and the spec/execution/contract integration. Stable previous releases require known source and all-three native hash pins; downloaded bytes must match that baseline. Exact versions are installed in one disposable prefix without force/uninstall. A real prior conversation and native inference/MCP identities survive candidate use and exact stable restoration; each phase completes a real synthetic MCP turn. Protected configuration/binding/auth state, unchanged credential generations, exactly three total tool calls and native cleanup are verified.

Review correction: initial command mapping used alpha numbers across all base versions, incorrectly selecting removed airs-harness/setup commands for future0.1.2-alpha.1.mcp.1. Final mapping limits historical cutoffs to base0.1.0; boundary tests include alpha20/21/22 and the proposed next version. Stable/hash pinning is enforced both by the driver and shared result contract.

Final worker has a bounded deadline and graceful termination window. A private copy of the initial stable executable remains available for cleanup if npm fails while replacing the prefix. The ExitStack restoration callback runs after TUI contexts close and before unique credential cleanup; observed successful phase binaries remain unchanged.

The initial same-version development observation records three real turns, exact old native restoration and credential cleanup. It is correctly labeled same-version-development-driver-only and npm_upgrade_acceptance=false. It is not exact next-version npm acceptance. The final frozen tooling observation also passed after the cleanup safeguard (development-roundtrip-final.json, tooling digest6170fcac60a38e79e80b206107997d728ab07bb725a6588f4e4b7824f9f577d7). It retains the same explicit development-only scope. Exact all-target npm release gates remain outstanding.

## Readiness limits

No remaining source blocker identified in these slices. Required before a numeric release readiness score: final formatting/regressions, frozen identities, exact next-version signed/native packages, all-three candidate and registry acceptance including required cases and actual npm upgrade/rollback, current full GNU diagnostics with specific failure review, and a Git-object audit of the newly committed evidence. Stable latest remains0.1.1 until separately authorized.
