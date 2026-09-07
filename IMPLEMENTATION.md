# Implementation and evidence

Updated: 2026-09-07. This is a protocol-prototype receipt, **not a completed-MVP or
production-readiness claim**.

## Delivered scope

- Separate Git repository with full upstream ancestry, executable `airs-terminal`
  (`0.1.0-alpha.1`) and independent `~/.airs-terminal` state.
- Setup for a configurable HTTPS Responses API root, workspace-key environment
  reference and local capability catalog. Existing config is
  never overwritten; newly created Unix state/config permissions are 0700/0600.
- Gateway-default inference omits the root `model` field. Explicit choices retain
  `@provider/model`, including nested model names and version suffixes. Invalid
  choices and missing capability descriptors fail before inference.
- No implicit upstream provider when unconfigured. The terminal ignores legacy
  Codex application-home and SQLite-home overrides. Selected upstream hosted
  commands and the Codex updater are blocked.
- Gateway HTTP headers are marked sensitive; redirects are not followed. The
  generated shell policy excludes the key variable and disables login shells.
- Actual executable tests drive a local file edit and its tool continuation,
  verify both model-routing modes, state isolation, credential exclusion, missing
  credential handling, setup overwrite protection and redirect rejection.

No PAH package, SDK, proxy, browser application or runtime is imported or required.
The installed Prisma AIRS administration SDK was used only as external discovery
and provisioning tooling; it is not a dependency of this repository.

## Validation record

The tested local executable is installed at `/home/cdot/.local/bin/airs-terminal`.
Run `airs-terminal --version` or `airs-terminal setup --help`. It remains
unconfigured for live inference until the gateway workspace access is resolved.

See `VALIDATION.json` for the binary SHA-256 and focused test receipt.

| Check | Result |
| --- | --- |
| Unchanged-source upstream baseline with normalized workspace versions | Passed; SHA-256 in `BASELINE.json`. |
| Standalone Linux/musl Cargo build | Passed. Development profile; not an optimized release build. |
| Focused API, provider, home-dir and CLI regression run, before final credential/state guards | 889 tests: 886 passed, 3 failed, none skipped. |
| Three failing regression tests | Bubblewrap could not mount `/proc` in this host; the affected tests are listed below. |
| Final actual-executable protocol/credential/state tests | 6 passed against both the build output and installed binary; see sandbox limitation below. |
| Dedicated gateway-routing and sensitive-header tests | 31 passed, none skipped. |
| Full workspace test attempt | Blocked during compilation: upstream `v8` 150.4.0's musl prebuilt archive returned HTTP 404. No full-suite pass is claimed. |
| Config schema generation | Passed; schema updated. |
| Bazel dependency metadata update | Passed using native musl Cargo override described below; `MODULE.bazel.lock` did not change. This is not a Bazel build result. |
| Scoped Clippy fix/check | Passed for CLI, core, API, model-provider-info and home-dir; the final provider changes also passed a dedicated run. |
| Formatting | `just fmt` and `just fmt-check`, using upstream CI's pinned Just 1.51.0. |

The failing host-dependent regression cases were:

- `sandbox_with_network_proxy_blocks_direct_loopback_access`
- `sandbox_with_network_proxy_allows_explicit_loopback_access`
- `sandbox_fetches_and_enforces_cloud_managed_permission_profile`

The actual tool-loop protocol fixtures were run with
`AIRS_TERMINAL_TEST_SANDBOX=danger-full-access`. Their server returns a fixed,
test-owned command in a temporary directory; it is not a live model issuing
unrestricted commands. This proves the local protocol loop and credential/state
behavior, **not sandbox enforcement**. The default test mode remains
`workspace-write`; run it on a compatible Linux/macOS host for acceptance.

The Alpine host cannot run Bazel's downloaded GNU Cargo. The ignored
`user.bazelrc` points Bazel's `rules_rs` host-tool repository to the installed,
unmodified Rust 1.95.0 musl Cargo binary using `--override_repository`. This is a
host-only metadata-tool override, not a dependency or toolchain version change.
No downloaded archive or checksum was altered. Recheck Bazel without that local
override on the supported glibc CI host.

An unrelated unused import removed by broad Clippy was restored to keep this
fork change focused. No application behavior depended on that import.

## Live AIRS blocker

A dedicated workspace was created successfully:

- Name: **Prisma AIRS Terminal**
- Workspace ID: `f4aca25e-fe23-4cae-bca7-91f8f3c78594`
- Slug: `ws-prisma-ff3d74`
- Required workspace scope: `ws_airs_terminal`

Provider creation in that workspace returned **HTTP 403 / AB03**. The available
management service account needs an administrator-authorized role assignment for
that scope. No provider, inference key or routing configuration was successfully
installed there. The live inference/tool-loop, security-policy enforcement and
unauthorized-model denial gates therefore remain open. Existing PAH routing and
credentials were not changed to work around this denial.

The new workspace's required metadata was accepted after matching the tenant's
existing metadata schema. Its safe creation receipt is recorded in the vault.
No credential values are committed or written to the vault.

## Remaining MVP gates

1. Authorize the management identity for the terminal workspace, then configure
   and prove live AIRS Responses turns, local tools and mandatory policy handling
   for default and explicit routes. Confirm context limits from the actual model.
2. Implement named environments and secure OS credential lifecycle.
3. Implement public-client Keycloak login, refresh, logout and user attribution.
4. Validate remote MCP with separate per-user authorization and denial cases.
5. Finish product UX/branding, skills/approval/session recovery acceptance and
   application isolation across two environments and two users.
6. Package and test installed macOS ARM64 and Linux x86-64 artifacts, including
   sandbox enforcement. Resolve the glibc CI/full-suite gate and release pipeline.
7. Perform owner hands-on acceptance against a live, fully configured environment.

The local protocol milestone is useful and reviewable. It does not yet satisfy
all MVP gates and should not be scored or announced as a production-ready 9/10.

## Context default update — 0.1.0-alpha.2

At the owner's request, setup now defaults the local context budget to
**1,000,000 tokens**. `--context-window` is optional and still overrides this
value. Setup continues to reject non-positive values. The generated configuration
and catalog use the same selected budget; the request model-omission behavior is
unchanged. Existing configurations are not rewritten by changing the default.

Use `airs-terminal setup --gateway-url https://airs.cdot.io/v1`. The local
budget does not raise the actual backend limit. Read-only inspection of the two
deployed Qwen servers found context settings of 131,072 and 32,768 tokens; the
actual route and server enforcement remain separate from this client preference.

Validation: 10 setup tests and all seven executable integration tests passed;
the seven integration tests also passed against the installed alpha.2 binary.
Formatting passed. The installed binary matches the build SHA-256 recorded in
`VALIDATION.json`. Earlier broad regression results above describe alpha.1.

## Project configuration isolation — 0.1.0-alpha.3

Launching alpha.2 from the user's home reproduced `Fatal error: select the
gateway default route or an explicit @provider/model`. Although AIRS user state
was isolated, inherited project discovery still loaded `.codex/config.toml`.
The user's Codex model selection then overrode the valid AIRS default route.

Project discovery now uses `.airs-terminal` in the standalone process, including
the root checkout hook directory for linked worktrees. Ordinary Codex processes
retain `.codex` discovery. The existing trust gate and exclusion of the active
application home from project layers remain in place. User configuration files
are not rewritten. The config crate reuses the existing application-identity
helper through one workspace dependency; no external dependency version changed.

Two actual-executable regressions first failed against alpha.2: ignoring a
conflicting Codex model in a trusted directory, and applying an AIRS-specific
project route in a trusted directory. The first asserts that default inference
still omits `model`, and the second checks the exact explicit route on both
requests of the local tool loop.

Validation: all 284 `codex-config` tests passed, as did scoped Clippy, formatting,
and Bazel dependency metadata (no MODULE.bazel.lock drift). All nine executable
integration cases passed against the build and installed alpha.3 binary. An
interactive PTY smoke check displayed the gateway route despite a conflicting
trusted `.codex` config. Startup from `/home/cdot` now passes route validation
and reaches the intentionally missing-credential check without inference.
