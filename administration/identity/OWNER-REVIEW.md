# Authentication release hands-on review

Run this after the operator confirms alpha.6 is installed and your Terminal roles
are enabled. Use the LAN/VPN that reaches `airs.cdot.io` and `auth.dev.cdot.io`.
The Linux terminal needs an unlocked Secret Service session. For a headless
shell, follow the [keyring setup](../../README.md#sign-in-with-keycloak) first;
keep login and the terminal in that same D-Bus session.

## Start an isolated user environment

```sh
airs-terminal --version
# Expected: airs-terminal 0.1.0-alpha.6

airs-terminal setup --environment work-sso --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'
airs-terminal login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot --audience airs-terminal-inference
airs-terminal setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp --audience airs-terminal-security \
  --tool pan_inline_scan --required
airs-terminal status
airs-terminal doctor
```

If `work-sso` already exists, select it with `airs-terminal env use work-sso`
instead of repeating setup. Add `--device-auth` to both login commands when using
a browser on another machine. Sign in as the same Keycloak user both times.
No workspace key or client secret is needed for these user logins.

## Exercise files, tools, routes and refresh

```sh
airs_review_dir=$(mktemp -d "$HOME/airs-terminal-auth-review.XXXXXX")
cd "$airs_review_dir"
git init -q
airs-terminal
```

Confirm the header shows the `work-sso` environment and verified user identity.
Use `/mcp` to check that `security` is connected, then submit:

> Create calculator.py with a multiply function and unittest tests for positive,
> negative and zero inputs. Run the tests and fix any failures. Then call
> pan_inline_scan with profile "Prisma AIRS Terminal" and response "Hello from
> AIRS Terminal". Report actual test output, the scan action and scan_id.

Use `/model` to select `@openai-terminal-auth/gpt-4.1`, then ask it to review the
changes, rerun tests and perform another real scan. Switch back to **AI Gateway —
default**, leave the same process open for at least two minutes, and request a
third scan. All three turns should complete without a reasoning-parameter or
credential-expiry error. Default-route payloads omit `model`; explicit selection
preserves its qualified route. These wire properties are also checked by the
automated release fixtures.

## Check logout and resume

Exit the terminal, run `airs-terminal logout`, and check `airs-terminal status`.
Both inference and MCP credentials should be unavailable. The current IdP policy
uses a 15-minute idle timeout and a one-hour maximum session; reaching those
limits requires signing in again. Repeat the two login
commands above with the same user, then run `airs-terminal resume` from the review
directory. The existing session should remain available under the same identity.

Do not paste access tokens, refresh tokens, browser authorization URLs or device
codes into a review report. Useful evidence is the version, environment name,
test output, scan IDs and any error message with credentials removed.
