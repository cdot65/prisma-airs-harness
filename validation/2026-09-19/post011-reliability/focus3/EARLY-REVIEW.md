# Focus 3 early independent review

Status: read-only guard/fixture review while author iterates. No Rust commands, product edits or score. Scope is synthetic inference refresh failure across installed helper invocations with native persistence; production lifecycle and uncertain MCP execution remain separate.

## Production guard inspected

`codex-rs/cli/src/airs_oidc.rs` already implements the intended safety ordering:

1. Load a matching native Active record.
2. Return access token while safely outside the refresh window.
3. Complete bounded metadata discovery before marking the grant pending, so metadata outage alone does not consume local state.
4. Persist tokenless RefreshPending before submitting the refresh request.
5. A definitive invalid_grant becomes durable tokenless SignInRequired; uncertain network/token response leaves RefreshPending.
6. Returned generation persistence can retry without retrying the consumed exchange.

`airs_oidc_refresh_tests.rs` already covers these stored-state classifications. New installed faults are useful acceptance evidence, not justification to weaken the guard or manufacture a product defect.

## Meaningful new boundaries present

- Typed fault enum and one-shot arm; rejected grant consumes its predecessor without issuing a new generation.
- Lost-response fault consumes predecessor and issues generation2 before the HTTP handler closes without token headers/body.
- Three independent helper subprocesses check precise status markers, no stdout access token, explicit /signin recovery, and exactly one total refresh request.
- Real signed initial login, native record and completed read-only MCP tool turn precede fault injection.
- Environment registry, config, binding, auth-generation and sibling environment files are compared before/after; actual conversation history must retain its prefix.
- Native/tooling identity and secret-free fixed-schema receipt are retained; raw helper/PTY exceptions are suppressed outside the private worker.
- Cleanup occurs before success return and the context unwinds native credentials/services; existing exact-account helpers avoid enumeration.

## Findings sent to author

### F3-R1: passive doctor is not network-free

Initial `airs_lifecycle_faults.py` compares total identity.requests before/after plain doctor, and proposes `passive_doctor_no_network: true`. Plain doctor intentionally sends an unauthenticated /v1/health GET; IdentityFixture records that GET. This assertion is wrong and would fail correct behavior. Permit/verify the health request, forbid token/discovery/inference requests, and use a precise receipt name such as `passive_doctor_no_authentication_or_inference`.

Also require doctor exits nonzero and its credential_configuration check fails. Parsing any structured checks alone does not prove the pending/rejected session is reported as unhealthy.

### F3-R2: preserve the local MCP credential, not only issuer generation

Unchanged issuer MCP generation proves no new MCP issuance, but would still pass if recovery accidentally deleted the native MCP record. Require the existing `record_exists(session.identity, session.env)` after the inference fault. This queries only the run's exact isolated identity and validates Linux record structure; on Mac it queries exact metadata without extracting token values.

### F3-R3: keep scope labels precise

The unchanged count of one earlier real MCP read proves no spontaneous replay during inference-refresh recovery. It does not inject a lost tools/call response and therefore does not establish safety for an uncertain upstream MCP operation. Keep that optional scenario explicitly deferred. Similarly three restarted helper subprocesses establish durable recovery classification; they are not fresh full-TUI sign-in/restore or attended production acceptance.

## Remaining evidence needed

- Author's fixes to R1/R2 and installed execution of both loss/rejection cases.
- Native record cleanup completed and failure receipts emitted only afterward.
- Existing successful lifecycle/token fixture checks remain passing after the one-shot fault additions.
- Source/tooling provenance included; unrelated formatting edits excluded.

No production runtime defect has been found from source review. No numeric score is assigned before execution evidence.

## Follow-up correction and runtime defect scope

The author correctly noted that plain doctor's credential_configuration row intentionally inspects saved metadata only; it must not be required to inspect native Active/pending status. My suggested failed metadata-row assertion above is withdrawn. Instead, explicit `doctor --verify-access --json` can prove failed gateway_access with no token/Responses request while permitting the normal health GET. Existing helper status markers establish the exact durable recovery classification.

Installed reproduction then found a real user-facing boundary defect: restarting the prior conversation fails before TUI in helper relocation/session-binding validation because identity() loads the tokenless pending/rejected record, yet the propagated recovery message only recommends `/signin`, which is unusable outside an active TUI. Root authorized a minimal CLI-boundary hint correction.

Review constraints: map only typed CredentialRecovery at the standalone session-launch/helper-relocation boundary, leave the credential helper's marker and active-session semantics intact, derive environment name from the bound home rather than mutable default, and use existing airs_environment::command() for leading-hyphen-safe `--environment=NAME` formatting. Prefer `login --restore-session` to preserve same-identity conversation recovery; ordinary login is a distinct auth-session boundary. Other configuration/native-store errors must retain their actual category rather than being reclassified.

The installed fault root must use the existing private home-cache layout so helper trusted-path validation is exercised under supported conditions rather than blocked by /tmp. Author now also includes a successful normal refresh before injection, exact consumed-token set difference, and restarted conversation checks. Uncertain MCP tool-response fault remains explicitly deferred.
