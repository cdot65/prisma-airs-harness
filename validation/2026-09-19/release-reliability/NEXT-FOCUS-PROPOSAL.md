# Proposed next focus: native MCP storage for new AIRS environments

Read-only source plan, September 19, 2026. No application edit, build, owner-session operation or publication was performed. This work is separate from frozen mcp.3 and starts only after the current five focus gates finish.

## Recommendation

Yes: this is a useful next onboarding focus. Add the existing supported top-level setting `mcp_oauth_credentials_store = "keyring"` to the **new AIRS environment configuration template**, in private `codex-rs/cli/src/airs_harness.rs::configuration`. The production behavior change can be one JSON field in that owned module. Do not change the upstream enum default, configuration schema, MCP OAuth implementation, protocol or TUI manager.

Every named environment creation path converges through `airs_environment::create` → `airs_harness::setup_in` → `configuration`: explicit `env create NAME --gateway-url`, guided `env create`, and the bare-AIRS welcome form. Persisting the setting there makes both shell `airs mcp` and the in-session manager's child process load it from the same environment home. It removes the manual editor step for **newly created profiles**, independently of whether inference uses SSO or a workspace key.

Do not add automatic migration. Existing profiles with explicit `file`, `auto` or `keyring`, and existing profiles with no setting, remain untouched. The existing creation code refuses duplicate names and `setup_in` refuses preexisting config/catalog files, using no-clobber publication. No new startup write, native credential read, token copy, logout or cleanup is needed just to choose the new default.

## Source evidence

- `codex-rs/cli/src/airs_harness.rs:133–166`: generated AIRS configuration currently omits MCP storage mode. `setup_in` at 197–220 refuses existing config/catalog and uses `persist_noclobber`.
- `codex-rs/cli/src/airs_environment.rs:281–305`: new UUID home creation calls `setup_in` before registry publication; existing environment name rejected. Runtime template change naturally covers all named-creation callers.
- `codex-rs/cli/src/airs_setup.rs:52–78` and `airs_welcome/forms.rs:124`: guided plain-terminal and branded welcome flows use those same creation paths.
- `codex-rs/config/src/types.rs:124–155`: upstream MCP mode defaults to `Auto`; `Keyring` is already supported. Linux/macOS default keyring backend is Direct; Windows uses the encrypted Secrets backend and remains outside released native platform scope.
- `codex-rs/core/src/config/mod.rs:276–289,4277–4281`: absent configuration resolves to the upstream default. Special package version `0.0.0` rewrites keyring/auto to file for local development; current workspace version is `0.154.0`. Acceptance must use the real installed release binary/version, not a special development build that bypasses the intended store.
- `codex-rs/tui/src/airs_mcp_manager/process.rs:135–150`: in-session operations launch the exact running native executable with selected `AIRS_HARNESS_HOME`. They do not force a store mode. Persisted configuration is the minimal common boundary.
- `codex-rs/cli/src/mcp_cmd.rs:467,597,639,672`: MCP operations consume the loaded store mode.
- `codex-rs/rmcp-client/src/oauth/resolved_store.rs:147–207`: Auto may consult File when native credentials are absent/unavailable; Keyring reads only the selected native backend and propagates errors.
- `codex-rs/rmcp-client/src/oauth.rs:446–468,547`: Keyring persistence uses existing native writer and fallback-file cleanup for that login. The proposed configuration change itself performs no credential migration.

## Scope and compatibility choices

1. **A creation default, not universal enforcement.** Existing explicit settings remain authoritative in existing profiles. Normal upstream configuration layering remains in effect; do not claim that a generated keyring field prevents all later user/project/CLI overrides. Avoid global runtime `-c` injection, which would silently override existing choices and broaden the fork delta.
2. **No credential migration on upgrade.** An older omitted setting can currently resolve to file credentials. Automatically adding keyring there could make existing credentials disappear from the client's selected view or switch authority. Keep the older-profile manual prerequisite documented until a separate, explicit, reviewed migration UX exists.
3. **No secret-store availability gate during environment creation.** Creating public configuration should still work when Secret Service is unavailable; actual MCP sign-in must fail safely rather than write plaintext. Describe the prerequisite accurately. A default does not install/unlock a Linux keyring or solve the deferred owner Ubuntu issue.
4. **Preserve independent inference auth.** Neither SSO inference bindings nor workspace API keys are changed. Both get the same new-profile MCP default; organizational MCP SSO remains separate.
5. **Keep current backend semantics.** Do not force `auth_keyring_backend = "direct"` or change an explicit encrypted Secrets backend. `keyring` requires native-backed protection under the existing selected backend, rather than promising every byte is physically inside Keychain for every possible configuration.
6. **Do not promise per-environment native MCP token isolation.** Matching connection name/URL records can be shared by the OS user across homes. This existing boundary still applies. Use unique test connection names and endpoints; do not touch owner entries.
7. **Do not broaden CLI override propagation.** The current manager child reloads persisted config and does not carry arbitrary parent `-c` overrides. Changing that unrelated behavior is outside this small default change. Fixture tests must set their deliberate overrides in the generated file, not rely on a parent-only flag reaching the child.

The alternative of changing upstream `OAuthCredentialsStoreMode::default()` or core resolution is larger, affects unrelated upstream users/tests, and risks legacy credential authority changes. The private configuration-template field keeps future upstream imports simple.

## Fixture compatibility work required

The runtime change is tiny; existing test setup needs deliberate updates to avoid duplicate top-level TOML keys:

| Owned fixture/validator | Current behavior | Bounded update |
| --- | --- | --- |
| `scripts/test_airs_mcp_manager.py:37` | Prepends `file` after `env create` | Replace/update exactly one top-level field in the synthetic fixture; retain this explicit File-mode workflow test. |
| `scripts/test_airs_harness_mcp_login.py:115` | Prepends `file` plus MCP tables after creation | Update the existing generated field, preserving the test's explicit File-mode intention and server tables. |
| `scripts/validate_gateway_mcp.py:336` | Prepends `keyring` after creation | For a new target version, verify the generated default or update idempotently if backward-version validation is required. Do not edit/run historical receipts or live acceptance now. |
| `scripts/validate_builtin_mcp.py:258` | Prepends `keyring` after creation | Same narrow idempotent/config-verification correction. |

The conformance adapter creates its own test configuration rather than using AIRS environment creation; do not rewrite upstream/conformance fixtures merely because they intentionally choose File. Prefer a tiny fixture-only top-level update that preserves TOML tables, with no new production dependency. Do not blindly prepend another key. Existing file-mode manager tests do **not** establish native-default behavior and must not be counted as that proof.

## Meaningful RED before the production change

Use the existing isolated HTTPS MCP gateway fixture and an isolated Linux native credential service boundary. Create an environment through the actual executable with **no manual MCP storage edit**, then attempt MCP sign-in while its private Secret Service is unavailable. The new requirement is: sign-in cannot report success or write a fallback credential file. On the frozen old behavior, Auto is expected to save into `.credentials.json` and/or report success, producing a behavioral RED. Verify that expectation by execution; do not claim RED based only on reading source.

A companion available-store fixture must sign in through `/mcp`, persist in a disposable native record and verify initialization/tool discovery without editing configuration. This catches ineffective defaults or development-version overrides. Merely asserting that the generated TOML contains a constant is not sufficient acceptance and would mirror the one-line implementation.

## Acceptance matrix

| Scenario | Required observation | Evidence kind |
| --- | --- | --- |
| Fresh explicit create; no native service | Creation succeeds without opening native store or sending OAuth; later MCP login fails safely and creates no plaintext fallback | Installed Linux isolated fixture |
| Fresh guided `env create` | SSO or workspace-key choice reaches the same generated native mode; cancellation preserves expected pre/post-creation boundaries | Existing terminal fixtures extended behaviorally |
| Fresh bare `airs` welcome | Newly created profile can add/sign in through `/mcp` without leaving to edit config | Installed terminal/native-store fixture |
| Native store available | Gateway OAuth persists to unique disposable native record; no fallback token file; initialize/tools-list succeeds; Start new conversation remains required | Linux x64, native Linux ARM64 and signed Apple Silicon exact-package checks |
| Native store denied/unavailable | No successful connection claim, no file fallback, existing inference/history/default unchanged, actionable existing recovery remains | Deterministic private Linux failure fixture plus applicable existing backend tests; do not label an unrun Mac denial scenario passed |
| Existing explicit `file`, `auto`, `keyring` | Opening/selecting/renaming/diagnosing and failed duplicate-create do not rewrite config, token store, bindings or history | Byte/hash preservation fixtures, no real owner credentials |
| Existing omitted mode + fake file token | Upgrade leaves omission and credential file intact; no silent migration or native-store write on startup | Installed upgrade fixture from mcp.3 |
| Two environments and default switch during UI | New-profile operations remain bound to the displayed home; other profile/config/inference/history unchanged | Existing manager isolation scenario reused |
| Explicit fixture File mode | No duplicate TOML key; existing file-mode callback/cancel/reconnect regression suite still runs | Existing scripted fixture suite |
| Cancellation/failure after callback | No callback/token leakage into transcript, inference headers or histories; no accidental repeated tool request | Existing manager/manual-callback checks plus new native path |
| Packaging/docs | Three-platform source/hash/signing evidence retained; docs distinguish new-default version from older profiles; stable tags unchanged unless separately authorized | Normal next test-release pipeline |

Creation must not make native-store calls just to persist a policy choice. Native positive tests use isolated disposable accounts/unique record keys; Linux failure tests use a private D-Bus service/session, never the owner's session. Real human SSO, workspace-key and ServiceNow acceptance remains a separate attended activity.

## Proposed implementation sequence and gate

1. Capture the failing installed behavior against frozen mcp.3 using synthetic credentials only; preserve version/native digest and RED log.
2. Add the single owned configuration field; update the four affected fixture/validator setup paths without changing their intended storage modes.
3. Run focused CLI/creation preservation tests and installed native MCP fixtures. Reuse existing MCP manager/manual-callback regressions. Run only affected package tests; no new core/protocol edit means no new full-workspace trigger.
4. Independently review native failure behavior, legacy preservation and lack of upstream/shared-module changes before broad platform packaging.
5. Build and validate a new test version on the three supported native targets through the normal release process. Update canonical/package guides only after behavior is verified, keeping older-profile instructions.

Suggested scoring remains completeness 3 + capability 3 + best practices 2 + optimization 2. A score of at least 9 requires executed native success/failure evidence, unchanged existing settings/credentials, RED/GREEN, no fallback leak in the unavailable-store case, and no new upstream config/auth/protocol patch. Documentation-only or constant-only tests cannot satisfy capability points. No score is assigned by this read-only plan.

This is a good next focus because it removes a visible first-session shell detour with a minimal private change. An in-app opt-in conversion flow for older profiles, broader native-store provisioning UX, or credential migration deserves its own later design and should not be quietly bundled into this default.
