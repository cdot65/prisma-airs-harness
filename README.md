# Prisma AIRS Harness

A standalone local terminal agent derived from the open-source Codex Rust CLI.
Inference goes directly to a configurable Prisma AIRS AI Gateway. Files, shell
commands, skills, approvals and session history stay under the local runtime;
selected file contents and tool results become inference context. Remote MCP
servers are configured and authenticated separately.

The terminal has no PAH application, SDK, proxy or web-service dependency.

## 0.1.0-alpha.9 — managed Prisma AIRS CLI

The npm package is `airs-harness`; its launcher runs a matching prebuilt Rust
agent and requires `@cdot65/prisma-airs-cli@5.2.0` (SDK 0.28.0). Eight embedded
skills cover setup/diagnosis and seven Prisma AIRS capability areas. Run
`airs-harness airs doctor --output json` to check CLI credentials and reachability.
See [CLI setup and upgrade policy](PRISMA-AIRS-CLI.md) and
[publication status and evidence](PUBLICATION.md).

Linux x64 and Apple Silicon native/npm acceptance passed. Existing environments,
credential bindings and history retain the [rename compatibility contract](RENAME.md).
[VALIDATION.json](VALIDATION.json) records current acceptance; [RELEASE.md](RELEASE.md)
and archived validation files retain historical evidence.

## Installation

Connect to the organization's LAN/VPN, then use Node.js 22.13+ in the 22.x line, or 23.5+:

```sh
npm install -g airs-harness@0.1.0-alpha.9 --registry https://npm.cdot.io
airs-harness --version
```

Downloads are anonymous; no npm login or Rust compiler is required. Mac users
should follow [MACOS.md](MACOS.md) for prerequisites, Keycloak sign-in, scanner
setup and an end-to-end task. Intel Macs are unsupported.

The standalone native binary needs Git, ripgrep and your project tools. Linux
also requires Bubblewrap and a kernel/container policy allowing its namespaces.
The runtime fails explicitly if it cannot establish its sandbox. Python is a
validation/project prerequisite, not a dependency of the agent binary.

macOS native builds and acceptance are described in [MACOS.md](MACOS.md).
Guided first-run onboarding is a separate upcoming feature; current setup and
sign-in commands follow. The inherited `scripts/install/` tools install upstream
Codex and are not Prisma AIRS Harness installers.

## Sign in with Keycloak

Keep workspace-key and user-identity histories in separate environments. For the
owner's deployment, connect through the LAN/VPN. The verified owner `cdot` has
both Terminal roles; teammates need explicit operator grants. The deployment endpoints are not publicly reachable from
an unrelated GitHub runner:

```sh
airs-harness setup --environment work-sso --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'
airs-harness login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot --audience airs-terminal-inference

airs-harness setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp --audience airs-terminal-security \
  --tool pan_inline_scan --required

airs-harness status
airs-harness
```

Use `--device-auth` on `login` or `setup-mcp` when the browser is on another
machine. The MCP login must use the same user/issuer, with its distinct client
and audience. Configure MCP before the first coding session so history starts
with its intended tool configuration. Repeat the same login/setup-mcp options
after logout; reauthentication preserves the binding for the same user/resource.

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

`airs-harness logout` disables local credential helpers and revokes usable
refresh tokens for inference and MCP. Stop running sessions to discard cached
access tokens; issued JWTs can remain valid for their 120-second lifetime.
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

`logout` and interactive `/logout` disable new inference and MCP credential use
in the selected environment until login. Referenced key files and external
variables remain owned by the user. Stop other running sessions to discard their
cached credentials. Revoke workspace keys through AIRS when server-side revocation
is needed.

The first agent startup pins the environment's destination, credential identity,
capability catalog, context budget and MCP configuration. A changed key, catalog
or MCP binding requires a new
environment, so existing history cannot silently move to a different identity or
capability revision. Model choices within the pinned catalog remain selectable.
A running process keeps its environment even when another process changes the
default. Resume instructions include the environment name.

## Remote MCP

Configure MCP before the environment's first agent session. Use a separately
authorized MCP credential and the complete MCP server endpoint:

```sh
airs-harness setup-mcp --name security \
  --url https://tools.example/workspace/security/mcp \
  --credential-file /absolute/path/to/mcp-key \
  --tool pan_inline_scan --required

airs-harness mcp list
```

Workspace-key inference helpers use `Authorization: Bearer …`; OIDC inference
and AIRS MCP helpers use the native `x-portkey-api-key` header. Inference and MCP
are distinct credential bindings. MCP setup stores a private
reference and supplies the header through the existing MCP helper mechanism.
Changed destinations, changed keys and removed bindings fail closed. `/mcp` shows
tool availability. `--tool` limits the local catalog; configure server-side
permissions too. `--required` makes an unavailable server a startup error.

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
