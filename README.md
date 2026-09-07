# Prisma AIRS Terminal

A standalone local terminal agent derived from the open-source Codex Rust CLI.
Inference goes directly to a configurable Prisma AIRS AI Gateway. Files, shell
commands, skills, approvals and session history stay under the local runtime;
selected file contents and tool results become inference context. Remote MCP
servers are configured and authenticated separately.

The terminal has no PAH application, SDK, proxy or web-service dependency.

## 0.1.0-alpha.7 — authentication workflow polish

Credential-store errors now explain recovery for Linux, macOS and Windows.
MCP inventory reports **Credential helper** when headers come from the configured
helper; connection status remains a separate result. Repeated logout reports
**Already logged out locally**, and doctor directs users to their actual identity.
The alpha.6 authentication workflow below remains the functional baseline.
See [VALIDATION.json](VALIDATION.json) for the current release's checks.

## 0.1.0-alpha.6 — user authentication

Alpha.6 adds public-client Keycloak browser/device login, verified user and resource
identities, native credential storage, refresh rotation, logout and separate MCP
authentication. One existing realm/JWKS serves both resources; AIRS receives the
original signed JWT. Workspace-key environments remain supported.

The optimized Linux binary passed 20 executable/PTY fixtures, all 27 live OIDC
checks before and after installation, and the seven-turn workspace-key regression.
The gateway requests serial tools, fixing the excessive parallel calls found in
acceptance. Native credential-store CI passed on Linux, macOS and Windows; the
full distributed terminal remains Linux x86-64. The supported core selection passed
3,929 tests. Full upstream workspace validation remains unavailable because V8
publishes no prebuilt archive for this musl target; code mode is disabled here.

Start with the [owner review guide](administration/identity/OWNER-REVIEW.md).
[RELEASE.md](RELEASE.md) and [VALIDATION.json](VALIDATION.json) record scope and
evidence; the release's `RELEASE-VERIFICATION.json` records publication checks.

## Download and install on Linux x86-64

The release is private; authenticate GitHub CLI with your own repository access.
On Debian/Ubuntu, install `git`, `ripgrep` and `bubblewrap` with your package manager.
On Alpine, use `apk add git ripgrep bubblewrap`. Python is needed by the included
acceptance fixtures, not by the terminal itself. The kernel/container policy must
permit Bubblewrap's user namespaces; the runtime fails explicitly when it cannot
establish its sandbox.

```sh
set -e
gh release download airs-terminal-v0.1.0-alpha.7 \
  --repo cdot65/prisma-airs-terminal \
  --pattern 'airs-terminal-0.1.0-alpha.7-linux-x86_64-musl.tar.gz*' \
  --dir airs-terminal-download
cd airs-terminal-download
sha256sum -c airs-terminal-0.1.0-alpha.7-linux-x86_64-musl.tar.gz.sha256
tar -xzf airs-terminal-0.1.0-alpha.7-linux-x86_64-musl.tar.gz
cd airs-terminal-0.1.0-alpha.7-linux-x86_64-musl
sha256sum -c SHA256SUMS > /dev/null
mkdir -p "$HOME/.local/bin"
if [ -f "$HOME/.local/bin/airs-terminal" ]; then
  cp -p "$HOME/.local/bin/airs-terminal" "$HOME/.local/bin/airs-terminal.previous"
fi
install -m 755 airs-terminal "$HOME/.local/bin/airs-terminal.new"
mv -f "$HOME/.local/bin/airs-terminal.new" "$HOME/.local/bin/airs-terminal"
"$HOME/.local/bin/airs-terminal" --version
```

Add `~/.local/bin` to your shell's PATH if necessary. Install is an atomic executable
replacement; running sessions retain their original process image. Keep the archive
and checksum for recovery. To roll back, atomically copy the preserved executable
through another temporary filename in the same directory. Configuration/history
are preserved, but alpha.3 does not understand alpha.4's named environments; it
uses the retained legacy application-home state. Stop old sessions before testing
an upgrade or rollback. The inherited `scripts/install/` scripts install upstream
Codex and are not this product's installer.

## Sign in with Keycloak

Keep workspace-key and user-identity histories in separate environments. For the
owner's deployment, connect through the LAN/VPN. The verified owner `cdot` has
both Terminal roles; teammates need explicit operator grants. The deployment endpoints are not publicly reachable from
an unrelated GitHub runner:

```sh
airs-terminal setup --environment work-sso --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'
airs-terminal login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot --audience airs-terminal-inference

airs-terminal setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp --audience airs-terminal-security \
  --tool pan_inline_scan --required

airs-terminal status
airs-terminal
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
use `airs-terminal env list` and the literal environment name when resuming.
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

`airs-terminal logout` disables local credential helpers and revokes usable
refresh tokens for inference and MCP. Stop running sessions to discard cached
access tokens; issued JWTs can remain valid for their 120-second lifetime.
Interrupted refresh requires signing in again rather than retrying a possibly
consumed token. A different user, issuer, gateway or resource needs a new
environment to preserve history isolation.

## Set up an environment

```sh
airs-terminal setup --environment work \
  --gateway-url https://your-gateway.example/v1 \
  --model '@openai/gpt-4.1'

# Explicit headless authentication using an existing private file:
chmod 600 /absolute/path/to/workspace-key
airs-terminal login --credential-file /absolute/path/to/workspace-key

airs-terminal doctor
airs-terminal
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
airs-terminal env list
airs-terminal env use work
airs-terminal env show work
airs-terminal --environment work exec 'Inspect this project and run its tests.'
airs-terminal --environment work resume
```

State lives in `~/.airs-terminal`, or the absolute directory selected by
`AIRS_TERMINAL_HOME`. Trusted project configuration uses `.airs-terminal/config.toml`.
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
airs-terminal setup-mcp --name security \
  --url https://tools.example/workspace/security/mcp \
  --credential-file /absolute/path/to/mcp-key \
  --tool pan_inline_scan --required

airs-terminal mcp list
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
cargo build --locked --release -p codex-cli --bin airs-terminal
./target/release/airs-terminal --version
```

From the repository root, executable fixtures use Python's standard library:

```sh
AIRS_TERMINAL_BIN=codex-rs/target/release/airs-terminal \
  python3 -m unittest discover -s scripts -p 'test_airs_terminal*.py' -v
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
