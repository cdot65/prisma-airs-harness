# Current GNU failure review: remote-sandbox

Test: `codex-exec-server::exec_process shell_snapshot_v2_filters_profile_exports_and_stays_in_memory::remote_sandbox`

Source/tooling: `44872ebf4c125f06fa69bee7a486f8e7ab26c6af`. Run260 (internal3724), full-workspace. Raw `validation.log` SHA256 `24ad97b0f4aa1fdca0cb39c0e898234cddb188998a51b6e88c2c45bfb4b2793e`; matching failed-test lines: 10015, 10048, 19129.

Both attempts fail at exec-server/tests/exec_process.rs:331. The synthetic .bashrc writes captures under its test HOME, receives Read-only file system, and exits41 instead of returning the expected filtered helper/profile output. This is a real failed remote snapshot-v2 sandbox scenario, not a successful sandbox smoke test.

Disposition: nonblocking for this default AIRS product release, with the failed experimental remote shell snapshot-v2 behavior explicitly excluded from readiness claims. Current `features/src/lib.rs:975` identifies ShellSnapshotV2 as UnderDevelopment and default_enabled:false. Current `core/src/unified_exec/shell_snapshot.rs:180` returns no snapshot request unless enabled; prewarm is also gated at37. These immutable source excerpts and blob IDs are retained in `CURRENT-SOURCE-REVIEW.json` (SHA256 `0f846b52f52d5ae972a86d7f7aaf492ac8127de43156072c1eb9ad9f909358cc`). The fixture directly requests a snapshot and therefore exercises a path that default-disabled product execution does not take.

Current related coverage: local pipe/TTY recovery and all local filtering variants passed; remote non-sandbox filtering, remote retry-budget exhaustion, managed-proxy context, and core profile-secret filtering passed. All six codex-v8-poc tests passed. The related source objects match the pinned stable source, which establishes that this release did not modify them; this is contextual evidence, not a reused historical waiver.

This review does not resolve or waive the defect for opt-in remote shell snapshot-v2. Full GNU status remains failure (18331 passed,3 failed,34 skipped), and no production ServiceNow/SSO result is inferred. No test or feature default was changed and no failed test was skipped or removed. Follow-up belongs to the experimental remote snapshot-v2 path before enabling it by default.
