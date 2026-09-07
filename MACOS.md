# Test Prisma AIRS Terminal on macOS

The current packaged executable is Linux-only. Build the source natively on your
Mac to test local filesystem access and macOS execution. This fork has not yet
passed macOS acceptance; these are source-testing instructions, not a claim that
a signed/notarized Mac release is available.

The same commands support Apple Silicon and Intel when run from a native Terminal
with a matching Rust installation. No cross-compilation target is necessary.
The upstream CI uses macOS 15 runners; older macOS/SDK combinations are unverified
for this prototype.

## Prerequisites

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

## Download and build

Download `prisma-airs-terminal-source-0.1.0-alpha.1.tar.gz` and its `.sha256`
sidecar from the workspace's `codex-rs/target/` directory. There is no public
release URL yet. Extract in a directory where `prisma-airs-terminal` does not
already exist. The archive contains committed source, licenses and dependency
lockfiles, without credentials, application state, build output or `.git` history.

```sh
shasum -a 256 -c prisma-airs-terminal-source-0.1.0-alpha.1.tar.gz.sha256
tar -xzf prisma-airs-terminal-source-0.1.0-alpha.1.tar.gz
cd prisma-airs-terminal/codex-rs

rustup show active-toolchain
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4 \
  cargo build --locked -p codex-cli --bin airs-terminal

./target/debug/airs-terminal --version
./target/debug/airs-terminal setup --help
```

The first build downloads dependencies and compiles the Rust agent engine. It
needs internet access and substantial disk space; elapsed time depends on the
Mac. This deliberately builds the terminal binary rather than the whole upstream
workspace or its optional V8 code-mode host.

On Apple Silicon, `uname -m` should print `arm64`, and `rustup show active-toolchain`
should identify `aarch64-apple-darwin`. On Intel, the corresponding architecture
is `x86_64-apple-darwin`. Use a native Terminal and matching Rust installation if
you inadvertently started under Rosetta.

## Test locally without a live gateway key

From `codex-rs`, return to the repository root and run the executable integration
suite. It starts its own loopback Responses server, uses a test-only credential,
and asks the actual agent runtime to perform a fixed file edit in a temporary
directory. It also checks model omission, explicit routing, missing credentials,
state isolation and redirect rejection.

```sh
cd ..
python3 -m unittest discover -s scripts -p test_airs_terminal.py -v
```

Keep the test's default `workspace-write` sandbox on macOS. The Linux-host
`danger-full-access` workaround recorded elsewhere is not the Mac acceptance
procedure. These deterministic tests do not prove live model or gateway-policy
compatibility.

## Install the binary you built

After the build and tests succeed, from the repository root:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 codex-rs/target/debug/airs-terminal "$HOME/.local/bin/airs-terminal"
"$HOME/.local/bin/airs-terminal" --version
"$HOME/.local/bin/airs-terminal" setup --help
```

This overwrites an existing `~/.local/bin/airs-terminal` if present; retain any
previous version you want to keep. Add `~/.local/bin` to your shell's PATH if you
want to invoke it as simply `airs-terminal`.

The new terminal uses `~/.airs-terminal` (or `AIRS_TERMINAL_HOME`), independently
of an existing Codex installation. For live inference, configure an AIRS workspace
you are authorized to use, with its real context limit and credential. The new
terminal pilot workspace still needs its administration access resolved; see
[IMPLEMENTATION.md](IMPLEMENTATION.md). Keycloak and the full team MVP remain open.

If the Mac build or tests fail, preserve the first error and the output of
`sw_vers -productVersion`, `uname -m`, and `rustup show active-toolchain`. Do not
include keys or tokens in a diagnostic report.
