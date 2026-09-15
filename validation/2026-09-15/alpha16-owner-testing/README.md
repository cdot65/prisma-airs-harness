# Alpha.16 owner testing publication — September 15, 2026

Alpha.16 is published for Linux x64 and Apple Silicon at https://npm.cdot.io under
`gateway-validation`. `alpha` and `latest` retain alpha.14; alpha.15 remains immutable.

```sh
npm install -g airs-harness@0.1.0-alpha.16 --registry=https://npm.cdot.io
```

The owner authorized updating and publishing the alpha binary with the inference
recovery fix. Source is frozen at `6020835a7995de4a25caac5a6158b5400d4b6925`.
Alpha.15 bypassed typed credential recovery in normal provider dispatch and showed
a fatal helper error after approximately 30 minutes idle. The fix, already merged
in PR 38, routes AIRS command authentication through scoped recovery and preserves
`/signin` guidance. The user's manual alpha.15 sign-in restored the same identity,
conversation and subsequent inference reply. The 30-minute idle policy remains.

## Verification

- The real-helper provider regression failed before the fix and passed afterward;
  all 78 provider checks and scoped Clippy passed. The built provider and test files
  are byte-identical to the tested fix commit `d2f7a238a`.
- Eleven native packaging contract tests passed. Linux release compilation and
  both native package preparation workflows passed.
- Apple Silicon build/acceptance passed Forgejo run 136. Developer ID signing,
  Apple notarization and native Keychain persistence passed run 137. Source release
  contract checks passed run 138.
- Exact npm upgrades from alpha.15 passed on both platforms, preserving configured
  state and both supported legacy command layouts.
- Anonymous fresh registry installs verified native/archive integrity, the bundled
  CLI inventory and executable checks: 43 passed and one platform-inapplicable
  check skipped per platform (Linux Seatbelt; Mac session-bus recovery). Mac installed bytes passed
  strict signature and online notarization verification again.

The distributed validation receipts deliberately retain `passed: false` and
`release_ready: false` for full production gateway lifecycle acceptance. No new
browser consent, authenticated utility call, concurrent frontend renewal or
long-running expiry cycle is claimed for these alpha.16 bytes. This owner-requested
testing publication does not change the regular release promotion gate.

## Documentation handoff

The documentation technical revision was prepared in
`prisma-airs-reference-architecture`, branch `docs/mcp-server-1-calvinize`: all 15
lessons, 18 Mermaid diagrams, the landing page and the authorization exercise now
use **mcp server 1** and its eight local utility tools. Both inference and MCP
still traverse AI Gateway. Utility execution runs on the remote MCP server host
without an outbound management-API call; authentication still uses public signing
keys and gateway-owned upstream OAuth.

The production build and all 19 browser checks passed for the technical revision
and again in an isolated checkout of Claude Code's subsequent voice rewrite at
`2948b29`. Documentation writes stopped at the owner's concurrent-editing handoff;
this release does not deploy Pages or overwrite those editorial changes. Canonical
vault indexing completed; checking reported the same 63 baseline errors and only
additional pending-review warnings. Full vault cleanliness is not claimed.

The MCP inventory receipt records read-only inspection of both running replicas,
not authenticated tool-call acceptance. SCM management API troubleshooting remains
stopped because those tools are no longer in the target inventory.
