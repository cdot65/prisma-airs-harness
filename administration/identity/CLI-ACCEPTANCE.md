# Terminal authentication acceptance

The installed client must pass the actual Keycloak/gateway flow before enabling
standing user access. `scripts/validate_auth_cli.py` is a deployment-specific
operator fixture. It creates two disposable users, temporarily enables the two
Terminal clients, tests native credential storage, and restores disabled clients
and scanner default-deny access in `finally`. Do not run concurrently with other
fixtures or against enabled clients with standing grants.

Prerequisites are the reviewed `talos-cluster` checkout containing the Terminal
refresh validation helper, operator Kubernetes access, the built Linux binary,
`uv`, a session D-Bus, and `gnome-keyring-daemon`. The `--management-module` is an
operator-owned ES module exporting an authenticated Prisma AIRS SDK client as
`gw`. It should read management credentials from the operator's protected CLI
configuration; do not embed tokens in source or command arguments. The client
itself does not require any of these operator dependencies.

```sh
umask 077
dbus-run-session -- uv run --locked scripts/validate_auth_cli.py \
  --binary /absolute/path/to/airs-terminal \
  --infrastructure-root /absolute/path/to/talos-cluster \
  --management-module /absolute/path/to/operator-management.mjs \
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
