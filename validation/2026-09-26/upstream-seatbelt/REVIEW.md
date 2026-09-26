# Adversarial feature review: restricted macOS mutating fcntl denial

Reviewed production diff and native upstream test source, not only test totals.

- Scope: deny commands 80/110 only when policy does not allow full disk write. No changes to network, credentials, environment identity, gateway routing, or authorization requests.
- Negative cases: native test opens files with read-only descriptors and checks EPERM for both mutation operations under read-only and workspace-write policy, then compares file bytes, block allocation, flags and timestamps.
- Positive control: unrestricted policy permits compression/truncation. Extent transfer positive control tolerates filesystem EINVAL/ENOTSUP explicitly; this does not waive restricted EPERM checks.
- Parent/child ownership: test keeps preallocation donor open while collecting metadata, drops it after comparison; child uses owned Files and a temporary directory.
- Platform caveat: nested Seatbelt refusal is a test early return; native acceptance must show captured success output without the skip marker. Linux results alone cannot prove this feature.
- Initial native suite: new test passed; two old tests failed because Homebrew Bash prepends line 1. Unchanged baseline reproduces both failures. Use system Bash as intended, retaining all assertions.
- Maintainability: 11 production lines plus unchanged upstream separate 256-line tests; no unrelated formatter churn, new dependencies or protocol changes.

Final four scores remain pending until the normalized native run and native lint complete.

Gate closed: native tests and native Clippy passed. All four dimensions 9/10 for this bounded feature; release validation remains pending.
