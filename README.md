# Prisma AIRS Terminal

A standalone terminal agent derived from the open-source Codex Rust CLI. The
`airs-terminal` executable connects directly to a user-configured Prisma AIRS AI
Gateway. Files, shell commands, skills, approvals and sessions use the local agent
runtime. Selected file contents and tool results are sent to the gateway as
inference context.

**There is no PAH package, SDK, proxy, web application or service dependency.**

## Status: 0.1.0-alpha.2 protocol prototype

Implemented: independent application state, gateway setup, workspace credential
references, local capability catalog, optional-model Responses serialization,
qualified route validation and redirect rejection. A deterministic test exercises
the built agent, a local file edit, and the tool-result continuation for both
routing modes.

This is not the completed team MVP. Named environments, OS credential storage,
Keycloak login/refresh, per-user remote MCP validation, complete product branding
and installed macOS/Linux acceptance remain open. Live gateway validation is
blocked by missing access to the new terminal workspace. Read
[IMPLEMENTATION.md](IMPLEMENTATION.md) before treating this as a release candidate.

## Build and configure

For native Mac testing, see [MACOS.md](MACOS.md), including a local protocol test
that does not need a live gateway key. No verified macOS binary is published yet.

Rust 1.95.0 is pinned. Read `AGENTS.md` for build prerequisites and test conventions.

```sh
cd codex-rs
cargo build --locked -p codex-cli --bin airs-terminal
./target/debug/airs-terminal --version
./target/debug/airs-terminal setup --help
```

Setup requires your HTTPS inference API root (including any `/v1` prefix). The
workspace key comes from `AIRS_API_KEY` by default; `--credential-env` changes its
environment-variable name. The local context budget defaults to **1,000,000
tokens**; `--context-window` overrides it. This client setting does not increase
the actual gateway or provider context limit:

```sh
airs-terminal setup \
  --gateway-url https://your-gateway.example/v1 \
  --credential-env AIRS_API_KEY \
  --model '@your-provider/your-model'
```

Supply the key through your secret manager or shell environment, then run
`airs-terminal` inside a local repository. Use `airs-terminal exec "your task"`
for noninteractive operation. Never put a real key in a command argument or a
configuration file. Setup stores only its environment-variable reference and
excludes that variable from local tool environments; login-shell loading is
disabled in the generated configuration.

The default selection, `airs-gateway-default`, refers to local capabilities and
**omits the root `model` field** in inference requests. An explicit `-m
'@provider/model'` preserves that exact route. Gateway policies and authorization
must govern both choices; the local model list is not an access-control boundary.

State is stored in `~/.airs-terminal`, or the absolute directory specified by
`AIRS_TERMINAL_HOME`. This does not change `HOME` or `CODEX_HOME`. Setup refuses to
overwrite existing configuration. An unconfigured terminal fails with a setup
instruction instead of selecting the upstream OpenAI provider.

Some inherited screens and help still say Codex. The separate upstream `codex`
binary is retained for compatibility tests and is not the product entry point.
Upstream login, cloud, remote-control, app-server and update commands are disabled
in this prototype.

## Validation

Use `just test` for Rust tests. The actual-executable protocol test needs only
Python's standard library and the built binary:

```sh
python3 -m unittest discover -s scripts -p test_airs_terminal.py -v
```

It uses a loopback Responses server and a fixed shell command in temporary test
directories. Its default is the `workspace-write` sandbox. On a host unable to
run Bubblewrap, `AIRS_TERMINAL_TEST_SANDBOX=danger-full-access` allows protocol-only
validation of these fixed test fixtures. That result does **not** establish
sandbox acceptance. Do not use that setting to work around sandbox failures for
ordinary agent tasks. Current host limitations and test receipts are recorded in
[IMPLEMENTATION.md](IMPLEMENTATION.md).

## Upstream and license

Pinned baseline: Codex `rust-v0.153.4`, commit
`3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`. Internal upstream crate names and
versions are preserved for reviewable updates. The release tag's stale lockfile
required normalizing 149 workspace versions; external resolutions are unchanged.
See [BASELINE.json](BASELINE.json) and [UPSTREAM.md](UPSTREAM.md).

Codex-derived code remains Apache-2.0. Preserve [LICENSE](LICENSE),
[NOTICE](NOTICE), dependency notices and upstream history. The product does not
imply OpenAI endorsement. The original introduction is retained as
[README.upstream.md](README.upstream.md).
