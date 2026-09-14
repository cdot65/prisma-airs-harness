# Prisma AIRS Harness

A standalone local terminal agent derived from the open-source Codex Rust CLI.
Both inference and remote MCP traffic target Prisma AIRS AI Gateway. The native
Codex MCP client runs inside `airs-harness`; the gateway proxies upstream MCP
servers and manages their OAuth credentials. Files, shell commands, skills,
approvals and history remain under the local runtime. Selected file contents and
tool results become inference context.

The harness has no dependency on the hosted PAH application. Its npm launcher
includes the managed Prisma AIRS CLI/SDK for administration; those packages do
not implement the native MCP connection.

## Release status

Alpha.14 is the gateway correction. Both exact installed Linux x64 and signed
Apple Silicon candidates have passed production browser login, native storage,
all eight gateway-proxied tools, executable checks and npm upgrades. Two real
one-hour token-expiry cycles are running. Alpha.13 remains published until the
alpha.14 promotion gates pass; its direct-MCP onboarding is superseded.
See [MCP.md](MCP.md) and [PUBLICATION.md](PUBLICATION.md) for current evidence.
Native Windows distribution and independent release review are not claimed.

## Installation

Connect to the organization's LAN/VPN, then use a supported Node.js installation:
Node.js 22.13+ in the 22.x line, or 23.5+.

```sh
npm install -g airs-harness@latest --include=optional --registry=https://npm.cdot.io
airs-harness --version
```

Downloads are anonymous; no npm login or Rust compiler is required. At this
checkpoint `latest` is alpha.13; use the maintainer-provided alpha.14 candidate
for gateway acceptance. After promotion, the same command updates to alpha.14.
Existing npm installations need no uninstall or force. Inspect `command -v airs-harness` and `npm prefix -g` if an old manual command shadows npm; preserve
old binaries referenced by existing credential bindings.

The package supports Linux x64 and Apple Silicon. Intel Macs are unsupported.
Git, ripgrep and project tools remain prerequisites. Linux also needs Bubblewrap
and a kernel/container policy permitting its namespaces. The runtime fails
explicitly when its sandbox is unavailable. See [MACOS.md](MACOS.md) for Mac
onboarding. The inherited `scripts/install/` tools install upstream Codex.

## Inference login and gateway MCP onboarding

Keep an existing inference environment. Create one only for a new connection,
using the inference URL supplied by your administrator:

```sh
airs-harness setup --environment work --gateway-url https://gateway.example.com/v1
airs-harness --environment work login
airs-harness --environment work doctor --verify-access
```

Company sign-in uses the supplied OIDC issuer, public client ID and inference
audience. A workspace API key is a separate inference option; it is not a
Keycloak JWT and does not authorize the gateway's user MCP flow.

With alpha.14 and a provisioned gateway integration, add the **gateway MCP URL**:

```sh
airs-harness --environment work mcp add prisma-airs \
  --url https://gateway-mcp.example.com/prisma-airs/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
airs-harness --environment work mcp list
airs-harness --environment work
```

The scopes above match this deployment's gateway discovery. Complete CAS/company
SSO and any gateway-managed upstream consent. The gateway integration has its
own confidential upstream OAuth client. Do not put its client secret, upstream
client ID or upstream read scopes into the harness. CIE group membership must
map to the selected gateway workspace; upstream roles and object grants are
also required. Shared browser SSO does not make these credentials interchangeable.

Set `mcp_oauth_credentials_store = "keyring"` at the top level of the selected
environment's `config.toml` when native MCP file fallback is prohibited. Check
both CLI login completion and an actual tool call. `/mcp` shows the tool inventory.
See [MCP.md](MCP.md) for migration, refresh and separate logout behavior.

OIDC tokens require an unlocked native credential store. Linux needs a session
D-Bus and Secret Service (such as GNOME Keyring); installing a headless binary
does not create a desktop keyring session. macOS uses Keychain and Windows uses
Credential Manager. No plaintext refresh-token fallback is provided. Explicit
workspace-key file/environment modes remain available on headless hosts.

If a new terminal reports `OS credential store unavailable`, return to your
unlocked D-Bus shell or unlock the same keyring in a new session as described
below. Shell variables such as `$airs_e2e_env` do not carry into a new shell;
use `airs-harness env list` and the literal environment name when resuming.
Your named environments and conversation history remain on disk.

On a headless Linux host without an existing session bus, start an interactive
Bash session with `dbus-run-session -- bash`, then unlock Secret Service inside it:

```bash
set +x
read -r -s -p 'Keyring password (choose one on first use): ' airs_keyring_password
printf '\n'
if [ -n "$airs_keyring_password" ]; then
  printf '%s' "$airs_keyring_password" | gnome-keyring-daemon --unlock --components=secrets
fi
unset airs_keyring_password
```

Use a nonempty password you control. Run the Terminal login and coding commands
inside that same D-Bus session. On a later session, unlock with the same password;
refresh tokens remain encrypted on disk. This password is for your local keyring,
not your Keycloak account, and is never sent to AIRS. Desktop sessions normally
unlock their keyring through the OS login instead.

`airs-harness logout` signs out inference. Use `mcp logout prisma-airs` separately
for the native gateway MCP credential. Neither action proves immediate revocation
of a gateway-held upstream grant. Stop running sessions to discard cached access
tokens; issued credentials follow their own expiration and revocation policies.
Interrupted refresh requires signing in again rather than retrying a possibly
consumed token. A different user, issuer, gateway or resource needs a new
environment to preserve history isolation.

## Set up an environment

```sh
airs-harness setup --environment work \
  --gateway-url https://your-gateway.example/v1 \
  --model '@openai/gpt-4.1'

# Explicit headless authentication using an existing private file:
chmod 600 /absolute/path/to/workspace-key
airs-harness login --credential-file /absolute/path/to/workspace-key

airs-harness doctor
airs-harness
```

A bare hostname also works; setup supplies `https://` and `/v1`. The local context
budget defaults to **1,000,000 tokens**. `--context-window` overrides that budget
and the generated model capability entries. It cannot increase a provider's real
limit. Use a separate environment with an appropriate budget for a smaller model.

The default selection **AI Gateway — default** omits the root `model` key in
initial requests, tool continuations and local compaction requests. The gateway
chooses its default route. `-m '@provider/model'` or `/model` selects a configured
explicit entry and preserves that exact qualified route on the wire. The local
catalog describes capabilities; authorization and policy enforcement belong to
the gateway. Gateway requests select one tool call at a time because the
resolved model's parallel-call capability is not known to the local catalog.

Setup does not overwrite an environment. Each environment has an independent
UUID directory containing its configuration, model catalog, MCP state and history.

```sh
airs-harness env list
airs-harness env use work
airs-harness env show work
airs-harness --environment work exec 'Inspect this project and run its tests.'
airs-harness --environment work resume
```

State defaults to `~/.airs-harness`, or the absolute directory selected by
`AIRS_HARNESS_HOME`. If only `~/.airs-terminal` exists, the harness reuses it
in place. `AIRS_TERMINAL_HOME` remains a fallback override. Trusted project
configuration uses `.airs-harness/config.toml`, falling back to the legacy
`.airs-terminal` project directory when the new directory is absent.
The application does not change `HOME` or `CODEX_HOME`, and does not discover
`.codex` project settings. Endpoint, credential, capability and MCP settings must
come from the selected environment; project/profile/CLI overrides are rejected.

## Credentials and recovery

Choose one authentication source:

- `login --credential-file PATH` references an existing owner-only regular file.
  The application stores the path and a credential fingerprint, without copying
  the key into configuration. Symlinks and public files are rejected.
- `login --credential-env NAME` references a variable supplied by your secret
  manager. Its value is excluded from local tool subprocess environments.
- Pipe a workspace key into `login --with-api-key` to use the OS credential store.
  An unavailable store produces an error; there is no automatic plaintext fallback.

`status` performs local checks. `doctor --json` additionally makes a bounded,
unauthenticated API-root `/health` request (for example `/v1/health`) to the selected gateway and checks local tool
availability and, on Linux, actual Bubblewrap namespace creation. It does not
submit inference, and distinguishes local availability from remote authorization.

`logout` and interactive `/logout` disable new inference credential use in the
selected environment until login. Native MCP has a separate `mcp logout` command. Referenced key files and external
variables remain owned by the user. Stop other running sessions to discard their
cached credentials. Revoke workspace keys through AIRS when server-side revocation
is needed.

The first agent startup pins the environment's destination, credential identity,
capability catalog, context budget and MCP configuration. A changed key, catalog
or explicit MCP credential binding requires a new environment, so existing
history cannot silently move to a different identity or capability revision.
Native HTTPS MCP OAuth configuration is excluded from the inference binding;
adding a gateway connection preserves that inference history. Model choices within the pinned catalog remain selectable.
A running process keeps its environment even when another process changes the
default. Resume instructions include the environment name.

## Remote MCP

Use the native gateway workflow above. The earlier `setup-mcp` header helper and
separate MCP candidate executable are historical paths, not this integration.
The gateway-facing MCP access token in the tested deployment is opaque and lasts
one hour. The gateway separately holds the five-minute upstream Keycloak JWT and
its refresh grant. All tool requests still pass through the gateway after refresh.

`mcp login prisma-airs --no-browser` presents the native authorization URL. On a
remote host, the browser must reach the login process's loopback callback through
an explicitly forwarded port. The tested native callback expires after five
minutes. Device authorization for inference is a separate feature; do not assume
that the gateway MCP OAuth endpoint supports it.

Local files and skills are accessed with local tools. Place a skill's `SKILL.md`
in `.agents/skills/<name>/` in a project, or the selected environment's `skills`
directory. Invoke it with `$name`. Use `/compact` to summarize a long thread,
Escape to interrupt a turn, and `resume` to continue persisted work.

## Build and validate

Building the Rust terminal requires a source checkout of this repository. The
binary archive also includes its acceptance scripts, redacted evidence and the
optional scanner backend source/Dockerfile. Rust 1.95.0 is pinned. Read [AGENTS.md](AGENTS.md) for prerequisites and conventions.
The Linux runtime needs a usable shell and ordinary project tools such as Git and
ripgrep; Linux also needs Bubblewrap (`bwrap`). The sandbox preserves kernel/filesystem/network restrictions; it does
not automatically fall back to unrestricted execution.

```sh
cd codex-rs
cargo build --locked --release -p codex-cli --bin airs-harness
./target/release/airs-harness --version
```

From the repository root, executable fixtures use Python's standard library:

```sh
AIRS_HARNESS_BIN=codex-rs/target/release/airs-harness \
  python3 -m unittest discover -s scripts -p 'test_airs_harness*.py' -v
```

Rust checks use `just test`. The live contract probe is
`scripts/validate_live_gateway.py --help`; it reads explicit credential files,
submits small acceptance fixtures, and writes a redacted result. See
[RELEASE.md](RELEASE.md) for platform-specific test limitations and install receipts.

## Upstream and license

Pinned upstream: Codex `rust-v0.153.4`, commit
`3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`. Internal crate names and versions stay
unchanged for reviewable upstream updates. See [BASELINE.json](BASELINE.json) and
[UPSTREAM.md](UPSTREAM.md) for the fork boundary.

Codex-derived code remains Apache-2.0. Preserve [LICENSE](LICENSE), [NOTICE](NOTICE),
dependency notices and upstream history. The product does not imply OpenAI
endorsement. [README.upstream.md](README.upstream.md) preserves the original intro;
[IMPLEMENTATION.md](IMPLEMENTATION.md) records historical prototype work.

GitHub Packages is an additional distribution channel; see [GitHub installation and publication](PUBLICATION.md#github-packages-distribution) for authenticated installation without redirecting the public CLI dependencies.
