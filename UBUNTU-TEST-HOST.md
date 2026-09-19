# Prepare an Ubuntu test host

Run [scripts/prepare_airs_ubuntu.sh](scripts/prepare_airs_ubuntu.sh) as your normal
SSH login user. It calls `sudo` for Ubuntu packages; npm and harness state remain
owned by your user. The script targets Ubuntu x86_64 and detects the installed
release. Ubuntu 26.04's Node.js package satisfies the harness requirement; an older
unsupported Node version stops preparation with an explicit error.

The mcp.5 npm package includes this corrected guide and helper. It fixes the
competing-keyring-daemon problem in the mcp.4 helper, verifies the default
collection's lock state, and prints the actual script path for later unlocks.
On a machine with npm, download the published package:

```bash
npm pack airs-harness@0.1.0-alpha.22.mcp.5 --registry=https://npm.cdot.io
tar -xOf airs-harness-0.1.0-alpha.22.mcp.5.tgz package/scripts/prepare_airs_ubuntu.sh > prepare-airs-ubuntu-mcp5.sh
```

Review the script, then copy it to `~/prepare-airs-ubuntu-mcp5.sh` on your Ubuntu host
(for example with `scp`). This download does not install or run the harness.
Run the corrected copied script in an interactive SSH terminal:

```bash
bash ~/prepare-airs-ubuntu-mcp5.sh
source ~/.config/airs-test-host/env.sh
```

The helper defaults to `0.1.0-alpha.22.mcp.5` from `https://npm.cdot.io`,
matching this package.
You can select another published version with `AIRS_TEST_VERSION`:

```bash
AIRS_TEST_VERSION=0.1.0-alpha.22.mcp.5 bash ~/prepare-airs-ubuntu-mcp5.sh
```

The script installs Node.js/npm, Git/ripgrep, Bubblewrap, D-Bus/Secret Service
utilities, Python/venv and HTTPS tools through apt. It installs the harness in
`~/.local/share/airs-test-host/npm`, appends one guarded environment-file source
line to your Bash login startup file, and leaves existing npm configuration and
harness environments intact. The explicit optional-dependency flag makes this
preparation reproducible even if npm configuration omits optional packages; it
is not required for ordinary installs with npm defaults.

SSH key authentication does not supply a password to unlock an encrypted login
keyring. The script reuses your user D-Bus session and prompts locally to unlock
an existing keyring or choose a nonempty password for a new one. Remember that
password. It is not a gateway API key or an SSO password. The password goes to the
keyring daemon through stdin and is never recorded in the readiness report.
GNOME describes login/session startup and passwordless login behavior in its
[daemon documentation](https://wiki.gnome.org/Projects/GnomeKeyring/RunningDaemon).
When unlock is needed, the script replaces the locked Secret Service daemon so
D-Bus clients reach the unlocked instance, then checks the collection lock state.
An already unlocked service is reused. It does not reset a keyring, modify PAM,
disable AppArmor or enable plaintext credential fallback.

A successful `READY` report proves:

- The exact installed package and bundled product CLI launch.
- A disposable environment's read-only harness sandbox reads a canary and denies
  modification, while the owner's saved environments remain untouched.
- The registry package and gateway are reachable over verified HTTPS. A gateway
  HTTP 401 is expected without credentials and is not a sign-in failure.
- Native Secret Service can write, read and remove a uniquely named dummy record.

Reports live under `~/.local/state/airs-test-host/`. A failed check exits nonzero.
No production sign-in is attempted. After readiness:

```bash
command airs env create
command airs doctor --verify-access
command airs
```

For company inference SSO without a browser on Ubuntu, use **Use device authorization**
or `airs --environment NAME login --device-auth`. Open the verification link and
enter the code in a browser on your laptop or phone; keep SSH open while AIRS
polls. The company issuer must enable this grant for the harness client. Device
login needs no callback port or SSH tunnel. Follow [the SSH walkthrough](GETTING-STARTED.md#browserless-sign-in-over-ssh)
to use a separate SSO environment while preserving an existing workspace-key profile.

Gateway MCP authorization is separate. Inside `/mcp`, open its authorization URL
on the browser host and paste the full callback into the hidden callback input,
even if the browser cannot load that localhost page. Do not paste callbacks into
the agent conversation. Inference `login --no-browser` is a different flow from
`--device-auth` and still needs its callback to reach Ubuntu.

After reboot or keyring lock, use `bash ~/prepare-airs-ubuntu-mcp5.sh --unlock`.
Use `bash ~/prepare-airs-ubuntu-mcp5.sh --check` for checks without installing packages
or prompting for a keyring password. Both modes may create a disposable readiness
record and sandbox environment, which they clean up. They do not test real SSO,
workspace API-key authorization or ServiceNow access.
