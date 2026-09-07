# Authentication release hands-on review

Run this after the operator confirms alpha.8 is installed and your Terminal roles
are enabled. Use the LAN/VPN that reaches `airs.cdot.io` and `auth.dev.cdot.io`.
The Linux terminal needs an unlocked Secret Service session. For a headless
shell, follow the [keyring setup](../../README.md#sign-in-with-keycloak) first;
keep login and the terminal in that same D-Bus session.
If you open another shell, unlock the same keyring in its D-Bus session and use
`airs-harness env list` to recover the literal environment name. Shell variables
from the previous session will not automatically exist there.

## Start an isolated user environment

```sh
airs-harness --version
# Expected: airs-harness 0.1.0-alpha.8

airs-harness setup --environment work-sso --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'
airs-harness login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot --audience airs-terminal-inference --device-auth
airs-harness setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp --audience airs-terminal-security \
  --tool pan_inline_scan --required --device-auth
airs-harness status
airs-harness doctor
```

If `work-sso` already exists, select it with `airs-harness env use work-sso`
instead of repeating setup. These commands use device login so the browser can
run on another machine while the terminal stays in this Linux workspace. Omit
`--device-auth` from both commands to use a browser on the same host. Sign in as
the same Keycloak user both times.
No workspace key or client secret is needed for these user logins.

## Exercise files, tools, routes and refresh

```sh
airs_review_dir=$(mktemp -d "$HOME/airs-harness-auth-review.XXXXXX")
cd "$airs_review_dir"
git init -q
airs-harness
```

Confirm the header shows the `work-sso` environment and verified user identity.
Use `/mcp` to check that `security` is connected, then submit:

> Create calculator.py with a multiply function and unittest tests for positive,
> negative and zero inputs. Run the tests and fix any failures. Then call
> pan_inline_scan with profile "Prisma AIRS Terminal" and response "Hello from
> AIRS Harness". Report actual test output, the scan action and scan_id.

Use `/model` to select `@openai-terminal-auth/gpt-4.1`, then ask it to review the
changes, rerun tests and perform another real scan. Switch back to **AI Gateway —
default**, leave the same process open for at least two minutes, and request a
third scan. All three turns should complete without a reasoning-parameter or
credential-expiry error. Default-route payloads omit `model`; explicit selection
preserves its qualified route. These wire properties are also checked by the
automated release fixtures.

## Check logout and resume

Exit the terminal, run `airs-harness logout`, and check `airs-harness status`.
Both inference and MCP credentials should be unavailable. The current IdP policy
uses a 15-minute idle timeout and a one-hour maximum session; reaching those
limits requires signing in again. Repeat the two login
commands above with the same user, then run `airs-harness resume` from the review
directory. The existing session should remain available under the same identity.

Do not paste access tokens, refresh tokens, browser authorization URLs or device
codes into a review report. Useful evidence is the version, environment name,
test output, scan IDs and any error message with credentials removed.
