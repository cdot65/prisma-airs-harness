# In-session MCP manager development acceptance

This is a development candidate based on `79599e4b63`, on branch
`feat/airs-mcp-manager`. No npm version or dist-tag is changed. The working
onboarding.4 release remains published.

The manager adds gateway MCP servers, routes existing `/signin` actions to the
private browser/callback dialog, reconnects/verifies, signs out and removes
connections. It preserves the existing fresh-conversation boundary and uses the
running environment's home and working directory. Core inference, public APIs,
configuration schemas and dependencies are unchanged.

## Local evidence

- Affected CLI/TUI suite: 5,280 passed, six skipped.
- Final focused manager/recovery checks: 14 passed. Other tests were filtered.
- Scoped CLI/TUI Clippy completed without warnings; final formatting completed.
- Three existing standalone CLI callback terminal tests passed.
- Reviewed six manager/authorization/recovery snapshots, including narrow width.
- Real-terminal HTTPS fixtures cover hidden remote callback entry, same-machine
  HTTP callback delivery, cancellation, discovery failure, duplicate rejection,
  add/reconnect/sign-out/removal, changed-default isolation and preserved inference.
- Configuration removal does not claim success if another configuration layer
  still supplies the connection. Removal intentionally retains OAuth credentials;
  choose Sign out first to clear them.

Run the terminal fixture with a built executable:

```sh
AIRS_HARNESS_BIN=/absolute/path/to/airs python3 -m unittest discover \
  -s scripts -p test_airs_mcp_manager.py -v
```

The fixture uses disposable loopback inference and HTTPS OAuth/MCP servers. It
issues synthetic credentials in an isolated file store; it does not use the
owner's production credentials or change system trust. Unit snapshots preserve
an unsent draft and test cancellation actions and stale-attempt invalidation.
The terminal fixture asserts no inference calls occur during connection
management, then explicitly submits a message after MCP removal.

## Release boundary

This does not establish real CAS/Keycloak browser login, gateway-managed upstream
ServiceNow OAuth, a production ServiceNow tool result, locked native-store
acceptance, or native ARM64/Apple Silicon acceptance for this candidate. Those
checks and release packaging remain required before promotion under AGENTS.md.
See `MCP.md` for the development workflow and `UPSTREAM.md` for merge hooks.

Review the smallest independent stage first: the private CLI interaction adapter
and duplicate guard. The remaining stages are the private callback UI, manager
orchestration and tests, then executable fixtures/documentation. All new runtime
modules are below 500 lines. The complete feature exceeds the usual 800-line
change guideline because it includes a private terminal form, cancellable child
adapter, lifecycle menus and installed-executable fixtures; these boundaries
allow the review to be split without introducing a second OAuth client.
