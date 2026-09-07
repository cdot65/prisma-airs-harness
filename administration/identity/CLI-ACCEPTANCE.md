# Terminal authentication acceptance

The installed client must pass the actual Keycloak/gateway flow before enabling
standing user access. `scripts/validate_auth_cli.py` is a deployment-specific
operator fixture. It creates two disposable users, temporarily enables the two
Terminal clients, tests native credential storage, and restores disabled clients
in `finally`. It uses the deployed scanner access policy unchanged. Do not run concurrently with other
fixtures or against enabled clients with standing grants.

Prerequisites are the reviewed `talos-cluster` checkout containing the Terminal
refresh validation helper, operator Kubernetes access, the built Linux binary,
`uv`, a session D-Bus, and `gnome-keyring-daemon`. The client itself does not require these operator dependencies.
No management OAuth credential or scanner-permission update is needed for this
fixture. The full mode additionally keeps a TUI open across access-token expiry
and requires successful inference and MCP calls in that same process. Its 27
checks include a real default → explicit → default model switch under OIDC,
empty MCP inventories and a four-command budget for the simple
file task. The budget is an acceptance check, not a general runtime command cap.
`--observe-tools` is an optional diagnostic relay that records only tool names,
item counts and model-key presence; final release acceptance uses the direct
TLS endpoints without this relay.
`--credentials-only` is a narrower diagnostic mode and cannot certify agent/MCP
execution. Receipt success requires every phase to finish and records the binary
SHA-256; an interrupted run never becomes a passing receipt.

```sh
umask 077
dbus-run-session -- uv run --locked scripts/validate_auth_cli.py \
  --binary /absolute/path/to/airs-terminal \
  --infrastructure-root /absolute/path/to/talos-cluster \
  --output /private/cli-oidc-acceptance.json
```

The receipt contains only validation status and public subject/binding facts.
The sibling `.private-exec.log` contains the synthetic coding/tool session for
operator diagnosis; do not publish it blindly. Passwords, JWTs, authorization
codes and browser login URLs stay in memory. Cleanup revokes the synthetic users'
sessions by deleting the users. The temporary native store is encrypted and
removed when the fixture exits.

The portable native-store fixture is separate:

```sh
python scripts/validate_native_credentials.py \
  --binary /absolute/path/to/store_acceptance \
  --output /private/native-store.json
```

That test uses disposable values and needs no IdP or gateway credentials. Linux
starts an isolated Secret Service session; macOS and Windows use a random account
in the current user's unlocked native store. All three native jobs passed in
[run 34147583219](https://github.com/cdot65/prisma-airs-terminal/actions/runs/34147583219).

Persisted attribution is independently verified through the existing AIRS
security-log export, including both input and output scans and a caller metadata
spoofing attempt. The read-only `scripts/verify_persisted_airs_audit.py` requires a
local Elasticsearch port-forward and reads the operator credential in memory.
It emits only known synthetic subject/trace/scan IDs and verdicts. Management connectivity recovered at 19:30 UTC. A separate SCM request-telemetry
check matched all three subjects and trace IDs, including the spoofing case,
with successful response status and populated cost/usage fields. See
`validation/2026-09-07/auth-release/persisted-management-audit.json`.
The earlier timeout and unsupported native gateway log-read attempts remain
historical evidence.


For SCM request attribution, `scripts/verify_management_airs_audit.mjs` accepts an
operator-owned module exporting the authenticated Prisma AIRS SDK client as `gw`.
The terminal runtime does not depend on this management module or SDK. Supply
only known synthetic subject/trace pairs, including the metadata-spoofing case:

```sh
node scripts/verify_management_airs_audit.mjs \
  --management-module /absolute/private/management.mjs \
  --expectations validation/2026-09-07/auth-release/audit-expectations.json \
  --workspace ws-prisma-ff3d74 --output /private/management-audit.json
```

The check reads exactly those traces and emits only identifiers and validation
booleans. It does not export prompts, responses, credentials or unrelated users.
