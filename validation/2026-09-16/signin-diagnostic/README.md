# Alpha.20 company sign-in diagnostic

The owner reported successful gateway MCP login and repeated inference login failure with `access JWT identity or lifetime mismatch` on Linux ARM64. The failing host and HTTP Date agreed to the second. Both production Keycloak replicas agreed with the local clock. The client has no access-token lifespan override; the realm access lifespan is 900 seconds and SSO idle timeout remains 1800 seconds. Evaluated sample tokens for Calvin matched the configured client, subject, and 900-second lifespan. These samples are not the owner's failed token response.

Source ef365e990 separates the four existing checks without changing validation thresholds. All 28 codex-airs-identity tests passed, including signed wrong-client/subject, future-issued, and insufficient-remaining-lifetime cases. `just fmt` completed; unrelated formatter changes were restored.

The `login_diagnostic` example uses the same Provider discovery, browser PKCE flow, and verification as the harness. It outputs authorization instructions and status only, does not save credentials, and does not refresh or revoke the shared session. Discarded diagnostic tokens are not printed or written to disk. The package wrapper selects one bundled executable by platform and architecture.

Published `airs-harness-diagnostics@0.1.0-alpha.20` to https://npm.cdot.io under the `signin` tag. Anonymous download was checked against each staged binary hash. Linux x64 and Apple Silicon Mac startup passed natively; Linux ARM64 startup passed under QEMU. Actual Linux ARM64 sign-in and the underlying failure diagnosis await the owner. This is not a harness release or evidence that production login has been fixed.

Run:

```sh
npx --yes --registry=https://npm.cdot.io --package=airs-harness-diagnostics@0.1.0-alpha.20 airs-harness-signin-diagnostic
```

Open the displayed URL on that machine and complete sign-in within five minutes. Share only the final Error or PASS line. No Docusaurus content was modified.
