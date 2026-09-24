# Blue menu selection: 0.1.3-alpha.4.mcp.1

Runtime/packaging: `76d88c609556d1e00fbe1925ce34fe9521920364`.
Frozen validation tooling: `061c3c73a3af0348520d3ef2daa6a77cbdb5f054`.

Apple Silicon signed/notarized native package and Mac-only launcher are published
under `mac-preview` at https://npm.cdot.io. Stable latest stays 0.1.2 and mcp stays
0.1.3-alpha.2.mcp.1. CLI remains 7.1.5. Existing tags and Linux distributions are
unchanged. Linux builds await owner Mac acceptance; another platform release
requires a new immutable npm version.

Shared menu rows use upstream blue fills with contrast checks on known truecolor
and 256-color palettes, with reverse video on unknown/limited palettes. Selection
covers wrapped descriptions, truncation and trailing cells. Menu layout, keyboard
behavior and AIRS gateway/OAuth flows are unchanged; fullscreen remains deferred.

TUI source suite: 4,388 passed, two skipped; scoped Clippy passed. Ten visual
snapshots were reviewed. Exact signed candidate and anonymous registry installed
checks passed, including actual PTY palette negotiation for light/dark blue menus
and arrow-key movement. Native Keychain, MCP manager, doctor, managed CLI and the
actual Jev agent approval boundary remain exercised. Stable and alpha.3 upgrade/
rollback preserve history and native credentials with synthetic MCP turns. These
fixtures do not claim owner SSO, ServiceNow, paid Jev or model-accuracy acceptance.

Full GNU suite: 18,429 passed, 3 failed, 34 skipped.
The complete raw log and direct comparison to the raw stable baseline are retained.
See WORKSPACE-REVIEW.json for exact status; inherited failures are not called green.

The original Mac CI acceptance failed only the new blue test's single-row assumption
in both themes: a selected option legitimately wraps. Frozen tooling corrects that
assertion and verifies contiguous selected rows and movement. Signed native and
installed candidate/registry acceptance passed with that correction, without a
runtime rebuild. Original CI failures remain in implementation. The native signing
suite ran 56 tests: 52 passed, three Linux cases and one npm-managed CLI case skipped;
installed package suites exercise that managed CLI case.

READINESS.json scores bounded implementation, native acceptance and Mac delivery
at 9/10. Owner visual/real-account acceptance remains separate. Public guide passed
23 browser checks and all 16 live pages match the reviewed main content after only
CSS module hash normalization. Startup performance uses warm launches and isolated
fixture state, not cold-boot timing. Source provenance is in UPSTREAM-ADOPTION.md.

SHA256SUMS covers every retained file. The separate immutable Git audit verifies
the committed inventory and acceptance receipt dependencies and output hashes.
