---
id: evidence
title: "Implementation status and public sources"
sidebar_label: "Implementation status and public sources"
---

## September 20: post-0.1.1 reliability test release

Optional test version **0.1.2-alpha.1.mcp.1** is published under `mcp`. The launcher and all three native packages passed exact candidate and fresh anonymous registry acceptance on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. `latest` continues to select stable **0.1.1**, and other tags are unchanged. Source implementation, installed package acceptance and production-account acceptance remain separate claims.

The launcher now gives exact-version recovery when its native dependency is absent, preserving the original registry and installation scope. A disposable local installation reproduced the published 0.1.1 failure and recovered after reinstalling the same version with optional dependencies included. Global omission did not reproduce that failure on the examined npm installations; no universal global-omission claim is made. The launcher regression suite passed 31 tests, and the exact packaged 0.1.1 Ubuntu helper passed its six isolated keyring/readiness cases without using owner credentials.

The new diagnostic-report UI uses an explicit field allowlist and reuses completed `/doctor` results. Preview, clipboard and private local-save actions do not initiate another request or upload a report. Stale or wrong-thread actions are rejected. The implementation passed 13 focused tests, the TUI suite recorded **4,360 passed and six skipped**, and all five installed doctor scenarios passed. An independent implementation review scored this bounded work **9.4/10**. These results cover privacy, preserved state, request counts, clipboard failure handling, local saves and existing doctor behavior; the score is not a publication decision.

Clipboard failure was tested with an injected backend, while the installed SSH fixture captured the terminal-mediated clipboard request and matched its decoded bytes to the saved file. Neither establishes successful pasting in every desktop or terminal. All three native package and fresh registry checks passed. The isolated fixture checks do not establish a fresh human SSO login, a live ServiceNow request or production long-duration renewal.

The same test version corrects pre-session company SSO recovery guidance. When startup or `resume` encounters a rejected grant or an uncertain consumed refresh, the shell error recommends `airs --environment NAME login --restore-session`, followed by retrying the original command. Same-person restoration preserves the conversation binding; an active terminal session retains `/signin`. This narrow fix passed independent source review and installed recovery fixtures on all three native targets. No attended restoration is claimed.

Release acceptance also exercised exact stable 0.1.1 → test release → stable 0.1.1 npm roundtrips on every platform, both before and after publication. Configuration, native credential bindings and conversation history remained intact while actual synthetic inference/MCP turns completed before upgrade, after upgrade and after rollback. Separate continuous-process fixtures completed three turns and two inference/MCP expiry cycles on each platform. They do not substitute for long-duration production renewal.

The current full GNU workspace completed with **18,331 passed, three failed and 34 skipped**. Its status remains **failed**. Independent current-source review found that the three failures exercise experimental remote shell snapshot V2, which is disabled by default in AIRS. The explicit AIRS product gate passed while retaining the failed workspace result. No historical release waiver was reused, and this test release does not promote a new stable version.

Stable 0.1.1 and its original full-workspace results remain recorded below. This work does not reclassify the earlier non-green workspace run as green or carry its version-specific exceptions into a new release.

## September 19: 0.1.1 stable release

Version **0.1.1** consolidates the onboarding and MCP work into a stable release. The `latest` tag selects 0.1.1. Exact native candidates, fresh anonymous registry installations, upgrades from mcp.6 and onboarding.4, and unversioned default installations passed on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. Existing prerelease tags remain separate. No new authentication service is added, and native MCP device authorization remains dependent on gateway support.

The owner confirmed all real-account Apple Silicon checks on mcp.6: inference sign-in, ServiceNow sign-in through `/mcp`, a read-only query and reuse after restarting the harness. Stable runtime behavior is unchanged except for the version stamp; release tooling, test isolation and documentation are verified separately. This owner report does not claim long-duration production renewal or other users' entitlements.

The complete GNU/Linux workspace run on the native release source recorded **18,313 passed, 6 failed, 34 skipped**. It was not a green run. Three inherited remote-executor cases concern an upstream service disabled in AIRS; an installed-command check confirmed it is unavailable. Two ambient test fixtures and one voice-runtime lookup were corrected in test/CI follow-up and passed focused checks without changing application runtime source. All six original failures remain in the release evidence. Unknown failures were not waived.

Each platform also passed two synthetic inference/MCP renewal cycles and three read-only fixture tool turns. This does not establish production long-duration renewal or revoke prior evidence limitations. Native MCP persistence, second-process reuse and logout passed in isolated stores.

The dated records below retain the evidence boundaries and tag values that applied at each release.

## September 19: mcp.6 browser sign-in and callback guidance

**0.1.0-alpha.22.mcp.6** was published for local testing under the `mcp` tag. Exact candidates and fresh anonymous registry installations passed on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. Non-mcp tags remain unchanged. This is isolated package acceptance, not production SSO or ServiceNow acceptance.

The release adds desktop browser opening, a failed-launch fallback, SSH/manual callback guidance, scrollable long links and typed authorization/storage/discovery progress. Native MCP persistence, reuse and logout passed; upgrades from mcp.5 and onboarding.4 preserved checked configuration and legacy command targets. Candidate lifecycle checks completed two renewal cycles for inference and MCP and three synthetic read-only tool turns on each platform. The real five-minute callback-expiry check passed during development; that opt-in test was not repeated against every packaged binary. The desktop browser-launch stub fixture runs on Linux; the owner subsequently confirmed the real Apple Silicon browser/SSO, ServiceNow read and restart/reuse checks described above.

Affected Rust checks passed 5,633 tests with 14 skips; an unchanged pet-rendering assertion failed because its encoded temporary filename contained the searched substring. That single test passed with a different temporary path, and inherited-color terminal failures also passed after environment correction. New MCP UI snapshots were reviewed. Scoped lint completed with one preexisting warning and no source edits. This does not claim a green full-workspace run or replace the earlier long-duration lifecycle evidence.

### Previous mcp.5 release and Ubuntu preparation helper correction

**0.1.0-alpha.22.mcp.5** is published and ready for local testing under the `mcp` tag. Fresh anonymous registry installations passed isolated acceptance on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon. At that release, the protected `latest`, `alpha` and `onboarding` tags remained **0.1.0-alpha.22.onboarding.4**. The release packages the corrected Ubuntu SSH readiness helper. The helper bundled in mcp.4 predates that correction; use the verified mcp.5 helper or a corrected administrator-provided copy for SSH keyring setup.

Exact mcp.5 candidate and published packages passed the installation, onboarding, terminal, installed-regression, MCP manager, diagnostics, bundled CLI and command-output checks. Upgrades from mcp.4 and onboarding.4 preserved the checked configuration and legacy command targets on all three platforms. Required native MCP checks passed on every platform, including persistence, reuse in a second process and logout. Separate candidate quick lifecycle runs each completed two renewal cycles for inference and MCP and three verified read-only tool turns. These shorter exact-build checks do not replace the older development-binary timing evidence below.

The correction prevents an unlock attempt from leaving Secret Service clients connected to a different locked daemon. It verifies the collection's actual lock state, preserves encrypted credentials after a wrong password, reuses an already unlocked service, and prints the actual script path for the next unlock. A real Ubuntu regression exercised six checks with a disposable encrypted keyring, private D-Bus session and isolated installation. It used the mcp.5 source helper with an explicit mcp.4 package-version override while mcp.5 was unpublished. That prepublication result is source-helper behavior evidence. The same checks then passed against the helper extracted from the combined npm candidate with the explicit mcp.4 override.

The helper extracted from a fresh anonymous mcp.5 installation passed all six checks with its default mcp.5 version and no override. Those checks cover encrypted-keyring creation and native write/read/delete, unlocked-service reuse, unlocked checks without prompts or restart, the exact shell-quoted rerun path when its filename contains spaces, wrong-password preservation, and successful recovery with the correct password. The fixture used a private user home, installation prefix, D-Bus session and credential directories; it did not unlock or read the user's real credential store.

A user separately reported successful workspace-key use and ServiceNow access. This report is separate from automated package checks and is not an independently captured tool receipt or proof of company SSO. A later readiness probe from the Ubuntu host confirmed that the company issuer accepted the public harness client's device request and returned `authorization_pending` to its first token poll. That probe obtained no user tokens and did not test human approval, native credential persistence or gateway inference. The endpoint itself needs no browser for this flow; the user completes company sign-in on another device. The login lesson now includes an explicit SSH walkthrough. No new production token-expiry or revocation result is claimed.

An attended follow-up then passed on the exact published mcp.5 Linux x64 executable. The user approved device authorization in a browser on another machine; AIRS saved the verified identity in Ubuntu's native credential store. Two subsequent, separate CLI processes each retrieved that credential and obtained a real gateway inference response without another sign-in. The isolated test profile was signed out afterward, its binding cleared, and issuer refresh-token revocation confirmed. Existing user profiles were not used or replaced. This verifies inference device sign-in and immediate credential reuse for the tested account; it does not establish MCP authorization, long-lived renewal or other users' entitlements.

The mcp.4 native-storage change and its exact package and synthetic lifecycle evidence remain recorded below. The helper correction does not migrate existing environment credentials or change the production issuer's policies.

## September 19: mcp.4 native storage default and measured synthetic lifecycle

**0.1.0-alpha.22.mcp.4** was published under the `mcp` tag. Fresh anonymous registry installations passed isolated acceptance on Linux x64, Linux ARM64 and Apple Silicon. At that release, the protected `latest`, `alpha` and `onboarding` tags remained **0.1.0-alpha.22.onboarding.4**. New environments generate `mcp_oauth_credentials_store = "keyring"`, so MCP sign-in requires the native store and cannot silently fall back to plaintext. Upgrades preserve existing storage modes, credentials and histories. Changing the mode alone does not migrate tokens; the login lesson describes cleanup under the original mode before optional migration.

Exact candidate packages passed installation, onboarding, terminal, installed-regression, MCP manager, diagnostics, bundled CLI, command-output and upgrade checks on Linux x64, Linux ARM64 and Apple Silicon. Apple Silicon signature and notarization checks passed. In-place upgrades from both mcp.3 and onboarding.4 preserved existing state on all three platforms. Native MCP checks completed gateway OAuth against a synthetic service, persisted credentials without a file fallback, reused them in a second process without another authorization exchange, then signed out and verified removal while inference remained usable.

Separate quick lifecycle checks on these exact candidate executables ran for approximately 178 seconds on each platform. They completed two expiry-and-renewal cycles for each of the inference and MCP credentials, three actual read-only tool calls with verified unique results, and native credential cleanup. Additional isolated checks on a fresh Ubuntu SSH host passed; they do not resolve the owner's earlier credential-session incident.

The longer elapsed-time checks used a recorded **development executable**, not the published mcp.4 package bytes:

| Synthetic exercise | Measured result | Boundary of the evidence |
| --- | --- | --- |
| 60-minute active use | 3,601.079 seconds; seven post-expiry renewal cycles each for signed OIDC inference and MCP; 16 completed read-only tool turns with unique results verified against the fixture | One continuing real process with synthetic identity and gateway services |
| 35-minute idle return | 2,100.085 seconds with no HTTP requests during the quiet window; both credentials renewed and an actual tool call completed on return | The fixture kept refresh grants valid throughout the idle period |

Both exercises preserved the process, conversation, identity binding, authentication epoch and append-only history. The fixtures enforced expiry and one-use rotating refresh tokens; the checks verified actual tool results, rather than accepting a model's claim that a tool ran. Credential and callback canaries were absent from retained transcript and history. Current cancellation and stale-completion terminal tests passed, alongside the scoped CLI, onboarding and MCP regression checks. This does not constitute a full Rust workspace pass.

These results demonstrate synthetic renewal across measured time boundaries. They do not prove a production issuer's idle or revocation policy, real workspace-key routing, company SSO, ServiceNow access, or new installed recovery behavior for rejected or ambiguous grants and concurrent refresh races. Production lifecycle acceptance remains open. The dated mcp.3 and earlier records below retain their original scope.

## September 19: earlier mcp.3 in-session management and reliability release

**0.1.0-alpha.22.mcp.3** was published under the `mcp` test tag, with the existing `/mcp` connection manager and `/doctor` diagnostics. Fresh anonymous registry installations passed isolated acceptance on Linux x64, Linux ARM64 and Apple Silicon, covering installation, onboarding, terminal behavior, installed regressions, MCP management, diagnostics, the bundled CLI, upgrades and command output. Apple Silicon signature checks also passed. These checks use synthetic gateway and credential fixtures; they do not establish live-account acceptance. At that release, `latest`, `alpha` and `onboarding` remained **0.1.0-alpha.22.onboarding.4**. Consult the login lesson for the exact tested installation; the default onboarding.4 installation at that time did not provide these dashboards.

The September 19 source changes preserve the selected environment in onboarding recovery commands, including valid leading-hyphen names; reject invalid loaded registry names without rewriting state; reject unsupported Node versions before native or bundled CLI launch; and retain precise credential-service failure and pending-cleanup diagnostics. The final scoped native CLI suite passed **955 tests**. Isolated Linux native recovery fixtures exercised both `staging` and `-staging` while another default remained selected. These source/fixture results are distinct from the fresh installed-package checks above.

In mcp.3, native-only MCP storage required a one-time `keyring` setting; automatic generation of that policy was not implemented until mcp.4. Successful MCP changes require **Start new conversation** within the same process. Workspace API keys authorize inference independently of the local environment name and do not replace MCP SSO.

Fresh human SSO, a real workspace-key inference request, the owner's Ubuntu credential-session investigation and a live ServiceNow incident call are deferred to attended testing. Full Rust workspace results contain classified historical failures; no full workspace pass is claimed. The dated records below remain historical evidence for their own versions.

## September 17: command migration and bundled product CLI

Standalone Prisma AIRS CLI **7.0.1** exports `airs-cli`. Harness **0.1.0-alpha.22** exports `airs` and bundles its tested CLI **7.0.0** as `airs cli ...`, with SDK **0.33.0**. The standalone patch finalizes stable publication without changing command behavior or tenant configuration. Linux x64, native Linux ARM64 and signed/notarized Apple Silicon installations passed acceptance. A separate global product CLI is optional. The temporary `airs-harness` alias is scheduled for removal in alpha.23.

Validation includes 1,920 TypeScript CLI tests, 925 native CLI tests and 51 skills tests. Each installed harness passed 47 executable checks (46 passed, one platform-specific skip), including an agent invoking the embedded skill and private CLI. The installed product CLI generated PDF, PNG, JPEG, SVG and DOCX fixtures on all three platforms. Mac Keychain checks passed. Upgrade checks preserve environment identity, configuration and history; the paired CLI/harness migration, independent uninstalls and ordered rollback passed on each platform without force. These checks use isolated state and deterministic gateway fixtures.

The rename does not merge harness environments with CLI tenants. Company SSO authorizes gateway inference and MCP access; product API credentials remain in separately selected tenant JSON. The login lesson covers the complete first session, including the optional product CLI setup after the ServiceNow read.

Publication and installed-client acceptance do not establish a fresh human SSO login or a live ServiceNow incident call. Complete production frontend expiry and hourly renewal acceptance remain open. The user performs the walkthrough's final read-only tool call with their own provisioned account.

## September 17: ServiceNow onboarding and environment commands

Alpha.21 is published for Linux x64, Linux ARM64 and Apple Silicon. It provides the login lesson's unified `env create` and `env status` commands and removes top-level `setup` and `status`. The implementation passed 780 scoped CLI tests and 11 focused onboarding/status checks. Fresh anonymous registry installations passed 47 executable checks per platform (46 passed and one platform-specific skip), plus environment lifecycle checks. Upgrades from alpha.20 preserved configuration on all three platforms; alpha.14 upgrades also passed on Linux x64 and Apple Silicon. Apple Silicon signing, notarization and Keychain checks passed; Linux ARM64 was exercised natively in an ARM64 Linux VM. These are installed-client checks against deterministic fixtures, not fresh production SSO or ServiceNow acceptance.

ServiceNow is a separate development integration from the course's eight-tool utility server. The deployment record reports upstream acceptance of four incident tools and refresh cycles with disposable users. The maintainer subsequently observed the gateway-connected `service-now` inventory with OAuth in the harness. That inventory observation does not establish a completed model-selected incident call through the gateway frontend. No ServiceNow production deployment, fresh human SSO acceptance, or new live tool-call acceptance is claimed by this documentation update.

The human uses company SSO for inference and gateway MCP authorization. The gateway owns upstream OAuth; the ServiceNow MCP service uses a server-side integration credential to access the incident table. These are separate credential boundaries. The walkthrough's final read-only tool call is the user's end-to-end verification step.

## Earlier implementation and publication scope

Every other lesson makes claims about how this system behaves. This lesson records what those claims rest on, as of the dates given, and where each piece of evidence stops. Reading it lets you tell the difference between something the reviewed source does, something a dated check observed, and something that has not been demonstrated yet.

The current architecture sends both inference and native MCP through Prisma AIRS AI Gateway. The upstream **mcp server 1** exposes eight utility tools whose execution is local to that server. Its tool implementation no longer depends on a management API.

| Area | Recorded evidence | Limit |
| --- | --- | --- |
| Server inventory | Both production replicas were inspected and expose calculate, format_json, transform_text, encode_base64, decode_base64, hash_text, generate_uuid and current_time | Inventory is not proof that every authenticated call succeeded |
| Utility execution | Reviewed implementation computes in process using local text/JSON/crypto operations, clock and random source | Authentication still uses trusted issuer public keys |
| Native client | Built-in MCP client targets the gateway; inference uses the gateway's separate listener | No direct upstream bypass is part of the accepted architecture |
| Alpha.15 maintainer testing | Published Linux x64 and signed Apple Silicon packages; fresh registry installs passed 44 executable checks per platform with one skip each | Full active-expiry acceptance was incomplete |
| Manual idle recovery | The maintainer returned after about 30 minutes, used /signin and received a new inference reply in the same conversation | This did not test a subsequent MCP call or active renewal |
| Alpha.16 source correction | Normal AIRS provider dispatch now preserves typed recovery; a real helper regression failed before the fix; 78 provider tests and scoped Clippy pass afterward | Installed production behavior must be distinguished from source tests |
| Alpha.16 distribution | Publication receipts are completed alongside the binary release | Timed production renewal is not implied by package publication |

The Limit column is the part to read carefully. Each row establishes one thing, and stacking the rows does not produce a stronger claim than any one of them makes: an inventory shows which tools exist, a package check shows what an installer receives, and a manual recovery shows that one path worked once. None of these historical observations stands in for production lifecycle acceptance, which is still open. The newer synthetic timed results above have their own explicit scope.

The failed alpha.15 lifecycle receipts remain in the private implementation repository. Earlier tests against configuration-read tools are historical observations from a previous tool inventory. They are not acceptance of this utility inventory, and the retired backend investigation is not a prerequisite for testing the harness.

## What alpha.16 changes

The 30-minute SSO idle policy stays in place; alpha.16 does not change how long a session may sit idle. What it corrects is a provider-routing bug that reduced a typed credential-recovery result to a generic fatal helper error. With that route fixed, the terminal can receive the classification it needs to display company sign-in guidance instead of an unexplained failure.

Token storage and renewal transactions are carried forward from alpha.15, so the lifecycle mechanism is unchanged and only the recovery presentation moved. Successful manual inference restoration has been observed. Complete production active renewal, idle-return behavior on both platforms and MCP recovery against the utility inventory still require their own evidence, because a source-level fix and a manual observation are each narrower than a timed run on installed packages.

## How to read the claims

The course distinguishes five kinds of evidence. **Implemented** means reviewed source contains the behavior; it says nothing about what is installed anywhere. **Observed** means a dated check saw the behavior in that configuration. **Package verification** binds checks to the downloaded executable. **Synthetic lifecycle observation** measures real elapsed time and protocol behavior against controlled identity and gateway fixtures, with the executable and test sources recorded. **Production lifecycle acceptance** exercises the real identity and gateway path across its expiration, recovery and revocation boundaries. Synthetic timing can establish renewal under the fixture's policy; it cannot establish the production issuer's policy. Read each claim with its recorded build, environment and limits.

No full Rust workspace pass, external security certification, Windows distribution or blanket immediate revocation is claimed. Public examples use fictional identities and endpoints. Credentials, private logs and operational identifiers stay outside this site.

## Primary references

These references explain protocols and product capabilities. The deployment observations above come from the reviewed implementation and operator checks; they describe one deployment on particular dates and are not guarantees made by the vendor.

| Reference | Topic |
| --- | --- |
| [Gateway authentication layers](https://portkey.ai/docs/product/mcp-gateway/authentication) | Client-to-gateway and gateway-to-server authentication |
| [Gateway CAS login](https://portkey.ai/docs/product/mcp-gateway/authentication/cas) | Organizational sign-in for the SCM gateway |
| [CIE Directory Sync](https://portkey.ai/docs/product/enterprise-offering/org-management/directory-sync/cie-directory-sync) | Provisioning and consuming workspace context |
| [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools) | Tool discovery, schemas and results |
| [Native OAuth (RFC 8252)](https://www.rfc-editor.org/info/rfc8252/) | System browser and native client requirements |
| [PKCE (RFC 7636)](https://www.rfc-editor.org/info/rfc7636/) | Authorization-code proof binding |
| [Refresh (RFC 6749 section 6)](https://www.rfc-editor.org/rfc/rfc6749#section-6) | Renewal within the granted scope |
| [Resource indicators (RFC 8707)](https://www.rfc-editor.org/info/rfc8707/) | Resource-bound OAuth grants |
| [Protected resource metadata (RFC 9728)](https://www.rfc-editor.org/info/rfc9728/) | Discovery of resource authorization requirements |
| [Keycloak OIDC endpoints](https://www.keycloak.org/securing-apps/oidc-layers) | Issuer endpoints and token operations |
| [SCIM (RFC 7644)](https://www.rfc-editor.org/info/rfc7644/) | User and group provisioning |

The gateway authentication overview and MCP tool specification were rechecked on September 15, 2026. The other references retain their previously reviewed protocol or product context. Before copying any diagram from this course into another environment, verify that environment's deployed settings. The diagrams describe this case study's configuration, not protocol defaults.
