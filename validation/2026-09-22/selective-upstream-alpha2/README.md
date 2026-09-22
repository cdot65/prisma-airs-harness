# Selective upstream preview: 0.1.3-alpha.2.mcp.1

Runtime and packaging: `bb31dcdd813122f577efb1b8b647f2c06f3078c4`.
Validation tooling: `766c9fb83a9bc32deacd2125cefcafd5b6527db7`; the only follow-up changes
native MCP test timing, not Rust runtime bytes.

The four npm packages are published under `mcp` at https://npm.cdot.io. Stable
`latest` remains 0.1.2 and bundled CLI remains 7.1.5. Exact candidate and anonymous
registry acceptance passed on native Linux x64 (Alpine), native Linux ARM64 and
signed/notarized Apple Silicon. All three also passed Jev's actual agent approval
boundary, native key handling, explicit doctor checks and terminal policy-denial
request-count checks. Candidate checks cover 0.1.2 upgrade/rollback and timed
fixture refresh. No owner Ubuntu, real-account SSO/ServiceNow, paid Jev or accuracy
acceptance is claimed. Stable promotion remains separate.

Full GNU workspace: 18405 passed, 3 failed, 34 skipped.
The failures are the three individually reviewed experimental remote-shell cases
already present in the stable baseline. Original complete logs are retained;
this is not a fully green suite.

The unpublished alpha.1 candidate was superseded after the actual agent probe
exposed HTTP 200 JSON/SSE policy-denial retries. The final runtime stops them.
Mac CI run 3829 failed its early menu-selection fixture. A bounded retry of only
the passive menu after an explicit startup-busy response corrected that fixture;
all later installed candidate and registry checks passed with pinned tooling.
A direct SSH-shell validation attempt could not use the Mac native store and
failed earlier in login; required Mac checks ran in its GUI login session.

READINESS.json records the seven 9/10 feature scores and their acceptance scope.
UPSTREAM-ADOPTION.md at the repository root documents adaptations and deferrals.
The published guide was checked against the reviewed production build and the
successful deployment's source revision; see LIVE-DOCS-VERIFIED.json.
