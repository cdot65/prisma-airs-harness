# Alpha.20 MCP backports and release validation

Runtime/source: `26f373c87e11ec8ece6c895b28723fe7c4905aae`.
Base: Codex 0.154.0 plus the AIRS alpha.19 patches. This is a selective backport, not an upgrade to the full Codex 0.155 prerelease.

## Behavior

- Remote-terminal MCP login accepts the final browser callback URL through hidden terminal input. `airs-harness mcp login <configured-server> --no-browser` retains a live HTTP callback listener, so either delivery path can complete the same login. Redirect URI, state, issuer, PKCE and credential persistence remain validated. Cancellation restores the terminal and does not save credentials.
- OAuth startup failures appear accurately in MCP status snapshots.
- An OAuth authorization-server metadata response of HTTP 503 may fall back to OIDC discovery for the same issuer. Header boundaries and issuer checks remain enforced.
- Cancelled elicitation state is cleared when an MCP connection is re-established.

Inference and remote MCP still target AI Gateway; CAS owns upstream OAuth. The 30-minute idle timeout, secure credential storage and protection against replaying uncertain refresh transactions are preserved. Linux musl mixed IPv4/IPv6 DNS handling from alpha.19 is retained.

## Upstream provenance

Each change retains its original upstream attribution through a separate cherry-pick:

| OpenAI change | Upstream commit | AIRS commit |
| --- | --- | --- |
| [OAuth failure status](https://github.com/openai/codex/pull/44359) | `d996b4f02a204b36060de4711e106e8ecc7be9ac` | `3a6282b4e` |
| [Same-issuer OIDC fallback](https://github.com/openai/codex/pull/44636) | `8e2afc09126c0cea4c282725fe68af43adad73d7` | `1b41bec3d` |
| [Elicitation cancellation](https://github.com/openai/codex/pull/44238) | `3436cad5abbe9199c061880421b16d96a9ba702b` | `03043eda0` |
| [Manual MCP callback](https://github.com/openai/codex/pull/44629) | `f8ab57359dde6b6d5de1aee613c18fe60b661aeb` | `ac38b3ee4` |

The manual callback change was adapted to the AIRS OAuth context and existing single credential-save path. Enterprise OIDC changes and the separate upstream expired-token retry patch were not imported.

## Source validation

- `just test -p codex-rmcp-client -p codex-mcp`: 544 passed, 8 skipped.
- `just test -p codex-cli --test mcp_login --bin airs-harness --bin codex`: 783 passed.
- Callback protocol suite repeated with `AIRS_MCP_TEST_BINARY` selecting the actual harness: 5 passed.
- Native development fixture suite: 47 ran, 45 passed, 2 expected skips (npm-managed CLI and Mac Seatbelt).
- Launcher: 16 passed. Packaging: 13 passed. Registry contracts: 4 passed. Owned Mac workflow preflight contracts passed.
- `just fix -p codex-rmcp-client -p codex-mcp -p codex-cli` exited successfully with one existing version-header `expect()` warning and no source edits.
- `just fmt` passed. Bazel dependency lock update/check passed on Apple Silicon without lockfile drift.

## Installed artifact validation

- All three platform builds passed from the frozen source revision. The Mac Developer ID signature verified, notarization was Accepted, and the signed executable passed native Keychain checks.
- Linux x64 and ARM64 each passed five OAuth callback protocol cases and three real-terminal cases using the exact release executable. ARM64 execution used QEMU.
- Both Linux targets passed the three musl DNS/verified-HTTPS regressions: A with AAAA NXDOMAIN, A with AAAA NODATA, and both families NXDOMAIN with no HTTPS request.
- Fresh anonymous npm installations on Linux x64 and Apple Silicon each ran 47 native fixture tests: 46 passed and one platform-specific test was skipped. These include gateway routing, credential binding, tool execution, logout, and manual MCP callback behavior.
- Upgrades from alpha.19 and legacy alpha.14 passed on Linux x64 and Apple Silicon, preserving configuration and existing state.
- All four alpha.20 packages are published. `latest`, `alpha`, and `gateway-validation` resolve to alpha.20. Normal unversioned anonymous installations on Linux x64 and Mac returned `airs-harness 0.1.0-alpha.20` with matching native hashes. The Linux ARM64 optional package also installed with npm's platform override and passed its emulated version probe.

| Target | Native executable SHA256 |
| --- | --- |
| Linux x64 | `8b9f3b3568bd56f91476385f0c16308332cfee64fb3b63385592e35a46ac8318` |
| Linux ARM64 | `07d73f4ce2bbf28f55e65ef2152c96909be53565b7bcc15bda84f7d74720908b` |
| Apple Silicon, signed | `b7d0e44869068813a6f08b95bbdbd4102e297d8998588f40dce0bae6f583c3f4` |

The gateway acceptance helper has a separate tooling adjustment: it holds stdin open while waiting for a browser callback, so a headless parent does not cancel the new manual-input mode through EOF. The helper is not part of the distributed executable; the published runtime revision remains the one above.

## Owner acceptance

Publication is authorized for owner testing. Production Keycloak/AI Gateway/CAS acceptance and acceptance on the owner's actual Linux ARM64 VM remain outstanding; emulation is not presented as native-host acceptance. No production consent flow was opened while the owner was away. No Docusaurus files were modified.

Install with `npm install -g airs-harness --registry=https://npm.cdot.io`, then verify `airs-harness -V` reports `0.1.0-alpha.20`. Test inference and the local utility tools exposed through the configured gateway MCP endpoint. For a browser on a separate host, use manual MCP callback input in the initiating terminal; do not share callback URLs in chat. Keep the existing 30-minute idle policy when testing return-to-session sign-in behavior.
