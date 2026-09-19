# Prepare an Ubuntu test host

Run [scripts/prepare_airs_ubuntu.sh](scripts/prepare_airs_ubuntu.sh) as your normal
SSH login user. It calls `sudo` for Ubuntu packages; npm and harness state remain
owned by your user. The script targets Ubuntu x86_64 and detects the installed
release. Ubuntu 26.04's Node.js package satisfies the harness requirement; an older
unsupported Node version stops preparation with an explicit error.

```bash
bash ~/prepare-airs-ubuntu.sh
source ~/.config/airs-test-host/env.sh
```

The default package is the published `0.1.0-alpha.22.mcp.3` from
`https://npm.cdot.io`. This does not select an unpublished development build.
To prepare a later published version, supply its exact version:

```bash
AIRS_TEST_VERSION=0.1.0-alpha.22.mcp.3 bash ~/prepare-airs-ubuntu.sh
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
