# Prisma AIRS Harness

A standalone local terminal agent derived from the open-source Codex Rust CLI.
Both inference and remote MCP traffic target Prisma AIRS AI Gateway. The native
Codex MCP client runs inside `airs`; the gateway proxies upstream MCP
servers and manages their OAuth credentials. Files, shell commands, skills,
approvals and history remain under the local runtime. Selected file contents and
tool results become inference context.

The harness has no dependency on the hosted PAH application. Its npm launcher
includes the managed Prisma AIRS CLI/SDK for administration; those packages do
not implement the native MCP connection.

## Release status

Alpha.22 introduces `airs` for the harness and `airs cli ...` for its bundled
Prisma AIRS CLI 7.0.0. The standalone product executable is `airs-cli`.
See [the command migration guide](PRISMA-AIRS-CLI.md) for install order and tenant
onboarding. [Alpha.21 release checks](validation/2026-09-17/alpha21-environments/README.md) remain historical evidence.

Alpha.21 consolidates environment creation and lifecycle management under
`env create/list/show/status/use/rename/remove`. Top-level `setup` and `status`
are removed. Existing environment credentials and history remain intact.
`env use` saves the default; `--environment` selects one command's environment.

The release retains gateway inference and MCP routing, manual MCP callback
input, and guided sign-in recovery. The deployed 30-minute SSO idle policy is
unchanged. Fresh production SSO, ServiceNow tool calls, and hourly frontend
refresh are not claimed by this environment-management release.

## Branded onboarding

Release `0.1.0-alpha.22.onboarding.2` uses the cyan Prisma AIRS mark with a subtle shimmer in the animated welcome. Environment creation/selection and guided sign-in remain available. The npm distribution targets Linux x64, Linux ARM64 and signed Apple Silicon at `https://npm.cdot.io`.

```sh
npm install -g airs-harness@0.1.0-alpha.22.onboarding.2 --registry=https://npm.cdot.io
airs --version
airs cli --version
airs
```

The product CLI and eight skills are bundled. Normal installs do not require `--include=optional`. Returning signed-in users enter the agent directly. [Release evidence](validation/2026-09-18/prisma-logo/README.md) records native acceptance and remaining attended production checks. The earlier review archive on [PR 47](https://git.cdot.io/cdot/prisma-airs-harness/pulls/47) remains available as a separate installation path.

## Installation

Connect to the organization's LAN/VPN, then use a supported Node.js installation:
Node.js 22.14+ in the 22.x line, or Node.js 24 or newer.

If the old standalone Prisma AIRS CLI already owns `airs`, upgrade it **first**:

```sh
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org
airs-cli --version
```

A fresh machine needs only the harness installation below. It includes CLI 7.0.0
as `airs cli` and the Prisma AIRS product skills.

```sh
npm install -g airs-harness@0.1.0-alpha.22.onboarding.2 --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Downloads are anonymous; no npm login or Rust compiler is required.
The exact version above includes the unified environment commands.
If an old standalone product CLI owns `airs`, upgrade it to
`@cdot65/prisma-airs-cli@7.0.1` first so it exports `airs-cli`, then install the
harness. Inspect `type -a airs airs-cli airs-harness` and `npm prefix -g` for
conflicting aliases or manual installs. Preserve existing credential bindings.
See [the command migration and bundled CLI guide](PRISMA-AIRS-CLI.md).
The `airs-harness` compatibility launcher remains in alpha.22 and is scheduled
for removal in alpha.23.

The package supports Linux x64, Linux ARM64 and Apple Silicon. Intel Macs are
unsupported. Linux ARM64 is cross-compiled by the `airs-harness-linux-arm64.yml`
workflow; release acceptance runs its installed package natively in Jadzia's
ARM64 Linux VM before publication.
Git, ripgrep and project tools remain prerequisites. Linux also needs Bubblewrap
and a kernel/container policy permitting its namespaces. The runtime fails
explicitly when its sandbox is unavailable. See [MACOS.md](MACOS.md) for Mac
onboarding. The inherited `scripts/install/` tools install upstream Codex.

## Sign in with SSO and connect ServiceNow

The [complete SSO-to-ServiceNow walkthrough](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/#sso-to-servicenow-a-complete-first-session)
covers one user, one environment, two company-SSO authorizations, and a real
read-only incident result. The commands below use example connection settings;
obtain the actual gateway URL, issuer, public client ID and audience from your
administrator. The account needs inference access, the gateway workspace grant,
and the ServiceNow MCP subject permissions.

The unified environment CLI requires alpha.21 or newer. Check
`airs env create --help` after upgrading. An existing environment for the correct gateway can be reused with
`env use work`; do not recreate it after a cancelled login.

Create the environment, then sign into inference as your company user:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs env use work
airs --environment work login \
  --issuer-url https://sso.example.com/realms/company \
  --oidc-client-id harness-native \
  --audience airs-inference
airs env status work
airs --environment work doctor --verify-access
```

Alternatively, `env create work` without the gateway flag guides both creation
and sign-in: choose **Company sign-in**. Wait for terminal confirmation that the
credential was saved. `env status` reports local configuration; the doctor probe
checks inference and can consume gateway quota. Neither verifies ServiceNow.

Use `env show work` to locate the environment's `config.toml`. Set
`mcp_oauth_credentials_store = "keyring"` at the top level, before any table
headers, so MCP also requires native credential storage. Then add the gateway's
ServiceNow connection, including the final `/mcp`:

```sh
airs --environment work mcp add service-now \
  --url https://gateway-mcp.example.com/mcp-service-now-dev/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
```

Adding the connection normally starts browser authorization. Complete the gateway
CAS/company SSO flow with the **same company account** used for inference, and
any gateway-managed upstream consent. An existing browser SSO session can avoid
another password prompt, but the grants remain separate. If login was cancelled
or failed after registration, resume it with:

```sh
airs --environment work mcp login service-now
```

Skip that extra login if `mcp add` already reported success. The connection uses
the gateway URL, not the ServiceNow instance or upstream MCP URL. The gateway
owns upstream OAuth; the ServiceNow MCP server uses a server-side integration
credential. The user does not supply an upstream client secret or ServiceNow
integration password to the harness.

```sh
airs --environment work mcp list
airs --environment work
```

Inside the harness, use `/mcp` to inspect `service-now`, then ask:

> Use service-now to list up to five active incidents, showing their numbers,
> short descriptions and priorities. Do not create or update records.

Verify an actual `list_incidents` tool result; an empty authorized list is valid.
A connection label alone is not end-to-end proof. Read-only users see
`list_incidents` and `get_incident`; management grants also expose `create_incident`
and `update_incident`. This example targets a development integration.
See [MCP.md](MCP.md) for migration, refresh and separate logout behavior.

OIDC tokens require an unlocked native credential store. Linux needs a session
D-Bus and Secret Service (such as GNOME Keyring); installing a headless binary
does not create a desktop keyring session. macOS uses Keychain and Windows uses
Credential Manager. No plaintext refresh-token fallback is provided. Explicit
workspace-key file/environment modes remain available on headless hosts.

If a new terminal reports `OS credential store unavailable`, return to your
unlocked D-Bus shell or unlock the same keyring in a new session as described
below. Shell variables such as `$airs_e2e_env` do not carry into a new shell;
use `airs env list` and the literal environment name when resuming.
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

`airs logout` signs out inference. Use `mcp logout service-now` separately
for the native gateway MCP credential. Neither action proves immediate revocation
of a gateway-held upstream grant. Stop running sessions to discard cached access
tokens; issued credentials follow their own expiration and revocation policies.
Interrupted refresh requires signing in again rather than retrying a possibly
consumed token. A different user, issuer, gateway or resource needs a new
environment to preserve history isolation.

## Manage environments

Use `airs env` for the environment lifecycle. With no action it lists
saved environments and marks the default. For guided onboarding, run:

```sh
airs env create work
airs
```

The wizard collects the inference gateway URL, creates and selects the named
environment, then starts sign-in. If sign-in is interrupted, resume with
`airs --environment work login`; do not create the environment again.
Check access with `airs --environment work doctor --verify-access`.
Configure gateway MCP using the onboarding steps above.

`env create` selects the new environment, so subsequent commands need no
environment flag. Use `env use NAME` to change the saved default. The optional
`--environment NAME` flag overrides that default for just one command; for
example, `airs --environment staging doctor` leaves your default alone.

For automation, supplying `--gateway-url` creates the environment without
prompting or starting sign-in:

```sh
airs env create work \
  --gateway-url https://your-gateway.example/v1 \
  --model '@openai/gpt-4.1'

# Explicit headless authentication using an existing private file:
chmod 600 /absolute/path/to/workspace-key
airs login --credential-file /absolute/path/to/workspace-key

airs doctor
airs
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

Creation does not overwrite an environment. Each environment has an independent
UUID directory containing its configuration, model catalog, MCP state and history.

```sh
airs env list
airs env use work
airs env show
airs env status
airs env rename work team
airs env use team
airs --environment team exec 'Inspect this project and run its tests.'
airs --environment team resume
```

`env show [NAME]` inspects the named or selected environment; `env status [NAME]`
reports its gateway and local credential availability. Renaming preserves
its UUID, credentials and history. `env remove NAME` unregisters the environment
and preserves its files; it does not sign out or revoke credentials. Sign out
first if needed. Removing the default requires selecting another with `env use`.
Recreating a removed name creates a fresh, isolated environment.

The top-level `setup` and `status` commands have been removed. Use `env create`
and `env status` instead. New environments are always named. Creating an
environment selects it for new processes; running sessions keep their existing
environment.

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
