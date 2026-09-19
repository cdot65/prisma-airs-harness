# Prepare an Ubuntu test host

Run [scripts/prepare_airs_ubuntu.sh](scripts/prepare_airs_ubuntu.sh) as your normal
SSH login user. It calls `sudo` for Ubuntu packages; npm and harness state remain
owned by your user. The script targets Ubuntu x86_64 and detects the installed
release. Ubuntu 26.04's Node.js package satisfies the harness requirement; an older
unsupported Node version stops preparation with an explicit error.

The npm package includes this guide and the script. On a machine with npm, once
`0.1.0-alpha.22.mcp.4` is available in the registry, download the package and
extract only the preparation script:

```bash
npm pack airs-harness@0.1.0-alpha.22.mcp.4 --registry=https://npm.cdot.io
tar -xOf airs-harness-0.1.0-alpha.22.mcp.4.tgz package/scripts/prepare_airs_ubuntu.sh > prepare-airs-ubuntu.sh
```

Review the script, then copy it to `~/prepare-airs-ubuntu.sh` on your Ubuntu host
(for example with `scp`). This download does not install or run the harness.
If you already have the package installed globally, find the same script at
`$(npm root -g)/airs-harness/scripts/prepare_airs_ubuntu.sh`. Run your copied
script in an interactive SSH terminal:

```bash
bash ~/prepare-airs-ubuntu.sh
source ~/.config/airs-test-host/env.sh
```

The helper defaults to `0.1.0-alpha.22.mcp.4` from `https://npm.cdot.io`,
matching this package. Run it once that exact version is available in the registry.
You can select another published version with `AIRS_TEST_VERSION`:

```bash
AIRS_TEST_VERSION=0.1.0-alpha.22.mcp.4 bash ~/prepare-airs-ubuntu.sh
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
The script does not replace the running daemon, reset a keyring, modify PAM,
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

Use `/mcp` inside the harness for gateway MCP connections. An SSH browser-login
callback may require an SSH port forward for the callback port shown by the
login flow. Do not share callback URLs, tokens or keyring passwords.

After reboot or keyring lock, use `bash ~/prepare-airs-ubuntu.sh --unlock`.
Use `bash ~/prepare-airs-ubuntu.sh --check` for checks without installing packages
or prompting for a keyring password. Both modes may create a disposable readiness
record and sandbox environment, which they clean up. They do not test real SSO,
workspace API-key authorization or ServiceNow access.
