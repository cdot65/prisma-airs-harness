# Mac terminal preview: 0.1.3-alpha.3.mcp.1

Runtime and packaging: `7b177cc58a875f7f35ab91960cbc06f57784d7e1`.
Frozen validation tooling: `a7488be6ed6f526717a7e3c970f38cdf51f35bd7`.

The Apple Silicon native package and Mac-only launcher are published under
`mac-preview` at https://npm.cdot.io. Stable `latest` remains 0.1.2; `mcp` remains
0.1.3-alpha.2.mcp.1. CLI remains 7.1.5. Existing tags and all Linux versions are
unchanged. Linux distribution builds await owner Mac acceptance. npm versions
are immutable, so a subsequent all-platform release needs a new version.

Exact candidate and fresh anonymous registry checks passed, including Developer
ID signature, explicit Apple notarization verification, native Keychain, transcript
resize/drafts, question recovery without sending, MCP manager, doctor and the
actual Jev agent approval boundary. Real isolated tmux 3.7c passed. Upgrade and
rollback from both stable 0.1.2 and alpha.2 preserve native credentials and history
while completing synthetic MCP turns. No owner SSO, ServiceNow, paid Jev or model
accuracy acceptance is claimed. Windows is not a distributed or tested native
target for this preview. Fullscreen selection/search is explicitly deferred.

Full GNU workspace: 18,423 passed, 3 failed, 34 skipped. The complete raw log and
comparison with the raw stable baseline are retained. The three remote-shell
assertions match after normalization of timestamps, ANSI, indentation, PIDs and
temporary paths. The suite is not fully green. Final TUI source tests: 4,382 passed,
2 skipped; release contracts: 82 passed; workflow/cache checks: 3 passed. Native
signing tests and installed registry suites have their own distinct skip counts.

Initial notarization ticket propagation checks failed after Apple accepted the
submission; bounded rechecks passed against identical bytes. Earlier terminal
fixtures made incorrect assumptions about raw bytes, title requests, resize timing
and request order; corrected frozen tooling passed. Those failures remain in
mac-extra-evidence and implementation. No runtime rebuilding or resigning was
needed for those test corrections.

READINESS.json records four implemented-phase scores of 9/10. Owner account and
terminal acceptance reserve the final evidence point. UPSTREAM-ADOPTION.md records
upstream attribution, adaptations and deferred prerequisites. The public guide
passed 23 browser checks; LIVE-DOCS-VERIFIED.json binds its reviewed build to the
successful deployment and 16 live pages. Performance checks use ten warm launches,
fresh application homes and no inference/tool/token calls, not cold-boot timing.

SHA256SUMS covers every retained file. The separate Git-object audit verifies the
committed inventory and each acceptance receipt's dependencies and output hashes.
