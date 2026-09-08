# Prisma AIRS Harness on your Mac

Use this runbook to install the prebuilt alpha.8 package on Apple Silicon.
Current release availability and test evidence are recorded in the
[publication report](https://github.com/cdot65/airs-harness/blob/main/PUBLICATION.md).

## Install

1. Connect your Mac to the organization's LAN or VPN. It must reach
   `npm.cdot.io`, `airs.cdot.io`, `auth.dev.cdot.io`, and `mcp-airs.cdot.io`.
2. Open **Terminal** from Applications → Utilities (or search for Terminal with
   Spotlight). If Homebrew is not installed, follow [Homebrew's official installer](https://brew.sh/),
   including its displayed “Next steps” to add `brew` to your shell.
3. Install the prerequisites and the harness:

   ```sh
   brew install node git ripgrep
   npm install -g airs-harness@0.1.0-alpha.8 --registry https://npm.cdot.io
   airs-harness --version
   ```

Node.js 22 or later is required. npm selects the native package for your Node
architecture automatically. This release supports Apple Silicon (`arm64`) Macs;
Intel Macs are excluded by project policy. Use a native
Terminal and Node installation on Apple Silicon, without Rosetta. You do not need
Rust or an npm login. Keep optional dependencies enabled; they contain the executable.

If npm reports `EACCES`, use a user-owned prefix instead of running npm as root:

```sh
npm config set prefix "$HOME/.local"
grep -qxF 'export PATH="$HOME/.local/bin:$PATH"' "$HOME/.zprofile" 2>/dev/null ||   echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zprofile"
export PATH="$HOME/.local/bin:$PATH"
npm install -g airs-harness@0.1.0-alpha.8 --registry https://npm.cdot.io
```

This follows [npm's permissions guidance](https://docs.npmjs.com/resolving-eacces-permissions-errors-when-installing-packages-globally/).
Do not use `--omit=optional` or `--ignore-optional`.

## Sign in and connect the scanner

Run the commands below in order. `setup` creates and selects the named environment.
Configure the scanner before your first coding session, because session history
is bound to your identity and tool configuration.

```sh
airs-harness setup --environment mac \
  --gateway-url https://airs.cdot.io/v1 \
  --model '@openai-terminal-auth/gpt-4.1'

airs-harness login \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-pilot \
  --audience airs-terminal-inference

airs-harness setup-mcp --name security \
  --url https://mcp-airs.cdot.io/ws-prisma-ff3d74/airs-terminal-runtime-scanner/mcp \
  --issuer-url https://auth.dev.cdot.io/realms/truffles \
  --oidc-client-id airs-terminal-mcp \
  --audience airs-terminal-security \
  --tool pan_inline_scan --required

airs-harness doctor
```

Login opens your browser for Keycloak. Complete the sign-in and return to Terminal;
the scanner may open a second sign-in using the same account. Your operator must
grant that account both inference and scanner roles. The existing `airs-terminal-*`
client and audience names are stable server-side identifiers, not the CLI name.
macOS stores credentials in Keychain. No Linux D-Bus or GNOME Keyring setup applies.

The explicit route above selects the deployed authenticated provider. In other
environments, omitting `--model` selects AI Gateway — default and sends no `model`
key. The context-window default is 1,000,000 tokens; configure it to match the
actual gateway route's model capacity if it differs.

## Try an end-to-end task

```sh
mkdir -p "$HOME/airs-harness-review"
cd "$HOME/airs-harness-review"
airs-harness --environment mac
```

At the prompt, enter:

> Create calculator.mjs with add and multiply functions. Add tests using node:test
> for positive, negative, and zero inputs, and run node --test. Then use
> pan_inline_scan to scan "Hello from my Mac". Report the actual test results,
> scan action, and scan ID.

Inspect the generated files and results. `/mcp` should show the security server
connected. Close the application and Terminal, open a fresh Terminal window,
and check that credentials and history survive:

```sh
cd "$HOME/airs-harness-review"
airs-harness --environment mac resume
```

Select your conversation and ask it to rerun the tests. No API key or JWT needs to
be copied into a shell variable. `airs-harness env list` lists saved environments;
`airs-harness --environment mac logout` removes local credentials for that environment.

## Updates and troubleshooting

Install an approved version from the same registry to update. Configuration and
history live outside npm in `~/.airs-harness` (or legacy `~/.airs-terminal` when
reused). `npm uninstall -g airs-harness` removes the executable, not that state.

- **Registry timeout:** check VPN/LAN connectivity and your organization's DNS.
- **`command not found`:** open a new Terminal after installing Homebrew and follow
  its PATH setup instructions; for a user prefix, use the PATH line above.
- **Keychain error:** unlock your macOS login keychain in Keychain Access, then retry
  in your normal signed-in desktop session. Do not export tokens as a workaround.
- **Access denied after browser sign-in:** your operator must check the account's
  roles and audience permissions; changing the CLI's model cannot grant access.
- **Diagnostic report:** include `airs-harness --version`, `sw_vers -productVersion`,
  `node -p 'process.arch'`, and the error text. Never include credentials or JWTs.

Native release acceptance targets macOS 15. The binaries have verified ad-hoc
signatures, not Apple Developer ID signing or notarization. Older macOS versions
and managed-device distribution remain separate acceptance work. Do not disable
Gatekeeper or endpoint security to install this alpha.

## Build from source (contributors)

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

## Test locally without a live gateway key

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

## Install the binary you built

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
[sign-in and scanner instructions](#sign-in-and-connect-the-scanner) on the LAN/VPN for your
deployment. No D-Bus or GNOME Keyring setup applies to a native Mac session.
Close Terminal, open a new window and run `airs-harness resume` to verify that
Keychain credentials and local history remain accessible across sessions.

If the Mac build or tests fail, preserve the first error and the output of
`sw_vers -productVersion`, `uname -m`, and `rustup show active-toolchain`. Do not
include keys or tokens in a diagnostic report.
