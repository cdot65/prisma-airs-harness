# 0.1.0-alpha.14 — gateway-mediated native MCP (internal alpha)

Both inference and native MCP target Prisma AIRS AI Gateway. The existing Codex
MCP client connects to the gateway listener, CAS handles gateway-facing company
SSO, and the gateway manages confidential upstream OAuth. Native dynamic client
registration preserves the existing inference environment and history.

Both exact installed platform candidates passed production native login,
credential storage, all eight model-selected tools and correlated gateway and
upstream observations. Native suites, npm upgrades and Apple signing/notarization
pass. Gateway-held upstream renewal was observed after expiry. The owner
authorized publication with hourly frontend refresh unverified; zero cycles
completed. Idle sessions can require fresh login after 30 minutes. See
[PUBLICATION.md](PUBLICATION.md) for the preserved failed lifecycle receipts and
limited release scope.

# 0.1.0-alpha.13 — built-in OAuth MCP and verified npm upgrades

> **Acceptance correction:** This published release was tested against a direct
> upstream MCP endpoint. It does not establish the required AI Gateway MCP proxy
> and CAS integration. See [GATEWAY_MCP_REMEDIATION.md](GATEWAY_MCP_REMEDIATION.md).

Published as `airs-harness@latest` on Verdaccio for Linux x64 and Apple Silicon.
Use the normal command's existing `mcp add` and `mcp login` workflow. The gateway
now receives callable flat tool declarations while canonical history retains
namespaces. Explicit read scopes and grant-preserving refresh support Redtail's
OAuth policy. Adding native MCP preserves inference history and credentials.

Both platforms passed all eight production tools, native OAuth with two
concurrent expiry cycles, npm upgrades, installed terminal inventory and fresh
registry integrity checks. Mac Developer ID signing and Apple notarization passed
again after registry installation. See [MCP.md](MCP.md) for onboarding and
[PUBLICATION.md](PUBLICATION.md) for the evidence and bounded alpha release scope.

# 0.1.0-alpha.12 — Codex 0.154 integration and signed Mac distribution

Published to Verdaccio as `airs-harness@latest` for Linux x64 and Apple Silicon.
Codex 0.154 is integrated while retaining the AIRS gateway, separate MCP
authorization, credential/environment isolation, CLI 5.2.0 / SDK 0.28.0 and
eight embedded Prisma skills. Deferred upstream services remain constrained.

The Mac package is Developer ID signed and Apple notarized. Forgejo build/signing
and package E2E passed; fresh production-registry installations passed 43 Linux
and 42 Mac executable checks (one Linux-only skip on Mac). See [PUBLICATION.md](PUBLICATION.md)
for artifact provenance and the pending full-suite/independent-review gates that
the owner acknowledged before authorizing this alpha publication.

# 0.1.0-alpha.9 — managed Prisma AIRS CLI

The npm harness now requires CLI 5.2.0 (SDK 0.28.0) and embeds eight focused
skills. `airs-harness airs ...` works before gateway setup; agent shell tools
receive an absolute managed CLI path. Missing/mismatched dependencies fail
explicitly. Project `.env` loading is disabled for managed CLI invocations.

Linux passed 27 native tests plus one npm-only skip and 27 installed npm tests.
Apple Silicon passed 26 native tests plus two platform/package skips and 26 npm
tests plus one Linux-only skip. Both exercised the embedded CLI skills through
the installed harness. Real CLI command/configuration/DLP generation checks
passed on Linux, Apple Silicon and Windows; 53 scoped Rust tests passed.
Live Linux checks exercised the new CLI skill, default/explicit inference,
local edits/tests, remote MCP scans and resume. Read-only management/scanner
preflight results are recorded separately from mutation and coverage claims.

See [PUBLICATION.md](PUBLICATION.md) for current publication/Keychain status,
[PRISMA-AIRS-CLI.md](PRISMA-AIRS-CLI.md) for authentication and upstream CLI limits,
and [MACOS.md](MACOS.md) for installation. The DLP environment-only credential
exception is documented in the guide; carry it into the next embedded skill
update. Windows native distribution and broader Mac live enterprise acceptance
remain separate milestones. Historical release evidence follows.

# 0.1.0-alpha.8 — Prisma AIRS Harness npm release

Prisma AIRS Harness is published as `airs-harness@0.1.0-alpha.8` at
`https://npm.cdot.io`, with prebuilt Linux x64 and Apple Silicon packages.
Downloads are anonymous on the LAN/VPN. See [MACOS.md](MACOS.md) for installation,
Keycloak sign-in, scanner setup and hands-on testing.

The rename preserves existing environments, native credential namespaces,
deployed auth IDs and session history. The npm launcher forwards arguments and
signals to the native executable without install scripts or source compilation.
It now forwards subsequent signals after an interrupt and rejects Intel Macs.

Mac acceptance passed 26 native tests plus one Linux-only skip, 25 npm-installed
tests plus one Linux-only skip, six launcher checks, native Keychain persistence,
and the installed CLI secure-login/tool/logout lifecycle. Published tarballs match
staged integrity values. A clean unauthenticated production-registry install
verified the Linux native hash and bundled Mac guide. [PUBLICATION.md](PUBLICATION.md)
and [VALIDATION.json](VALIDATION.json) separate new distribution evidence from
prior Linux live/Rust checks and remaining Mac enterprise hands-on acceptance.

The validated Mac executable was reused for a 4m45s acceptance/packaging job;
no Rust rebuild was needed. Future Mac builds and packages are Apple Silicon-only.
Historical releases below retain their original names and evidence.

# 0.1.0-alpha.7 — authentication workflow polish

When a user resumes from a new Linux shell without the unlocked D-Bus keyring
session, the error now explains how to recover. macOS and Windows receive native
credential-store guidance. No credentials or history are migrated.

MCP status reports a configured credential helper without executing it or
claiming that its token is valid. Runtime connection status remains independent.
The app-server v2 auth enum adds `credentialHelper`; clients using exhaustive
parsing must support this value. Stable and experimental exports are regenerated.
A repeated logout reports that the environment is already logged out locally,
while retaining MCP cleanup and existing fail-closed behavior. Doctor no longer
mislabels an OIDC credential as a workspace identity.

Validation passed: 5,914 affected Rust tests (14 skips), 21 executable/PTY
fixtures on the stripped optimized binary, and seven-turn live workflows before
and after installation. Both live runs displayed Credential helper, exercised
local code/tests and default/explicit/default routing, and completed three real
scanner calls. Installation preserved 11 checked configuration/binding files and
retained an alpha.6 rollback executable. The full upstream suite remains blocked
by the missing V8 musl archive; code mode is disabled. No macOS/Windows executable
or new native credential-store platform run is claimed by this polish release.

The owner confirmed the alpha.6 E2E workflow, including resumed execution, after
unlocking the original keyring. Alpha.7 checks are recorded separately in
VALIDATION.json and the post-publication RELEASE-VERIFICATION.json asset.

# 0.1.0-alpha.6 — single-realm user authentication

Browser PKCE and device login now authenticate users through the existing
`truffles` realm. Separate inference and MCP clients/audiences use the same JWKS
and user identity. AIRS receives the original signed JWT and applies its routing,
permissions and mandatory scans. Default requests omit `model`; explicit
`@provider/model` selections remain intact.

Native encrypted credential storage, durable refresh state, replay protection,
logout and stable user/history bindings are implemented. Linux Secret Service,
macOS Keychain and Windows Credential Manager passed native CI. The distributed
executable is Linux x86-64. Keycloak uses an opt-in database-backed refresh ledger
for the two Terminal clients; its image is pinned and upgrade protocol tests are
required because the extension uses internal Keycloak SPIs.

Self-review caught excessive parallel shell calls despite successful authentication.
The client now requests serial calls for gateway routes with unresolved model
capabilities. Direct candidate and installed acceptance used two and four shell
commands respectively, within the four-command test budget. This is not a general
runtime command cap or a guarantee of model quality on arbitrary tasks.

Validation of the optimized binary:

- 20 executable/PTY fixtures passed, including routing, local tool continuations,
  context defaults, compaction and logout/environment isolation.
- 27 live OIDC checks passed before and after installation: browser/device login,
  different-user rejection, separate resource JWTs, local files, real scans,
  default → explicit → default model switching, refresh in the same terminal,
  logout and identity-preserving reauthentication.
- Seven workspace-key turns passed with model switching, independent arithmetic
  assertions, tests and three actual scans.
- 3,929 supported core tests passed; scoped Clippy passed without runtime changes.
  The earlier 323 affected authentication-crate checks and three-OS native-store
  CI remain applicable to unchanged identity code.
- 20 live MCP authorization cases and 15 filter tests passed. Empty resource and
  template inventories work; resource content, subscriptions and unlisted tools
  remain denied. Scanner SDK tests passed and production npm audit found zero issues.
- Persisted AIRS input/output security scans and SCM request telemetry matched
  three signed subjects and traces, including a metadata-spoofing case. SCM cost
  and usage fields are present. Management connectivity and `airs doctor` pass.

The binary is installed as `~/.local/bin/airs-terminal`. Installation preserved
5,589 persistent state files and retained alpha.5 for rollback. Both public clients
are enabled and only the verified owner `cdot` has their group roles. Owner login
and hands-on acceptance remain manual; use
[OWNER-REVIEW.md](administration/identity/OWNER-REVIEW.md).

The release's `RELEASE-VERIFICATION.json` supplies post-publication archive/CI
verification. Source evidence and operator fixtures are under
`administration/identity/` and `validation/2026-09-07/auth-release/`.

Known boundaries: the full upstream workspace build fails on the missing V8
150.4.0 musl archive (HTTP 404); code mode is disabled in this release. Full
macOS/Windows terminal distribution, artifact signing and optional Conjur
workload-secret activation remain separate milestones. Conjur manifests are
prepared but require the authorized policy-administrator credential. Local user
refresh tokens remain in the OS store. The historical failed native gateway
log-read attempt is not the working SCM telemetry path.

The earlier exposed inactive Koi client key was retired. Realm keys and SAML
metadata were preserved, with signed response verification before and after;
the external Koi application was not contacted.

The prior release history follows for provenance.

---

# 0.1.0-alpha.5 — interactive model-switch repair

Owner E2E testing found a real alpha.4 integration failure: selecting GPT-4.1
through `/model` fabricated a default reasoning effort from an empty capability
list. GPT-4.1 rejected the resulting `reasoning.effort`, and the picker could remain
open. The earlier provisional 9/10 assessment is withdrawn. Startup-route tests
had missed this interactive path; their passing receipts remain historical evidence.

## Changed behavior

- Gateway model selection clears reasoning effort when no levels are advertised
  and dismisses the picker. Switching back to the gateway default also works.
- The inference request boundary omits effort values unsupported by the selected
  gateway capability entry, including stale persisted settings. Explicitly
  advertised reasoning levels remain available. Upstream non-gateway behavior is
  preserved; no provider-specific model names are hard-coded in the terminal.
- Runtime instructions distinguish remote inference from local tools, requested
  routes from verified backend identity, MCP tools from resources, and skills
  from executable integrations. A deterministic, bounded summary of enabled MCP
  server names and tool allowlists contains no URLs or credentials. It is labeled
  as configuration, not a live health check.
- Existing environments receive this runtime context without rewriting their
  pinned model catalogs, credentials or session history.

## Acceptance

The release requires the actual PTY path default → explicit → default, not only
separate process launches with `-m`. Deterministic tests inspect request payloads
for routing and unsupported effort, and preserve advertised effort in a separate
positive test. The live script asks the owner's model/codebase/MCP questions,
changes calculator code, independently checks tests and arithmetic, and invokes
MCP scans across model changes. Natural-language self-descriptions are retained
for human review and are not authoritative backend identity metadata.

The exact optimized binary passed 20 executable fixtures; the live seven-turn
interactive replay completed both model switches, five calculator tests and three
real scans. A separate two-route agent probe passed local skill/edit/test/MCP work
and resume. Alpha.5 also resumed an isolated alpha.4 session after reproducing its
live unsupported-effort failure. Installation preserved all ten fingerprinted
owner configuration/history files and retained alpha.4 for rollback.

Scoped source checks passed: 23 core tests, 4 home-directory tests and 4,090 TUI
tests (6 skips; one existing timing test passed on retry). Scoped Clippy passed.
The first final replay exposed raw-keyboard paste timing in the test driver; it
left a prompt in the composer. Explicit bracketed paste fixed the driver, and all
20 fixtures plus the full live replay passed again with the same binary.

Runtime descriptions improved on the reported model/MCP errors. Generated prose
still includes generalizations and is not authoritative routing, repository or
connection-health metadata; use configuration, source files and `/mcp` for those.

Final artifact evidence is recorded in `VALIDATION.json` and the release's
`RELEASE-VERIFICATION.json`. The earlier alpha.4 notes and receipts are retained
under `validation/2026-09-07/`; they do not certify this binary.

```sh
python3 scripts/validate_live_model_switch.py \
  --binary "$HOME/.local/bin/airs-terminal" \
  --gateway-url https://airs.cdot.io/v1 \
  --credential-file "$HOME/workspace.txt" \
  --mcp-url https://mcp-airs.cdot.io/ws-develo-71f8d8/airs-terminal-security/mcp \
  --mcp-credential-file "$HOME/.airs-terminal/credentials/mcp-workspace.key" \
  --output-directory /tmp/airs-terminal-new-interactive-check
```

The output directory must not exist. The probe uses isolated state and leaves the
owner's active environment and review project alone. After an installed upgrade,
exit the old running process and start the new binary; use `resume` to reopen the
existing session. A process already running retains the old executable code.

## Gateway compatibility transforms

The deployed AIRS 2.20.0 implementation was inspected in response to the owner's
suggestion to remove unsupported fields centrally. Its `default.regexReplace`
guardrail changes message text, not root JSON parameters. Its
`default.requestParametersCheck` validates/rejects parameters. The
`default.webhook` guardrail accepts transformed request JSON. Further inspection
also verified native target `drop_params` for simple key removal, before guardrail
context creation; four local transform checks passed. Prefer that mechanism for
model-specific removal and reserve a webhook for more complex transformations.

A future compatibility policy should match resolved provider/model capabilities,
remove only known unsupported fields, preserve supported reasoning, and retain
mandatory input/output security scans. Never use regex against arbitrary JSON or
silently strip reasoning from every model. This terminal repair itself does not
change the live gateway routing configuration or the scanner deployment.

## Remaining release boundaries

This remains the authorized standalone Linux workspace-key pilot. Keycloak login
and refresh, individual-user authorization/audit, macOS, native CI builds/signing,
and the complete upstream V8-dependent suite remain unfinished. The unavailable
V8 musl archive prevents full workspace validation on this host. Existing scanner
image findings (20 unfixed medium/low, no critical/high) and the broader workspace
scc warning outside the Linux CLI graph remain documented in the earlier audit.
Owner hands-on acceptance is required; a self-assigned score does not replace it.

## Publication verification

[Independent Ubuntu release CI](https://github.com/cdot65/prisma-airs-terminal/actions/runs/34111335276)
passed against immutable tag `airs-terminal-v0.1.0-alpha.5`: 20 packaged executable
fixtures, four scanner tests and zero production npm findings. The downloaded
archive's binary and source hashes match the installed binary and release
provenance. `RELEASE-VERIFICATION.json` and its checksum were uploaded, downloaded
again and verified. Owner hands-on revalidation remains pending.
