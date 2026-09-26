# Scoped TLS dependency review

Only rustls 0.23.36 -> 0.23.45, rustls-webpki 0.103.13 -> 0.103.15, aws-lc-rs 1.16.2 -> 1.18.1 and aws-lc-sys 0.39.0 -> 0.45.0 changed. Cargo generated the lock update; versions/checksums agree with upstream e5a2094817 (PR 45489). Bazel regeneration and strict lockfile_mode=error pass on native Mac; parsed lock diff has only the expected TLS crate facts. No model/provider selection, credential, endpoint, certificate validation or fallback policy code changed.

Evidence: 458 Linux and 466 native Mac affected tests pass; eight existing ignored tests on each, plus the explicitly excluded three Mac upstream remote-executor cases described in the preceding feature. Coverage includes live local custom-CA TLS, TLS-intercepting CONNECT proxy, rejection of untrusted roots, bounded protocol fallback, non-replayable requests, WebSocket proxy/trust behavior, identity tokens and MCP OAuth discovery/refresh/store isolation. These are local fixtures, not live company SSO acceptance.

The initial Linux run failed six native-backend tests because Cargo reinjected CA environment overrides, selecting Rustls. The unchanged product/test binaries pass with the existing test-only CA-clean runner. Added removal of its musl runner variable, matching the existing GNU handling so spawned helpers resolve real executables. A direct child-process check confirms CA/runner variables are absent and unrelated environment is preserved. No product environment handling or trust bypass was added.

No Rust source changes in this slice; preceding affected-source scoped lint remains applicable. Formatting and whitespace checks required at gate. One known static-header expect_used warning remains recorded in the preceding source review. Dependencies are not blanket-upgraded; musl OpenSSL distribution changes are separate from this Mac-first deliverable.

Proposed scores after formatting: implementation 9, code quality 9, design 9, bounded feature completeness 9. Final workspace, exact signed package and owner live-account checks remain separate.

Formatting passed; unrelated baseline formatter changes restored. Gate closed at 9/10 in each dimension for this slice.
