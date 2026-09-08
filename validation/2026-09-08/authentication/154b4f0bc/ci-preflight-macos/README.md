# Mac workflow preflight evidence

The actual GitHub preflight-only run [34272402139](https://github.com/cdot65/airs-harness/actions/runs/34272402139) passed on Ubuntu in 22 seconds. Its 21 tests passed without skips. Both Mac compilation and native acceptance jobs were skipped as requested.

These are downloaded, unchanged test logs and the workflow-generated `PREFLIGHT.json`. `RUN-RECEIPT.json` records separately collected GitHub API metadata, verified artifact ZIP SHA-256, counts, and the original runtime/tooling source identities. The downloaded ZIP is retained locally; the reviewed text evidence is tracked here. The artifact contained synthetic fixture tests and no production credentials.

This proves the inexpensive preflight lane and its build-skip boundary. It does not prove macOS Keychain behavior, signed distribution, or native end-to-end acceptance.
