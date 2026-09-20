# Interrupted refresh recovery

Source commits: `41a617e64c` (startup shell recovery), `fd9548d510` (typed refresh faults), and `3d62898236` (native installed observation and release handoff).

Both installed synthetic cases passed through unittest discovery in 131.799 seconds. Each starts with signed inference, an independent native MCP credential, a nonce-verified read and real saved conversation. A normal refresh succeeds before a rejected or consumed-but-lost refresh is injected. Repeated helper processes and a new resume process preserve the failure classification without resubmitting the consumed predecessor. Resume exits before the TUI with a selected-environment `login --restore-session` command. Configuration, binding, sibling environment, real conversation and MCP credential remain intact; exact test-account native cleanup completes before success.

`RUST-CHECKS.json` binds the full CLI suite (959 passed), final focused/snapshot tests (14 passed), lint, formatter and build logs. The final development executable SHA256 is `da54e47800ddfe78a0d233508fe0f6b92776f93a804472470056948b20a1c79e`; its prestamp version is 0.1.1. It is not a new published package. The installed receipts bind the unchanged validator inventory as well.

Initial failed observations are retained as safe phase/type records; their empty stdout files contain no receipt. They used an incorrect oracle expecting the TUI to open, which exposed the pre-TUI recovery-message defect. No raw terminal callbacks or token material are retained.

These observations do not establish attended production reauthentication, production expiry/revocation, an active-session recovery dialog, or behavior after a lost MCP tool response. Exact next-version acceptance on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon remains the release gate. Stable latest remains 0.1.1.

Independent implementation review: **9.5/10**; see `FINAL-REVIEW.md`. This score does not waive the later native package or publication gates.
