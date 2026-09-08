---
title: Prisma AIRS Harness cross-platform authentication release plan
description: Guided authentication, native credential lifecycle, platform acceptance, and evidence-based release scoring.
plan_version: 0.1
status: draft-interview-pending
created: 2026-09-08
updated: 2026-09-08
baseline: 0.1.0-alpha.9
baseline_commit: 7b7b03de5b5d55fdb09ea90e379985d95ce8552e
owner: cdot65
tags:
  [
    authentication,
    authorization,
    keycloak,
    credentials,
    macos,
    linux,
    windows,
    acceptance,
  ]
---

# Cross-platform authentication release

The implemented candidate command flow is documented in
[Private candidate setup and sign-in](AUTHENTICATION-ONBOARDING.md). That guide
does not establish release acceptance or completion of this plan.

## 1. Outcome and authority

A teammate installs Prisma AIRS Harness, runs `airs-harness`, follows its setup
and login prompts, and completes a real gateway task. Returning in a new terminal
retrieves credentials securely and resumes the correct environment. Routine
onboarding requires no shell recipes, environment exports, credential pipes,
manual Keychain administration, or D-Bus commands.

The owner requested this plan on September 8 after alpha.9 failed both Keycloak
and workspace-key storage on an unlocked Mac. This is a planning deliverable;
it is not evidence of implemented functionality or authorization to treat any
unanswered interview choice as confirmed. It replaces older Linux-first and
workspace-key-fallback completion criteria for this authentication release.

Canonical implementation repository: `cdot65/airs-harness`. Product name:
**Prisma AIRS Harness**. Command: `airs-harness`. GitHub package:
`@cdot65/prisma-airs-harness`. Hosted PAH remains outside the runtime dependency
graph. Preserve the pinned Prisma AIRS CLI dependency and capability skills.

**Release rule: every required gate passes, every supported platform scores at
least 9.0/10, and an independent reviewer verifies the evidence. Anything below
9.0 requires remediation and another assessment.** Agents cannot reduce scope,
remove tests, or change weights to raise their own score.

## 2. Verified baseline and unanswered decisions

| Finding at the baseline commit                                                                              | Consequence                                                                                   |
| ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `cli/src/airs_credentials.rs` rejects interactive stdin for workspace-key login and requires a pipe.        | Implement a masked prompt in the application. Preserve explicit automation input separately.  |
| Workspace and OIDC errors discard the original native storage failure.                                      | Add safe, structured operation and platform diagnostics before choosing a backend fix.        |
| Both authentication modes use native storage; workspace saves and OIDC preflight fail before completion.    | Switching authentication modes is not a storage repair.                                       |
| `keyring-store` uses Apple's native backend; Mac release workflow applies ad-hoc signing.                   | Keychain support exists. Diagnose the actual error; establish release signing continuity.     |
| OIDC uses bounded chunked storage, while workspace-key storage uses the direct backend.                     | Verify backend size limits for both paths; do not infer Windows parity from OIDC tests alone. |
| Native fixtures passed on three OS families; published Mac alpha.9 passed hosted checks on macOS 15 and 26. | These are useful baselines, not desktop onboarding or upgrade acceptance.                     |
| The actual user's Mac failure is neither reproduced nor resolved.                                           | The reported scenario stays a mandatory regression gate.                                      |
| Full published distributions exist for Linux x64 and Apple Silicon; Windows native-store tests exist.       | Windows needs a complete installed executable/package acceptance path.                        |

Source locations: [credentials](codex-rs/cli/src/airs_credentials.rs),
[OIDC](codex-rs/cli/src/airs_oidc.rs),
[native backend](codex-rs/airs-identity/src/storage/platform.rs),
[chunks](codex-rs/airs-identity/src/storage/chunks.rs),
[Mac release](.github/workflows/airs-harness-macos-release.yml),
[published Mac evidence](validation/2026-09-08/macos-storage-diagnostics/CI.json).

| Decision                     | Draft recommendation                                                                                                                        | Resolution required                                                                                                              |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| D1: Linux SSH/headless scope | Include a first-class headless track, with its persistence contract selected at P0.                                                         | Owner interview pending; do not declare headless delivered from desktop tests.                                                   |
| D2: Organization onboarding  | Organization-provided nonsecret connection profile, plus custom setup.                                                                      | Owner interview pending; use this as the draft UX assumption.                                                                    |
| D3: Distribution trust       | Apple Developer ID/notarization and Windows Authenticode signing with protected release credentials.                                        | Access availability unknown; collect availability only, never secrets in chat or source.                                         |
| D4: Platform baseline        | Apple Silicon macOS 15 and 26; Windows 11 x64; Ubuntu 24.04 LTS x64 GNOME and KDE sessions.                                                 | Freeze exact OS builds, terminal versions, Node versions and Linux service implementations at P0.                                |
| D5: Headless persistence     | Prefer an organization-provisioned secure service when present; otherwise evaluate a maintained encrypted store with an in-app unlock flow. | Security/design review at P0 must select a concrete implementation before P3. No custom cryptographic format or silent fallback. |

Intel Mac builds and packages remain prohibited. Windows ARM64, Linux ARM64,
older OS versions, and unattended service accounts are separate scope decisions;
the launcher must state unsupported combinations accurately. WSL is a Linux
execution environment and must not silently use the Windows user's credentials.
Do not advertise WSL support until its selected storage/session contract passes.

## 3. User journeys and command contract

The following is the proposed command behavior, not a claim about alpha.9.
Existing explicit flags remain compatible unless a reviewed migration requires
a documented change. Help, snapshots, guides and shipped packages must agree.

| Journey             | Required behavior                                                                                                                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| First launch        | `airs-harness` offers setup when no environment exists; choose organization profile or custom gateway, name environment, choose allowed authentication method.                                          |
| Organization setup  | `airs-harness setup` accepts a profile link/file through the wizard. Show organization, gateway origin, issuer and requested capabilities before trusting it.                                           |
| Workspace key       | `airs-harness login` offers a hidden, paste-friendly input field. Validate token shape, store securely, verify readback, then check gateway access.                                                     |
| Company sign-in     | Open the system browser for Keycloak. The harness never collects the user's Keycloak password. Offer device authorization where the configured issuer supports it.                                      |
| Native consent      | Let the OS display its authorization/unlock UI when required. Explain the action before the prompt. Treat cancellation as cancellation, preserve existing working state.                                |
| Return/resume       | Use saved environment settings and credentials automatically; refresh as needed. If reauthentication is necessary, launch the same guided login without requiring issuer flags again.                   |
| Change identity     | Explain environment isolation and offer to create a new named environment. Never silently attach old history to another user, gateway or workspace credential.                                          |
| Optional MCP        | Offer profile-authorized servers before the first coding session; authenticate separately and show per-resource status. Inference-only access must remain usable if optional MCP is declined or denied. |
| Credential rotation | Guide workspace-key rotation through a new environment under the current fingerprint contract. Same-user OIDC refresh keeps the existing environment/history.                                           |
| Logout              | Disable future use for that environment, remove local secret records, attempt supported refresh revocation, and report incomplete remote revocation accurately.                                         |
| Diagnose            | `airs-harness doctor` shows storage, identity, gateway access and optional MCP as distinct results; offer a redacted diagnostic bundle from inside the application.                                     |

The primary guides contain the install command and application commands, not
shell scripts for credentials. Registry login, AIRS login and optional SCM/scanner
configuration must be clearly separated. Management credentials are not needed
to chat, edit local files, or use an already authorized remote scanner.

### Honest success states

Use explicit states: `not-configured`, `signed-out`, `authenticating`,
`credential-saved`, `gateway-verified`, `reauthentication-required`, and
`action-required`. Display resource-specific authorization separately.
Saving a credential is not proof that it is authorized. A public health response
is not an authenticated gateway check. A successful gateway request is not proof
of MCP permissions. If the network is unavailable after secure storage succeeds,
say "Credential saved; gateway access not yet verified" and offer Retry.

Validate access with a documented, bounded authenticated capability endpoint
when available. Otherwise disclose and make one minimal inference probe with
fixed harmless input and a strict output limit; record its request ID. Do not
invent an endpoint, silently run arbitrary tools, or repeatedly spend tokens.
Optional MCP validation must use an explicitly selected harmless operation.

## 4. Authentication and authorization invariants

- Retain one existing Keycloak realm and one JWKS endpoint. Inference and MCP
  keep separate public clients/audiences and server-enforced permissions.
- Use system-browser authorization code plus S256 PKCE, state and nonce checks;
  bind callbacks to loopback with short-lived listeners. No client secret in the
  executable, no password grant, and no embedded webview collecting IdP passwords.
  Follow [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252).
- Device login handles expiry, denial, cancellation, polling interval and
  `slow_down`; require advertised/configured support and retain the deployed
  Keycloak PKCE extension already verified by this project. See
  [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628).
- Verify signature, issuer, resource audience, client binding, subject and token
  lifetime. Preserve rotation/replay protections and test JWKS rotation; apply
  current [OAuth security guidance](https://www.rfc-editor.org/rfc/rfc9700).
- AIRS validates authorization and routing. Client-side JWT decoding is display
  or validation assistance, never the authority for granting access. Preserve
  original user attribution and mandatory gateway policies.
- Default routing omits `model` from the payload; authorized explicit selection
  sends `@provider/model`. No credential, profile or model picker may override
  mandatory gateway guardrails or introduce an alternate inference destination.
- A workspace key establishes workspace access, not individual user identity.
  Inference credentials must not be reused implicitly for MCP, SCM, or scanning.
- Local filesystem/tool approvals remain separate from gateway authorization.
  Changing identity must not widen local tool permissions.
- Secrets must not enter logs, diagnostics, command arguments, shell history,
  environment files, model context or tool-result transcripts. Deliberate
  automation references remain advanced opt-in interfaces, not onboarding fixes.
- Native stores protect secrets at rest; they are not isolation from every
  program running as the same user. Enforce credential-helper boundaries and
  inherited-environment filtering; preserve the local execution sandbox. Do not
  claim protection from a compromised OS, administrator or arbitrary same-user code.

Connection profiles are versioned configuration, never credentials or executable
instructions. Validate HTTPS origins, issuer consistency, size limits and schema;
reject shell hooks, arbitrary credential-helper commands, embedded secrets,
insecure TLS options and routing-policy overrides. Profile redirects cannot
change trust origin silently. Profile updates that change destinations or
identity require explicit review and a compatible/new environment. A downloaded
profile is untrusted until its provenance is verified or the user confirms it.
Use a pinned organizational signing key for managed automatic profile updates.

The P0 profile schema must cover a schema version, display name, gateway URL,
allowed login methods, OIDC issuer/client/audience, optional default qualified
route, and optional MCP entries with endpoint, resource audience, client, tool
allowlist and required/optional status. These are nonsecret administrator values.
Review a concrete valid example and rejection examples before implementing the
wizard. Initial profile trust must be established separately from the profile's
own contents; a signing key supplied only by that same untrusted file is not
proof of provenance. Never forward authorization headers across a redirect to
another origin. No profile content may loosen local tool approval rules.

## 5. Credential architecture and platform behavior

Use one application-level credential lifecycle with platform adapters. Reuse the
existing Rust identity/storage crates; keep UI orchestration in focused CLI/TUI
modules. Introduce typed outcomes such as `locked`, `denied`, `cancelled`,
`service-missing`, `session-unavailable`, `corrupt-record`, and `write-failed`.
Preserve the raw OS status internally; expose only allowlisted, bounded metadata
and redacted context. Do not print unrestricted upstream error strings.

Separate noninteractive availability checks from explicit save/read/delete probes.
`status` must not trigger surprise OS prompts or network requests. The login flow
may perform a clearly announced probe using a unique nonsecret item and must
clean it up. A capability probe must never replace an existing credential.

Save transaction: stage new credential -> persist -> verify readback -> atomically
commit binding -> clean obsolete generation. Preserve the previous usable record
if replacement fails. New unverified network authorization is displayed as such.
Refresh requires per-binding concurrency control, durable pending state and no
replay of a possibly consumed token after an ambiguous failure. A new login may
be required after an interrupted rotation; explain this without deleting history.
Do not simplify the existing manifest/chunk mechanism without crash tests.

Credential prompts are owned by the trusted interactive application, never by
the model or its tool subprocesses. Background/noninteractive commands fail with
a stable actionable result instead of waiting indefinitely for input. Logout
does not revoke a shared workspace key globally or end unrelated Keycloak browser
sessions; global account/session administration is a separate explicit action.

| Platform           | Implementation direction                                                                                                                           | Required user experience and constraints                                                                                                                                                                                                      |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS              | Diagnose the current Apple-native backend first; retain stable service/account identifiers and use a consistent release signing identity.          | Native consent when needed; no Keychain Access window requirement. Test actual npm-installed path, Node version-manager paths, default user keychain, lock/unlock and upgrade. Never set broad item ACLs or disable Gatekeeper.               |
| Windows            | Credential Manager under the interactive user's identity with machine-local persistence where policy permits.                                      | No administrator terminal or credential-manager setup. Test PowerShell and cmd, Unicode/spaced paths, separate user accounts and restart. Native errors must distinguish policy denial and unavailable logon sessions.                        |
| Linux desktop      | Secret Service over the user's existing session bus; complete the service's unlock/prompt protocol.                                                | GNOME and KDE are separate acceptance environments. Do not start a conflicting bus or replace the user's keyring. Missing dependencies are detected during supported installation/setup, with an owned provisioning path.                     |
| Linux SSH/headless | P0 selects the supported encrypted persistence mechanism and unlock behavior; device authorization opens on another trusted device when necessary. | One harness-owned flow, with a clear in-app unlock step where needed. No `dbus-run-session`, exported passwords or manual daemon lifecycle instructions. Reconnect and reboot are mandatory tests if persistent headless support is selected. |

Windows storage belongs to the current logon context; network logons may lack a
credential set. Use [CredWriteW](https://learn.microsoft.com/en-us/windows/win32/api/wincred/nf-wincred-credwritew)
and actual backend limits when designing tests. Validate full permitted workspace
key sizes and large OIDC bundles, including interrupted writes; do not assume
one fixed token shape will represent production.

Linux Secret Service may return a prompt object that the client must invoke and
await. Test lock, unlock, dismissal and unavailable services through the real
protocol. See the [Secret Service specification](https://specifications.freedesktop.org/secret-service/latest-single/)
(the retrieved latest version is a draft; freeze the implemented contract at P0).

Apple distinguishes the file-based and data protection keychains. A migration
to data protection storage requires an explicit signing/entitlement/packaging
decision and legacy-item migration, not just a crate feature toggle. Prefer a
bounded repair of the existing backend until evidence demonstrates the need for
that migration. See [Apple TN3137](https://developer.apple.com/documentation/technotes/tn3137-on-mac-keychains).
Developer ID signing/notarization is a distribution trust requirement; it is
not an established diagnosis of the current failure. Stable signing identity
must survive upgrades; see [Apple TN2206](https://developer.apple.com/library/archive/technotes/tn2206/).

### Headless decision gate

An SSH session with no unlocked credential service has no implicit secret that
can unlock encrypted storage. Hiding shell commands cannot create that trust.
At P0 choose an organization-managed unlock mechanism or a maintained encrypted
store with an application-owned passphrase prompt, recovery policy, audited
dependencies and a documented threat model. Never store the unlock key beside
the encrypted credentials, collect the user's OS/IdP password as a vault password,
or silently persist plaintext. If no acceptable mechanism is available, mark
headless persistent delivery blocked and continue independent desktop work.
Session-only sign-in is a separate explicit product option requiring owner
agreement; it cannot silently substitute for promised persistent sessions.

## 6. Measurable acceptance catalog

Each row is a required acceptance case, not a suggested test. At P0 expand rows
into executable subcases and freeze the denominator per platform. Rows that say
"both" require workspace and Keycloak evidence. Synthetic protocol tests,
native-store tests, installed fixtures and live user trials have distinct evidence
labels; none can substitute for the next layer. Failed/unknown/skipped required
subcases count as not passed. Mark scenario applicability before execution.

| ID  | Observable pass condition                                                                                                                                                                                              | Evidence layer                                                        |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| A01 | Fresh install resolves the correct native package and pinned CLI; command runs as a standard user; no Rust build, permission override or manual shim cleanup.                                                          | Published package, each supported platform                            |
| A02 | Workspace login completes through a masked in-app prompt; no echo in normal, pasted, cancelled or invalid-input cases; terminal state restored after interruption.                                                     | PTY/ConPTY plus desktop user                                          |
| A03 | Profile-driven Keycloak browser login succeeds; no issuer/client/audience flags needed after setup; invalid state/nonce/PKCE and callback injection rejected.                                                          | Protocol plus installed/live                                          |
| A04 | Device flow succeeds where supported; expiry, denial, unsupported flow and cancellation leave a recoverable UI and no false success.                                                                                   | Protocol plus selected headless/live                                  |
| A05 | Both credential modes save/read/delete successfully across three separate processes and a logout/login or reboot cycle; records cannot cross OS users.                                                                 | Native and installed desktop                                          |
| A06 | Locked, denied, cancelled, absent-service, wrong-session and corrupt-record cases produce distinct safe errors; existing good credentials survive failure.                                                             | Fault injection plus native scenarios                                 |
| A07 | The owner's unlocked-Mac failure is diagnosed with an OS status and resolved on the affected machine or faithfully reproduced environment; owner retest remains explicit.                                              | Incident regression and owner evidence                                |
| A08 | Both modes complete a real gateway request; bad/expired keys and denied users fail without weakening policy; network outage is not misreported as bad credentials.                                                     | Live gateway and fault injection                                      |
| A09 | Correct issuer/audience/client/role/scope accepted; missing/wrong/forged/expired claims and header/config spoofing denied; identity visible in persisted server audit.                                                 | Live identity plus server evidence                                    |
| A10 | Gateway default request omits `model`; authorized explicit model works; incompatible parameters remain normalized; unauthorized routes are denied.                                                                     | Wire fixture plus live model switch                                   |
| A11 | Authorized MCP tool returns a verifiable real result; wrong audience/scope/identity denied; inference credential cannot authorize MCP implicitly. Optional MCP denial leaves inference usable.                         | Live resource authorization                                           |
| A12 | OIDC continues past two access-token expiries; concurrent refresh consumes a token only once; interruption/storage failure never replays a consumed refresh token.                                                     | Protocol faults, native persistence and live TUI                      |
| A13 | Same-user restart and resume preserve history; another environment/user/gateway cannot read or resume it as the old identity; guided reauthentication retains compatible history.                                      | Installed executable                                                  |
| A14 | Logout prevents subsequent requests from all harness processes for that environment within 5 seconds; cached helpers/MCP clients are invalidated and in-flight calls cancelled where possible.                         | Two-process inference/MCP test                                        |
| A15 | Remote revocation success/failure reported accurately; offline logout still disables local use; captured access-token validity is bounded by server policy and measured, never called instantly revoked without proof. | Live plus offline fault                                               |
| A16 | Fake credential markers absent from logs, files, arguments, history, diagnostics, model context and child tool environments; native encrypted records excluded from plaintext-content claims.                          | Automated leak checks and review                                      |
| A17 | Maximum supported workspace keys and OIDC bundles fit each backend; corrupt chunks, exhausted storage and interrupted replacement preserve safe state.                                                                 | Boundary/native crash tests                                           |
| A18 | Alpha.9 -> candidate -> subsequent build preserves credentials and sessions under stable signing identity; incompatible downgrade is rejected safely or migration is reversible.                                       | Actual installed upgrade/downgrade                                    |
| A19 | Tampered/oversized/untrusted profiles and destination-changing updates cannot execute code, leak credentials or silently change a bound identity.                                                                      | Schema/trust tests                                                    |
| A20 | Installation guide contains no mandatory credential shell recipe; an unfamiliar user can install, authenticate, run a task and resume using the guide and product only.                                                | Moderated task trial                                                  |
| A21 | App owns credential diagnostics; all injected failure classes retain operation, backend and safe platform code; no blanket "unlock your keychain" answer.                                                              | Error fixtures and redacted bundle review                             |
| A22 | Five unfamiliar participants per OS family all complete both auth methods without operator intervention; median setup-to-first-response <=3 minutes and maximum <=5 minutes.                                           | Timed user trials; human MFA delays also reported separately          |
| A23 | Already-authenticated local credential resolution adds <=500 ms p95 over 30 warm launches per platform; ordinary startup does not probe by writing/deleting credentials.                                               | Installed performance trace; network excluded and reported separately |
| A24 | All noninteractive tests terminate within configured deadlines; interactive waits show progress and support cancel; cancellation restores usable prompt within 2 seconds in 20 trials.                                 | PTY/ConPTY and fault tests                                            |
| A25 | Same published binary performs local skill/read/edit/test work, authorized model switch and MCP scan, then resumes after restart; assert actual file/test/scan effects.                                                | Complete live E2E per platform and auth mode                          |
| A26 | Secret-store absence on headless Linux is handled through the selected supported flow; reconnect/reboot preserves the promised persistence; no manual shell setup.                                                     | Clean server/SSH acceptance if D1 selects it                          |
| A27 | Mac signatures/notarization and Windows signatures verified after package extraction; Linux provenance/checksums verified; ordinary OS protection remains enabled.                                                     | Final published bytes                                                 |
| A28 | Fresh teammate installs with repository/package read access; no owner/admin npm credentials; intended private package inheritance verified.                                                                            | Registry access trial per native package                              |
| A29 | 20 consecutive automated installed credential lifecycle runs per platform have zero unexplained failures; first attempts and reruns are both retained.                                                                 | Reliability receipt                                                   |
| A30 | Linux GNOME and KDE native-service prompts work; Windows PowerShell/cmd and Mac Terminal/Ghostty work; special paths and denied native consent covered.                                                                | Desktop compatibility matrix                                          |

Freeze an authentication-only novice trial and a separate install trial so Node
downloads, registry access, user MFA and gateway latency remain visible instead
of being removed from an end-to-end claim. Report total wall time and phase times.
Performance metrics are proposed engineering targets; changing them requires a
versioned plan amendment before acceptance, with rationale, not retroactive edits.

Freeze bounded defaults with the test matrix: noninteractive local availability
checks return within 5 seconds; gateway access probes have a 10-second connection
timeout and 30-second overall deadline, including any retries. Never retry a
denied credential or ambiguous refresh automatically. Device polling follows
the issuer's interval and expiry. Human browser/OS authorization waits show a
cancel action and do not masquerade as a background hang. Authentication forms
must be keyboard-operable, label errors in text rather than color alone, and
restore terminal echo/input mode after cancellation or process interruption.

Test same-user package relocation and Node version-manager upgrades as well as
version upgrades: persisted credential-helper paths must resolve the current
trusted executable without asking users to edit configuration. A18 must exercise
the new binary's access to credentials written by the previous binary, not only
its ability to create fresh credentials under a new path/signature.

## 7. Implementation sequence and agent handoffs

| Phase                                     | Work and dependencies                                                                                                                                                                                                                                                | Exit evidence                                                                                                                |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| P0: Resolve design and reproduce          | Resolve D1-D5; inspect macOS error without secrets; inventory release signing access; freeze platform matrix, profile schema, headless mechanism, test subcases and ownership.                                                                                       | Decision records, incident reproduction/cause or explicitly unresolved status, evidence inventory. Do not guess the Mac fix. |
| P1: Credential lifecycle and diagnostics  | Add typed platform errors, safe probes and atomic replacement/readback; preserve namespaces/chunks; add failure fixtures. Can start independently of signing access after baseline capture.                                                                          | A06, A16, A17, A21 focused evidence and reviewed error UX.                                                                   |
| P2: Guided setup and login                | Add profile/custom wizard, hidden input, persisted nonsecret IdP settings, browser/device UI, cancellation and honest resource status. Depends on P0 contracts and P1 outcomes.                                                                                      | A02-A04, A19, A24 fixtures, help/snapshots and accessibility review.                                                         |
| P3: Platform adapters and incident repair | Repair evidenced Mac failure; complete Windows native integration; Linux prompt handling; implement selected headless flow. Platform tasks may run independently after P1 contract stabilizes.                                                                       | A05-A07, A17, A26, A30 on real target sessions.                                                                              |
| P4: Identity lifecycle and integration    | Wire refresh concurrency, resume, logout invalidation, MCP status, identity isolation and bounded access validation into actual runtime.                                                                                                                             | A08-A15, A25 plus leak checks through local tools.                                                                           |
| P5: Distribution and migration            | Produce signed Apple Silicon/Windows and verified Linux packages, maintain CLI pin, migration and docs; resolve package inheritance. Publish immutable candidates under a prerelease channel for P6. May prepare alongside P2-P4; package final tested runtime only. | A01, A18, A27, A28 with immutable artifact receipts.                                                                         |
| P6: Acceptance and remediation            | Run platform matrix, timed unfamiliar-user trials, repeatability/performance checks; independent reviewer assesses exact published candidate.                                                                                                                        | All applicable catalog cases, scorecards and defect resolution; >=9 per platform with no mandatory failures.                 |
| P7: Promotion and observation             | After P6 promote the same immutable version/bytes, perform fresh production smoke tests and record owner review.                                                                                                                                                     | Production version/digests, verification timestamps, handoff and rollback instructions.                                      |

Each implementing agent gets bounded file ownership and a handoff containing:
requirement IDs, dependencies, protected invariants, candidate commit, tests to
run, expected artifacts and open questions. A reviewer who did not author the
change checks results. Do not make concurrent incompatible edits to shared
storage contracts. Use repository `just test` and scoped lint/format requirements;
run relevant cross-platform fixtures, not unrelated broad test suites.

Re-use successful Rust compilation caches. Separate fast protocol/UX checks,
native storage checks and final packaging so a documentation or fixture edit
does not trigger a full Mac compilation. Mac runners must assert ARM64. Record
cold/warm build times; cache reuse cannot bypass testing final signed artifacts.
Never cancel a running Rust build merely because it is slow.

## 8. Scoring, hard gates and remediation

Use [the scorecard template](AUTHENTICATION-SCORECARD.template.json) to create
candidate records. It contains no measured scores. Instantiate one assessment
per required platform and enumerate every frozen subcase before running tests;
replace template objects with actual records and retain supporting receipts.

Required acceptance rows A01-A21 and A24-A30 are hard gates, with A26 conditional
only on the D1 decision recorded at P0. A22's unassisted completion requirement
is also mandatory; its timing targets and A23 are scored quality targets.
Zero credential disclosure, zero authentication/authorization bypasses, no broken
gateway/MCP/local-tool integration, and resolution of the reported Mac incident
are unconditional gates. Missing signing, missing full Windows packaging, or
missing actual desktop validation cannot be replaced by a passing unit test.

| Dimension                                         | Maximum points | Case IDs           |
| ------------------------------------------------- | -------------: | ------------------ |
| Secure native persistence and incident resolution |             20 | A05-A07, A17       |
| Guided onboarding and usability                   |             15 | A02-A04, A20, A22  |
| Gateway, policy and MCP correctness               |             15 | A08-A11, A25       |
| Refresh, resume, logout and identity isolation    |             15 | A12-A15            |
| Secret handling and profile trust                 |             10 | A16, A19           |
| Installation, signatures and upgrade safety       |             10 | A01, A18, A27, A28 |
| Diagnostic quality and cancellation               |              5 | A21, A24           |
| Performance and repeatability                     |              5 | A23, A29           |
| Session and terminal compatibility                |              5 | A26, A30           |
| **Total**                                         |        **100** |                    |

For each dimension, points = weight x passed applicable subcases / all frozen
applicable subcases. Subcases carry equal weight within that dimension. Shared
case IDs may have platform-specific executions, not double counting within a
dimension. Split A22 into mandatory unassisted completion and scored timing
subcases per authentication method per OS family; A23 is one quality subcase per
required platform session. Unexecuted, skipped, flaky/unexplained,
or unsupported-within-scope cases earn zero. N/A needs a recorded P0 rationale;
an all-N/A dimension indicates a broken matrix and cannot receive full credit.

Raw platform score = points / 10, with no rounding up across 9.0. Any failed or
unknown hard gate caps that platform's reported score at 8.9 and marks it FAIL.
Shared security gate failures fail all platforms. Overall delivery score is the
minimum platform score, not an average. Every required OS session variant must
pass its hard gates even when its OS family's aggregate score is >=9.0.

Progress dashboards report **verified applicable cases / frozen applicable
cases**, unresolved mandatory gates, pass rate on first attempts, current score,
candidate digest and evidence age. Keep design progress separate from delivered
progress. This document earns no implementation credit. Alpha.9 remains failed
against the new release contract; no defensible >=9 score exists today.

If any score is below 9.0 or a gate fails, the agent must produce a remediation
record: defect, affected case/platform, evidence, root cause or investigation,
specific corrective change, owner, dependency and regression check. Fix critical
security/integration failures first, then the highest-impact onboarding failures.
Rerun affected checks, the shared regression set where relevant, and installed
smoke tests against new bytes. Obtain a fresh independent score; repeat until
all thresholds pass. Preserve failed receipts. External signing/access blockers
remain visible while independent work continues; never self-waive a gate or
award points for an intention, a mocked substitute, or an unavailable reviewer.

## 9. Evidence, promotion and rollback

Store candidate evidence in `validation/<date>/authentication/<version>/`.
Every receipt includes plan version, requirement/subcase, platform/OS build,
architecture, shell/session type, native store/version, package version, source
commit, binary SHA-256, signing identity metadata, test timestamp, command/fixture
version, first-run result, reruns, redacted output and evidence-layer label.
Live receipts include request/scan IDs and pseudonymous audit correlation,
never tokens, auth codes, PKCE verifiers, raw JWTs or participant passwords.

Publish per-platform scorecards and an independent-review report with exceptions
and links to receipts. Freeze the receipt list used for each score. Record user
trial assistance and failures; five successful retries do not erase a failed
first experience. Owner acceptance on the reported Mac must be recorded as
pending until actually performed; do not impersonate it with another agent.

Promote the same signed/verified candidate bytes to the intended registry channel.
Validate a fresh teammate installation from that channel on each platform;
keep private package access separate from Keycloak roles. Do not overwrite an
existing package version or move the primary channel before mandatory acceptance.
Maintain concise platform-native guides and a single shared login explanation.

After promotion, run fresh install/login/resume/inference/MCP smoke checks and
two refresh cycles. Keep the known-good package available. Rollback instructions
must specify registry tag restoration and compatible state behavior; never
restore a revoked or possibly consumed refresh token from backup. Preserve
history and nonsecret settings; require guided reauthentication when needed.
Any runtime/signature/package change invalidates affected artifact acceptance.

## 10. Scope boundaries and release report

Conjur remains a separate stretch goal for centrally managed workload secrets;
it is not a prerequisite for ordinary user sign-in or a substitute for local
credential lifecycle work. No new realm/JWKS, hosted PAH dependency, unrelated
agent-engine rewrite, Intel Mac build, or blanket endpoint-security exception.

The final report must identify shipped version/channel, platform coverage,
per-platform and overall scores, resolved incident cause, migration behavior,
test/evidence links, residual limitations and owner test status. Do not declare
"everything fixed" while an applicable gate or external prerequisite is pending.
Planning may proceed with the clearly labeled assumptions above; implementation
of undecided headless/trust mechanisms waits for the corresponding design decision.
