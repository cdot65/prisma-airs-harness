# Alpha.14 gateway publication — September 14, 2026

Alpha.14 is published to Verdaccio for Linux x64 and Apple Silicon. Both `alpha`
and `latest` select the exact packages in PUBLICATION.json. Ordinary npm updates
on Linux and Jadzia installed those bytes and preserved existing state.

Both inference and native Codex MCP use AI Gateway. The gateway owns upstream
OAuth; no separate local MCP executable is used. Production/development routes
and upstream gateway-client allowlists are deployed.

## Completed evidence

Both exact installed binaries completed human inference SSO, gateway MCP consent,
native credential persistence, all eight model-selected production tool calls,
gateway/upstream tool correlation, invalid-bearer denial, terminal MCP inventory,
and history preservation. Gateway-held upstream JWT renewal was observed after
actual five-minute expiry while frontend credentials remained unchanged.
Correlation uses tool names and timestamps within two seconds; no shared trace
header is claimed.

Fresh anonymous registry installs verified archive integrity, every installed
archive member, native executable hashes and the managed CLI bundle. Both passed
44 executable tests (one platform-specific skip each). The downloaded Mac binary
passed Developer ID signature, hardened runtime, Apple notarization and Keychain
checks. Ordinary alpha.13 upgrades and two legacy command layouts passed before
publication; owner upgrade receipts confirm the final installed versions.

## Explicit owner release scope

The owner directed release without another hour-long authentication run. This
is a limited internal alpha release. Original lifecycle E2E receipts retain
`passed: false`; their `refresh_cycles: 2` field is the requested count, not a
completed count. Zero hourly frontend refresh cycles completed. Concurrent
frontend recovery remains unverified. The idle run also failed inference access
verification after MCP logout when its refresh grant expired; inference logout
was not reached. Native MCP logout/removal passed.

The deployed Keycloak SSO idle timeout and observed upstream refresh-grant
lifetime are 1,800 seconds. Inactivity can require fresh login. No idle policy,
grant, auth guardrail or token lifetime was relaxed for publication. Full
workspace validation and independent release review are not claimed.

The exception is bound to frozen runtime source `6195ca83e` and the exact binary
hashes in BUILD-INFO.json. Failed receipts are retained beside measured initial
workflow evidence. Default promotion still requires full lifecycle acceptance;
this explicit exception applies only to these alpha.14 binaries.

## Documentation

The Docusaurus architecture, workflow, identity and authentication lessons show
the two gateway routes and separate frontend/upstream OAuth lifecycles. The
obsolete architecture-correction announcement banner is removed. Public lessons
use fictional examples; operational receipts remain in this repository and the
internal npm packages.
