---
title: Prisma AIRS Harness for macOS
description: Install on Apple Silicon, choose workspace API key or Keycloak authentication, and verify your first session.
release: 0.1.0-alpha.10-auth-review
updated: 2026-09-09
audience: [users, administrators]
platform: macos-arm64
---

# Prisma AIRS Harness on your Mac

Install the prebuilt application, connect to your organization's AI Gateway, and
start working in a local folder. The terminal command is **`airs-harness`**.
No Rust compiler or source checkout is needed for the npm installation.
For the latest instructions, use [this guide on GitHub](https://github.com/cdot65/airs-harness/blob/main/MACOS.md); a bundled copy reflects the release that included it.

**Signed authentication review release:** `0.1.0-alpha.10` is published to
GitHub Packages under `auth-review`, with the signed and notarized Apple Silicon
executable. Install the exact review version:

```zsh
npm login --auth-type=legacy --registry=https://npm.pkg.github.com
npm install -g @cdot65/prisma-airs-harness@0.1.0-alpha.10 --include=optional --registry=https://npm.pkg.github.com
airs-harness --version
airs-harness
```

For npm login, use your GitHub username and a token with `read:packages` and
access to this private package. Skip that first command if already logged in.
The expected version is `airs-harness 0.1.0-alpha.10`. First launch guides new
users through gateway setup and company sign-in or a hidden workspace-key prompt.
Existing users can run `airs-harness login` to sign in again. Follow
[guided setup and sign-in](AUTHENTICATION-ONBOARDING.md) for verification,
resume, logout, and diagnostics. Credentials stay in the native Keychain.

This is an authentication review release, not full production acceptance. The
owner's affected Mac still needs hands-on retesting. The alpha.9 commands below
remain historical baseline/alternative-registry instructions; they do not install
this signed review release.

**Supported:** Apple Silicon Macs using native `arm64` Terminal and Node.js.
**Tested baseline:** macOS 15. Intel Macs are unsupported.

## Choose your path

Installation and gateway authentication are independent choices. Either registry
works with either gateway authentication method.

| Decision          | Choose this when…                                          | Go to                                                             |
| ----------------- | ---------------------------------------------------------- | ----------------------------------------------------------------- |
| GitHub Packages   | Your organization grants you package access on GitHub      | [Install from GitHub](#option-a-github-packages)                  |
| Verdaccio         | Your Mac can reach the organization's private npm registry | [Install from Verdaccio](#option-b-verdaccio)                     |
| Workspace API key | Your operator gives you a gateway workspace key            | [Save a key in Keychain](#option-a-workspace-api-key-in-keychain) |
| Keycloak          | Your operator provisions an individual SSO account         | [Sign in with your browser](#option-b-keycloak-browser-sign-in)   |

**Three different credentials:** a GitHub token downloads the application; a
workspace key or Keycloak login authorizes gateway inference; scanner and
management credentials enable the corresponding Prisma AIRS CLI operations.
One does not replace another.

Follow steps 1–3, optionally connect MCP in step 4, then run your first task in
step 5. The [troubleshooting table](#troubleshooting) covers common errors.

## 1. Prepare your Mac

Open **Terminal** using Spotlight (`Command` + `Space`, then type `Terminal`).
Install [Homebrew](https://brew.sh/) if it is not already installed and complete
its displayed **Next steps** to add `brew` to your shell. Then run:

```zsh
brew install node git ripgrep
node --version
node -p 'process.arch'
```

The architecture must be `arm64`. If it says `x64`, reopen Terminal without
Rosetta and use an Apple Silicon Node installation. The harness requires Node.js
22.13+ in the 22.x line, or 23.5+.

When using nvm, `node --version` may differ from Homebrew's installed version
because nvm selects the active executable. An active arm64 Node 22.23.2 already
meets this release's requirements; no switch to Homebrew Node is necessary.

Connect your organization's VPN when needed. GitHub installation uses
`github.com`, `npm.pkg.github.com`, and `registry.npmjs.org`. Verdaccio uses
`npm.cdot.io`. Both gateway login methods need `airs.cdot.io`; Keycloak also
needs `auth.dev.cdot.io`, and remote scanner tools need `mcp-airs.cdot.io`.
These are this deployment's addresses; use your operator's values elsewhere.

## 2. Install the application

Choose **one** registry. Both packages provide the same `airs-harness` command
and install Prisma AIRS CLI **5.2.0** with the harness's eight built-in skills.
Keep optional dependencies enabled: they contain your native executable.

### Option A: GitHub Packages

Your GitHub account needs read access to the launcher **and** Apple Silicon
package. Repository access is sufficient only after the owner enables package
permission inheritance. See the [administrator setup](#administrator-setup-for-github-package-access).

1. In GitHub, open **Settings → Developer settings → Personal access tokens →
   Tokens (classic) → Generate new token (classic)**.
2. Give the token an expiration and select **`read:packages`**. Use your own
   account with package access. If your organization requires SSO authorization
   for tokens, complete that authorization too.
3. Authenticate from Terminal:

   ```zsh
   npm login --auth-type=legacy --registry=https://npm.pkg.github.com
   ```

   Enter your GitHub username. At the password prompt, paste the **token**, not
   your GitHub password. Follow any remaining prompts. This is registry login,
   separate from the harness's gateway sign-in.

4. Resolve the approved launcher download:

   ```zsh
   airs_harness_url="$(npm view @cdot65/prisma-airs-harness@0.1.0-alpha.9 dist.tarball --registry=https://npm.pkg.github.com)"
   ```

   Continue only if this succeeds. If you previously installed the unscoped
   `airs-harness` package globally in this npm prefix, remove that old launcher
   before installing its replacement:

   ```zsh
   npm uninstall -g airs-harness
   ```

5. Install the resolved download:

   ```zsh
   if [[ -n "$airs_harness_url" ]]; then
     npm install -g "$airs_harness_url" --include=optional --registry=https://registry.npmjs.org
   else
     printf 'Resolve the GitHub package URL successfully before installing.\n'
   fi
   ```

The launcher and native dependency come from GitHub; the pinned public CLI/SDK
resolve through npmjs. **Do not map the entire `@cdot65` scope to GitHub**: those
public dependencies share the scope. For the registry's authentication rules,
see [GitHub's npm documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry).

### Option B: Verdaccio

Connect to the organization's LAN/VPN. This registry permits anonymous downloads;
no GitHub token or npm login is needed for this path. Its existing package name
is **`airs-harness`**.

```zsh
npm install -g airs-harness@0.1.0-alpha.9 --include=optional --registry=https://npm.cdot.io
```

If switching from the scoped GitHub package in the same npm prefix, first remove
that launcher with `npm uninstall -g @cdot65/prisma-airs-harness`. Environments,
credentials and conversations are stored outside the npm installation.

### Verify the installation

```zsh
airs-harness --version
airs-harness airs --version
```

Expected output:

```text
airs-harness 0.1.0-alpha.9
5.2.0
```

## 3. Choose how to authenticate

Create a separate named environment for each identity or credential. The examples
use **`mac-key`** for a workspace key and **`mac-sso`** for Keycloak. You can keep
both and select one explicitly for each session. Run each environment's `setup`
only once; it does not overwrite an existing environment.

### Option A: Workspace API key in Keychain

Ask your operator for an **AI Gateway workspace API key** authorized for your
workspace. This is not the Runtime scanner API key. You do not need a Keycloak
account for this authentication path.

Create the environment using the gateway's default route:

```zsh
airs-harness setup --environment mac-key \
  --gateway-url https://airs.cdot.io/v1
```

Paste this entire block into macOS Terminal. It prompts without echoing your key
and pipes it directly to the harness's credential-store login:

```zsh
(
  set +x
  read -rs 'airs_workspace_key?Workspace API key: ' || exit
  printf '\n'
  printf '%s' "$airs_workspace_key" |
    airs-harness --environment mac-key login --with-api-key
)
```

Paste the key **at the prompt**, then press Return. The temporary shell variable
is discarded when the block exits. The key is saved in **macOS Keychain**; you do
not need to export `AIRS_API_KEY` in subsequent Terminal sessions or put the key
in shell startup files. Allow the Keychain prompt if macOS asks.

```zsh
airs-harness --environment mac-key status
airs-harness --environment mac-key doctor
```

**Expected:** credentials are available and the environment passes local
readiness checks. Doctor does not submit inference; the first task in step 5
verifies that the gateway accepts the key and its route.

Omitting `--model` sends no `model` key in inference requests, allowing AIRS to
choose the route. To offer an operator-approved explicit route, include
`--model '@provider/model'` when creating a new environment, replacing the
placeholder with the actual authorized value. `/model` selects from that catalog.
The local context-window default is 1,000,000 tokens; your operator should set
`--context-window` to the actual model capacity if it differs.

### Option B: Keycloak browser sign-in

Ask your operator for a Keycloak account with the inference role. Use these
values for the current deployment:

```zsh
airs-harness setup --environment mac-sso \
  --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'

airs-harness --environment mac-sso login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot \
  --audience airs-terminal-inference
```

Complete sign-in in the browser and return to Terminal. macOS stores the login
credentials in Keychain. No workspace key needs to be pasted. The explicit route
above is the configured authenticated provider; your operator must supply the
correct route if your deployment differs.

```zsh
airs-harness --environment mac-sso status
airs-harness --environment mac-sso doctor
```

The existing `airs-terminal-*` client IDs and audiences are server-side identity
configuration names. Keep them as shown; the executable is still `airs-harness`.
If you cannot use the local browser callback, add `--device-auth` to `login`
when your identity provider enables device authorization.

### Advanced: use an existing credential source

Keychain is the standard Mac workspace-key flow. If your organization explicitly
provides a protected key file or a secret-manager environment variable, choose
**one** of these alternatives in a fresh workspace-key environment:

```zsh
airs-harness setup --environment mac-external --gateway-url https://airs.cdot.io/v1
```

| Credential source       | Login command                                                                                     | Required on subsequent launches                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| Existing file           | `airs-harness --environment mac-external login --credential-file /absolute/path/to/workspace.key` | The same accessible, user-owned regular file, mode `600`; symlinks are rejected |
| Secret-manager variable | `airs-harness --environment mac-external login --credential-env AIRS_API_KEY`                     | Your secret manager must supply `AIRS_API_KEY` each time                        |

These modes reference the credential source; they do not import it into Keychain.
Do not type a literal key into an `export` command or paste it into the agent chat.

## 4. Optional: connect remote scanner tools

Local file editing and coding do not require MCP. If you want the remote
`pan_inline_scan` tool, configure it **before the environment's first agent
session**: session history is bound to its identity and tool configuration.
Inference and MCP have separate credential bindings and permissions.

### With a workspace key

Ask your operator for the complete MCP endpoint and a **separately authorized
MCP credential file**. The file must already exist, be owned by you, have mode
`600`, and not be a symlink. Keep it outside project repositories. Replace the
example URL and file path below with the operator-provided values:

```zsh
airs-harness --environment mac-key setup-mcp --name security \
  --url https://tools.example/workspace/security/mcp \
  --credential-file /absolute/path/to/mcp.key \
  --tool pan_inline_scan --required
```

Alpha.9's workspace-key MCP setup accepts `--credential-file`; it has no
`--with-api-key` Keychain import option. Do not assume that the inference key or
an inference Keychain entry authorizes MCP. If your organization prohibits file
credentials, use an authorized Keycloak environment for MCP or omit MCP.

### With Keycloak

The operator must grant the account the scanner role as well as inference access.
For the current deployment, run:

```zsh
airs-harness --environment mac-sso setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp \
  --audience airs-terminal-security \
  --tool pan_inline_scan --required
```

Complete any browser sign-in using the **same user and issuer** as inference.
MCP has a distinct client and audience. Add `--device-auth` if you need supported
device authorization. `--required` intentionally prevents startup when this
server is unavailable; omit MCP setup entirely for a coding-only environment.

## 5. Run your first task

Create a local review folder:

```zsh
mkdir -p "$HOME/airs-harness-review"
cd "$HOME/airs-harness-review"
```

Launch **one** of your configured environments:

```zsh
airs-harness --environment mac-key
```

Or, for Keycloak:

```zsh
airs-harness --environment mac-sso
```

At the agent prompt, enter:

> Create calculator.mjs with add and multiply functions. Add tests using node:test
> for positive, negative, and zero inputs. Run node --test and report the actual
> results and changed files.

If you configured MCP, run `/mcp` to check the security server, then ask:

> Use pan_inline_scan to scan "Hello from my Mac". Report the actual scan action
> and scan ID. If the tool is unavailable, say so.

To check the included skills, ask:

> Use $prisma-airs-cli to check the managed CLI version and diagnose readiness.
> Report missing credential variable names without revealing their values.
> Do not change configuration.

## 6. Resume, switch environments, or sign out

Close the harness and Terminal. Open a new Terminal and return to your work folder.
For workspace-key sessions:

```zsh
cd "$HOME/airs-harness-review"
airs-harness --environment mac-key resume
```

For Keycloak sessions, use `airs-harness --environment mac-sso resume`. Select your
conversation and ask it to rerun the tests. Keychain credentials persist across
Terminal windows; a new shell does not require another workspace-key export.

| Task                                      | Command                                     |
| ----------------------------------------- | ------------------------------------------- |
| List configured environments              | `airs-harness env list`                     |
| Check workspace-key readiness             | `airs-harness --environment mac-key doctor` |
| Check Keycloak readiness                  | `airs-harness --environment mac-sso doctor` |
| Sign out of the workspace-key environment | `airs-harness --environment mac-key logout` |
| Sign out of the Keycloak environment      | `airs-harness --environment mac-sso logout` |

A different workspace key, gateway destination, identity, or MCP binding requires
a **new environment**. Use a new name during key rotation; existing conversations
must not silently switch credentials. Logout disables new local credential use;
server-side key revocation remains an AIRS operator action. Stop other running
sessions to discard their cached credentials.

## Optional: enable direct Prisma AIRS CLI operations

The installed CLI can diagnose scanner and management readiness before gateway
login:

```zsh
airs-harness airs doctor --output json
```

This differs from `airs-harness --environment mac-key doctor`, which checks the
harness environment. Missing scanner/management credentials in the CLI doctor
report do not mean your gateway workspace key is missing.

| Capability                        | Credential settings                                                              |
| --------------------------------- | -------------------------------------------------------------------------------- |
| Runtime scanning through the CLI  | `PANW_AI_SEC_API_KEY`, or a supported scanning token via `PANW_AI_SEC_API_TOKEN` |
| Management APIs                   | `PANW_MGMT_CLIENT_ID`, `PANW_MGMT_CLIENT_SECRET`, `PANW_MGMT_TSG_ID`             |
| Trusted alternative configuration | `PRISMA_AIRS_CONFIG_PATH`, or existing protected `~/.prisma-airs/config.json`    |

Have your operator provision these through the approved secret manager or trusted
configuration. Gateway workspace keys and Keycloak user tokens do not replace
management service-account credentials. CLI 5.2.0's DLP management commands require
explicit `PANW_MGMT_*` environment variables even when other commands accept the
config file. The wrapper does not automatically load a project's `.env`.

CLI environment credentials can be inherited by local tools; this is not a secret
broker. Doctor exception text is not universally redacted. Share summarized check
statuses and missing variable names, rather than raw credentials or debug dumps.
See [CLI setup and compatibility](PRISMA-AIRS-CLI.md) for provisioning and limits.

## Updates and troubleshooting

For updates, use the approved version and the same registry path from step 2.
Do not install both npm packages into the same prefix: both provide `airs-harness`.
Uninstalling the npm package removes the launcher, not environments or history.
Fresh state is in `~/.airs-harness`; existing `~/.airs-terminal` state is reused.

### Troubleshooting

| Symptom                                                   | What to do                                                                                                                                                                                           |
| --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| GitHub `401`, `403`, or private-package `404`             | Check npm login, token expiration and `read:packages`, any organization SSO authorization, and access to both launcher and native package. Repo access alone does not establish package inheritance. |
| Missing native package after npm install                  | Check access to the Apple Silicon package and reinstall with `--include=optional`. Keep `node -p 'process.arch'` on `arm64`.                                                                         |
| Public CLI dependency resolves against GitHub             | Remove or correct the `@cdot65:registry` override in the npm configuration you control. Follow the URL-based GitHub installation above.                                                              |
| `EEXIST` for `airs-harness`                               | Uninstall the previous npm package in the same prefix before switching registry/package names. Check `command -v airs-harness` for a manually installed binary; preserve it before replacing it.     |
| `EACCES` during npm install                               | Use a user-owned prefix as shown below; do not use `sudo npm install`.                                                                                                                               |
| `command not found`                                       | Complete Homebrew's PATH setup and reopen Terminal. Check `command -v node` and `command -v airs-harness`.                                                                                           |
| Missing `AIRS_API_KEY`                                    | Select the intended environment explicitly and complete its login. For `mac-key`, use the Keychain flow instead of adding an export.                                                                 |
| Keychain unavailable                                      | Unlock your login keychain in Keychain Access, then retry in your normal macOS desktop session. Linux D-Bus instructions do not apply.                                                               |
| Gateway or registry timeout                               | Check the VPN, organization DNS, and the endpoint required for your chosen path.                                                                                                                     |
| Gateway `401` or `403`                                    | Have the operator check the key's workspace permissions or the user's Keycloak roles/audience. Successful GitHub login only grants package download access.                                          |
| Environment already exists or credential identity changed | Use `env list` to find the intended environment. Use a fresh name for a different key, identity, endpoint or tool configuration.                                                                     |
| Resume fails after changing MCP                           | Return to the original matching configuration for that history; use a new environment for a new binding.                                                                                             |
| CLI doctor reports missing management credentials         | Provision the separate SCM/scanner credentials only if those capabilities are needed. See the CLI section above.                                                                                     |

For an npm prefix permission error, these commands configure a directory you own.
If you already use a Node version manager, use its managed prefix instead.

```zsh
mkdir -p "$HOME/.local/bin"
npm config set prefix "$HOME/.local"
touch "$HOME/.zprofile"
rg -q -F -x 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zprofile" ||
  printf '%s\n' 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zprofile"
export PATH="$HOME/.local/bin:$PATH"
```

Then repeat your chosen installation. This follows
[npm's permissions guidance](https://docs.npmjs.com/resolving-eacces-permissions-errors-when-installing-packages-globally/).

When reporting a problem, include `airs-harness --version`,
`sw_vers -productVersion`, `node -p 'process.arch'`, and sanitized error text.
The release has verified ad-hoc signatures; Apple Developer ID signing,
notarization and older macOS acceptance remain separate work. Follow your
organization's endpoint policy if installation is blocked; do not disable
Gatekeeper to install this alpha.

### Keychain is unlocked but login still fails

Both Keycloak login and `login --with-api-key` use the OS credential store. A
storage error occurs before either flow can establish gateway access; switching
between those two login commands does not bypass Keychain.

To continue with a workspace API key while investigating storage, use the
existing environment-variable mode. If you already configured `mac-key`, run
this block directly in your Mac's terminal; no reinstall or repeat setup is needed:

```zsh
(
  set +x
  read -rs 'AIRS_API_KEY?Workspace API key: ' || exit
  printf '\n'
  export AIRS_API_KEY
  airs-harness --environment mac-key login --credential-env AIRS_API_KEY &&
    airs-harness --environment mac-key
)
```

The prompt hides the key. The harness saves a variable reference and credential
fingerprint, not the key, and does not access Keychain for this credential.
Exiting the harness ends the temporary shell and its exported key. Repeat the
block with the same key on your next launch; to resume, append `resume` to the
last `airs-harness` command. A rotated key requires a new environment.

This is an explicit workspace-key alternative, not a repair for Keycloak token
storage. Gateway authorization is checked when you make a request. Do not paste
the key into chat, command arguments, or shell startup files.

Opening and unlocking the login keychain does not establish that the harness's
process can use the **default user keychain** selected by macOS. Alpha.9 reports
a generic storage error and does not expose the underlying OS status. If the
same error persists, stop reinstalling or repeating the unlock step and collect
these read-only diagnostics:

```zsh
security default-keychain -d user
security list-keychains -d user
sw_vers -productVersion
codesign --verify --strict --verbose=2 "$(npm root -g)/@cdot65/prisma-airs-harness/node_modules/airs-harness-darwin-arm64/bin/airs-harness"
```

The signature path above is for the scoped GitHub npm installation. These commands
report paths, OS version and signature validity, not saved passwords. Include
whether the failing command runs in a desktop Terminal, over SSH, or another
remote session. If using SSH, compare the same login in Terminal opened inside
the signed-in macOS desktop session. Do not reset the keychain or change its
access rules to work around an unidentified error.

## Administrator setup for GitHub package access

Do this once before distributing the GitHub instructions to teammates:

1. Open each of the three package pages: [launcher](https://github.com/users/cdot65/packages/npm/package/prisma-airs-harness),
   [Apple Silicon](https://github.com/users/cdot65/packages/npm/package/prisma-airs-harness-darwin-arm64),
   and [Linux x64](https://github.com/users/cdot65/packages/npm/package/prisma-airs-harness-linux-x64).
2. Select **Connect repository → cdot65/airs-harness** where needed.
3. Open **Package settings → Manage access / Inherited access** and explicitly
   enable **Inherit access from repository**. Confirm that the intended users
   have repository read access. Alternatively, grant specific users package Read
   access directly on both packages needed by their platform.
4. Keep package visibility private and test with a teammate's own account/token.
   Successful Actions installation does not prove that an individual user has
   inherited access.

Linking an already-published package does not automatically enable inheritance.
**Manage Actions access** is a separate setting and does not grant human users
package access. See [GitHub's access-control documentation](https://docs.github.com/en/packages/learn-github-packages/configuring-a-packages-access-control-and-visibility).
Our publication receipt recorded no repository association; the owner must verify
these settings. [Release evidence and registry maintenance](PUBLICATION.md).

## Build from source (contributors)

This is a development path. A direct native binary does not install or manage
the required Prisma AIRS CLI npm dependency. Use the npm distribution above for
the integrated CLI and skills experience; see [the direct-native CLI requirements](PRISMA-AIRS-CLI.md#run-and-diagnose) when developing from source.

1. Install Apple's Command Line Tools if needed:

   ```sh
   xcode-select --install
   ```

   Finish the installer, then confirm `xcrun --find clang` succeeds. See
   [Apple's instructions](https://developer.apple.com/documentation/xcode/installing-the-command-line-tools).

2. Install Rust through [rustup's official installer](https://rust-lang.org/tools/install/).
   If `rustup` is already available, keep your existing installation. The checked-in
   `codex-rs/rust-toolchain.toml` selects Rust **1.95.0** for this checkout; it does
   not require changing your global default toolchain.

3. Install native build tools and runtime file search. With an existing
   [Homebrew](https://brew.sh/) installation:

   ```sh
   brew install cmake pkgconf ripgrep
   ```

   References: [CMake](https://formulae.brew.sh/formula/cmake),
   [pkgconf / pkg-config](https://formulae.brew.sh/formula/pkgconf),
   [ripgrep](https://formulae.brew.sh/formula/ripgrep).
   Python 3 is needed for the protocol tests below (`python3 --version`).

### Compile

For source testing, clone the independent repository using your GitHub access.
The repository is named `airs-harness`.

```sh
git clone git@github.com:cdot65/airs-harness.git airs-harness
cd airs-harness/codex-rs
rustup show active-toolchain
test "$(uname -m)" = arm64 || exit 1
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4 \
  cargo build --locked -p codex-cli --bin airs-harness
./target/debug/airs-harness --version
./target/debug/airs-harness setup --help
```

The first build downloads dependencies and compiles the Rust agent engine. It
needs internet access and substantial disk space; elapsed time depends on the
Mac. This deliberately builds the terminal binary rather than the whole upstream
workspace or its optional V8 code-mode host.

On Apple Silicon, `uname -m` should print `arm64`, and `rustup show active-toolchain`
should identify `aarch64-apple-darwin`. Use a native Terminal and matching Rust installation if
you inadvertently started under Rosetta.

### Test locally without a live gateway key

From `codex-rs`, return to the repository root and run the executable integration
suite. It starts its own loopback Responses server, uses a test-only credential,
and asks the actual agent runtime to perform a fixed file edit in a temporary
directory. It also checks model omission, explicit routing, missing credentials,
state isolation and redirect rejection.

```sh
cd ..
python3 -m unittest discover -s scripts -p test_airs_harness.py -v
```

Keep the test's default `workspace-write` sandbox on macOS. These deterministic
tests do not prove live model or gateway-policy compatibility.

### Install the binary you built

After the build and tests succeed, from the repository root:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 codex-rs/target/debug/airs-harness "$HOME/.local/bin/airs-harness"
"$HOME/.local/bin/airs-harness" --version
"$HOME/.local/bin/airs-harness" setup --help
```

This overwrites an existing `~/.local/bin/airs-harness` if present; retain any
previous version you want to keep. Add `~/.local/bin` to your shell's PATH if you
want to invoke it as simply `airs-harness`.

Fresh state uses `~/.airs-harness` (or `AIRS_HARNESS_HOME`); existing legacy
state is reused according to [rename contract](https://github.com/cdot65/airs-harness/blob/main/RENAME.md). macOS uses Keychain. Follow
[authentication choices](#3-choose-how-to-authenticate) on the LAN/VPN for your
deployment. No D-Bus or GNOME Keyring setup applies to a native Mac session.
Close Terminal, open a new window and run `airs-harness resume` to verify that
Keychain credentials and local history remain accessible across sessions.

If the Mac build or tests fail, preserve the first error and the output of
`sw_vers -productVersion`, `uname -m`, and `rustup show active-toolchain`. Do not
include keys or tokens in a diagnostic report.
