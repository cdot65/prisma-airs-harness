# Browserless SSO readiness — September 19, 2026

The existing published mcp.5 supports inference device authorization through `login --device-auth` and the welcome screen's **Use device authorization** action. No runtime change or new npm version was needed for this preflight. The Ubuntu host's resolved installed command reports mcp.5.

`ISSUER-UBUNTU-PROBE.json` records an HTTPS probe from the actual Ubuntu host. The company issuer accepted the public harness client's device request with the same openid scope and S256 challenge fields used by the harness. The first token poll, after the advertised interval, returned `authorization_pending`. Sensitive device/user codes and the PKCE verifier existed only in the probe process; no bearer token was obtained or retained.

This validates device-grant initiation for this client and network path. It does **not** validate a human login, token claims, native credential persistence, gateway inference, renewal or MCP access. The probe was a separate HTTP client, not an installed-harness end-to-end run. The working owner environment and its credential binding were not changed. Device requests expire naturally without approval.

The [getting-started guide](../../../GETTING-STARTED.md#browserless-sign-in-over-ssh) now supplies an explicit SSH command, a separate SSO profile that preserves the workspace-key profile, and the distinction between inference device authorization and the gateway MCP hidden-callback flow. The next acceptance step is attended company sign-in from another device, followed by native persistence and actual gateway verification in the selected SSO environment. No account passwords should be shared with the agent.

The owner declined in-session workspace API-key replacement; it is not part of this work.

## Attended follow-up passed

The owner completed device approval in a separate browser. `ATTENDED-ACCEPTANCE.json` binds successful installed-harness login, native credential storage and two subsequent separate-process gateway inference checks to the exact published Linux x64 binary. Both checks passed with Company SSO and no second login. The first unapproved device attempt expired; a fresh attempt completed.

`ATTENDED-CLEANUP.json` records successful logout of only the isolated test profile, cleared binding, no pending cleanup and confirmed issuer refresh-token revocation. The normal owner profiles were not used or replaced. This closes the human-approval/native-storage/inference gap described in the earlier preflight above, without claiming MCP authorization or long-duration renewal. No runtime source or npm version changed.
