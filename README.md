# Prisma AIRS Harness

TypeSafe Jev preview: [install the harness with bundled CLI 7.1.4](RELEASE-TYPESAFE-JUDGE.md).

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

**0.1.1 is the reliability and onboarding release.** It consolidates environment
onboarding, company SSO and workspace API keys, in-session MCP sign-in and
connection management, and recovery diagnostics. Desktop MCP sign-in opens the
browser; remote sign-in supports hidden callback input. New environments use
native MCP credential storage; existing environments retain their configured mode.

The owner confirmed inference sign-in, ServiceNow sign-in, a read-only query and
credential reuse after restarting the preceding mcp.6 release on Apple Silicon.
Stable publication requires fresh installed checks of the actual 0.1.1 packages
on Linux x64, Linux ARM64 and signed/notarized Apple Silicon. See
[the release notes](RELEASE-0.1.1.md) and
[the preceding release evidence](validation/2026-09-19/mcp-signin-polish/README.md).

The npm package remains `airs-harness`; the command is `airs`. The bundled product
CLI is `airs cli ...`; the standalone product CLI uses `airs-cli`. A temporary
`airs-harness` compatibility alias remains for existing installations.

Environment lifecycle commands live under `env`: create, list, show, status, use,
rename and remove. `env use` saves the default; `--environment` selects an
environment for one command. The old top-level `setup` and `status` are removed.

## Getting started

Follow [Getting started](GETTING-STARTED.md) for installation, local environments,
company SSO or a workspace API key, then ServiceNow sign-in through the in-session
`/mcp` manager. `/doctor` inspects connection health and offers explicit access
verification. MCP connection changes start a new conversation inside AIRS while
preserving history and the draft.

For 0.1.1 use the exact version below. After stable promotion, an ordinary
unversioned install selects it through `latest`; prerelease channel tags remain
separate. Upgrading preserves existing environments, credentials and history.

## Installation

Connect to the organization's LAN/VPN, then use a supported Node.js installation:
Node.js 22.13.0 or newer in the 22.x line, or 23.5.0 or newer
(`^22.13.0 || >=23.5.0`). Installing npm alone does not upgrade Node.js; check
`node --version` and `npm --version` in the same terminal first.

If the old standalone Prisma AIRS CLI already owns `airs`, upgrade it **first**:

```sh
npm install -g @cdot65/prisma-airs-cli@7.0.1 --registry=https://registry.npmjs.org
airs-cli --version
```

A fresh machine needs only the harness installation below. It includes CLI 7.1.4
as `airs cli` and the Prisma AIRS product skills.

```sh
npm install -g airs-harness@mcp --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Downloads are anonymous; no npm login or Rust compiler is required.
The test channel includes the unified environment commands and in-session
connection dashboards. Check the installed version before reporting a result.
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
onboarding and the optional [Ubuntu test-host preparation guide](UBUNTU-TEST-HOST.md)
for an Ubuntu SSH host. The mcp.5 helper includes the September 19 SSH
keyring-daemon repair; mcp.4 users need the corrected separately supplied helper. The inherited `scripts/install/` tools install upstream Codex.

## Sign in and connect ServiceNow

The [current first-session guide](GETTING-STARTED.md) covers both company SSO and
workspace API keys, when to create an environment, and the separate MCP login.
Local environment names do not need to match AI Gateway workspace names. A saved
workspace key is not proof of inference access and does not grant MCP access.

New environments created by mcp.4 and later already require native MCP storage. For an
existing environment or mcp.3 installation, follow the guide's storage check
before sign-in. Open `airs --environment work`, then use
**Add connection** in `/mcp` with the administrator's **AI Gateway MCP URL**. Follow
the company sign-in and gateway-managed upstream consent, then choose **Start new
conversation**. The guide explains cached inventory versus an actual authorized
read-only tool result, as well as `/doctor` and the optional shell recovery path.
No attended production ServiceNow result is claimed by this documentation update.

The [published learning site](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/)
explains the identity and gateway boundaries; its source is maintained in the
project knowledge vault. See [MCP.md](MCP.md) for the longer technical lifecycle
and historical validation context.

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

On an Ubuntu SSH host, follow [the corrected host preparation guide](UBUNTU-TEST-HOST.md).
Use its reviewed helper to unlock the existing Secret Service collection and
verify native credential write/read/delete. The helper bundled in mcp.4 predates
the SSH daemon correction; use mcp.5 or the corrected administrator copy.
Run AIRS in the same user D-Bus session. On other Linux distributions, use the
platform's credential-service unlock procedure and verify access with `airs doctor`.

Use a nonempty local keyring password you control. It is not your company SSO
password and is never sent to AIRS. Desktop sessions normally unlock their
keyring through OS login; SSH public-key authentication does not do so.

`airs logout` signs out inference. Use `airs --environment work mcp logout service-now` separately
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

In [preview 0.1.2-alpha.3.mcp.1](RELEASE-TYPESAFE-JUDGE.md), setup also offers an
optional TypeSafe judge key for red-team ASR scoring. Bind or
inspect it later with `airs env typesafe set|status|clear`. The skill uses
`airs --environment NAME env typesafe exec -- COMMAND` to pass the key only to
its child process; existing shell variables take precedence. See
[GETTING-STARTED.md](GETTING-STARTED.md#optional-typesafe-judge-key-for-red-team-scoring).

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

## Workspace API keys and environments

An environment is a local profile for one gateway connection, its credentials,
MCP configuration and session history. Create one for a separate connection or
identity; reuse it for later sessions. Its name does **not** need to match an AI
Gateway workspace name. A workspace API key belongs to the gateway workspace
where you created it, independently of the local name.

In Strata Cloud Manager, open AI Security → AI Gateway, select the intended
workspace, and create a user workspace API key with inference permission
(`completions.write`). Apply the required expiry and usage limits. That workspace
also needs a default saved config/model route, or an explicit route supplied by
your administrator. Its gateway policy must permit workspace keys alongside SSO.

For a new local profile:

```sh
airs env create workspace-api --gateway-url https://gateway.example.com/v1
```

Choose **Workspace API key** and enter the key in the hidden prompt. If the
profile already exists, reuse it:

```sh
airs --environment workspace-api login --with-api-key
airs --environment workspace-api doctor --verify-access
airs --environment workspace-api
```

A saved credential is not yet proof of gateway access. Verification sends one
small inference request. HTTP 401 indicates credential rejection; 403 indicates
an authorization denial; 446 indicates a guardrail denial. Some gateway policies
return a blocking denial with HTTP 200; the harness checks the hook results too.
Use the reported request/gateway trace ID when asking the gateway administrator
to inspect the request. Creating another local environment does not change the
key's workspace or fix its policy.

```sh
airs env list
airs env use workspace-api
airs --environment work doctor --verify-access
airs env use work
airs env remove workspace-api
```

Removing an environment unregisters its local name and preserves its files. It
does not delete the gateway workspace or revoke its API key. Select another
environment before removing the default.

MCP authorization is separate: add the gateway-provided MCP URL in this same
environment, then complete `airs --environment workspace-api mcp login SERVER`
if the add flow did not already sign you in. Company SSO and any upstream
ServiceNow consent still apply; an inference API key does not grant MCP access.
See the [getting-started guide](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/)
for the complete SSO and ServiceNow flow.

## Credentials and recovery

Choose one authentication source:

- `login --credential-file PATH` references an existing owner-only regular file.
  The application stores the path and a credential fingerprint, without copying
  the key into configuration. Symlinks and public files are rejected.
- `login --credential-env NAME` references a variable supplied by your secret
  manager. Its value is excluded from local tool subprocess environments.
- Pipe a workspace key into `login --with-api-key` to use the OS credential store.
  An unavailable store produces an error; there is no automatic plaintext fallback.

`env status` performs local checks. `doctor --json` additionally makes a bounded,
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

From the repository root, executable fixtures use Python 3.11 or newer and
`pyte==0.8.2` for terminal checks:

```sh
AIRS_HARNESS_BIN=codex-rs/target/release/airs-harness \
  python3 -m unittest discover -s scripts -p 'test_airs_harness*.py' -v
```

Rust checks use `just test`. The live contract probe is
`scripts/validate_live_gateway.py --help`; it reads explicit credential files,
submits small acceptance fixtures, and writes a redacted result. See
[RELEASE.md](RELEASE.md) for platform-specific test limitations and install receipts.
For versioned test handoffs, follow [Publish and verify an AIRS test package](RELEASE-TEST-PACKAGES.md),
which binds three native installations to exact package and validator bytes.

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
