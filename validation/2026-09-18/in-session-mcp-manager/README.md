# In-session MCP manager development acceptance

This is a development candidate based on `79599e4b63`, on branch
`feat/airs-mcp-manager`. npm dist-tags are unchanged. The working
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

## Local review build

The Linux x64 development executable is
`/tmp/airs-mcp-manager-20260918/bin/airs`. It was rebuilt after lint/format cleanup
and passed a version smoke check. It retains the base onboarding.4 version string;
its hash in `checks.json` distinguishes it from the published npm artifact. This
is an unoptimized native development binary, not a packaged release. Launch it
with `--environment NAME`, then enter `/mcp`. The installed `airs` command and
npm dist-tags were not replaced.

The artifact used by the terminal fixtures is recorded separately from this
final rebuild. Tests preceded the final lint/format cleanup; no claim of final
release-artifact acceptance is made. Runtime source is `bc68d3db5b`, following
CLI adapter commit `3bee0c3e2d`; fixture/documentation source is `c4c659defd`.

## Linux x64 remote test candidate

Version `0.1.0-alpha.22.mcp.1` binds runtime/version source
`15a7f229bcfd08677ca9f193b8ff05db41f30737`. The earlier unversioned local build above
is retained as historical evidence. The owner requested remote testing after the
local credential-store failure persisted; that failure remains unresolved.

The portable bundle includes Prisma AIRS CLI 7.0.0 and installs into a new prefix
using a temporary loopback registry. Linux x64 is the only packaged platform.
The native executable is an unoptimized dev build (debug=0, incremental=false).
Native and npm candidate/private markers remain intact. This is a review download,
not npm publication or full release promotion.

`remote-install.json` verifies the exact installed native hash and managed CLI
inventory, with no unexpected registry requests. `remote-native-store.json`
records 18 successful isolated native Secret Service/onboarding checks on these
same native bytes, including browser PKCE, device login, workspace keys and
storage-failure recovery. These fixtures do not establish production SSO or
ServiceNow acceptance or resolution of the owner's local store failure.

The archive uses the existing `install_airs_review.py` and its verification tools.
Its layout follows `package_airs_review.py` with Linux x64-only package selection
and candidate-specific instructions; release validation guards are unchanged.
Remote download, archive digest and installed MCP results are recorded in
`remote-candidate.json`.

Download: [Linux x64 test candidate](https://git.cdot.io/cdot/prisma-airs-harness/releases/tag/v0.1.0-alpha.22.mcp.1). Forgejo sign-in is required; the release includes browser and authenticated terminal download instructions. The uploaded archive was downloaded again with authentication and its SHA-256 matched. All four MCP terminal fixture cases passed against the portable installation.

## Subsequent npm publication

After the owner explicitly requested npm publication, optimized `0.1.0-alpha.22.mcp.1` builds were published under the `mcp` tag for Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. Fresh anonymous registry installs passed on all three. See `../mcp-manager-npm/README.md`. The earlier archive receipts above remain historical and immutable.
