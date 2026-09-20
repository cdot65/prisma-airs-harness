# Focus 3 independent review

Status: independent implementation review **approved at9.5/10** after both final installed fault cases passed. No Rust commands or product edits performed by reviewer. This approval is limited to development-native implementation acceptance; exact all-platform package acceptance remains PRD4.

## Concrete defect and fix

A failed refresh leaves a durable tokenless native record, as intended. Resuming the saved conversation subsequently fails its pre-TUI identity check. The old message offered only `/signin`, which cannot be used when the interactive session never opens.

The change wraps only the standalone session-launch call to `airs_helper_relocation::refresh`. `airs_harness::startup_recovery` translates typed SignInRequired and OutcomeUnknown into a copyable selected-environment `login --restore-session` instruction and tells the user to retry their original command. It discards irrelevant private context for those two known states. Other errors retain their original chain and category. The credential helper return path, status markers, refresh ordering and active-TUI `/signin` behavior are unchanged.

The environment name comes from the already bound home, not a freshly consulted mutable default. Existing `airs_environment::command` renders leading-hyphen names with `--environment=NAME`. Tests parse the generated command with the actual CLI parser for no name, ordinary name, `-staging` and `--`, and verify restore-session remains selected. Inline snapshots cover both user-visible recovery states. No unsafe auth/storage algorithm change is present.

## Installed fault acceptance design

Each case establishes real signed synthetic inference and native MCP authorization, completes a nonce-checked read-only MCP tool result and stores an actual conversation. A normal refresh first proves the successful exchange path. Only then does a one-shot fault consume the exact current predecessor:

- response-lost: issuer creates generation3 and the HTTP handler closes without returning headers/token body;
- rejected: issuer returns invalid_grant, remaining at generation2.

Three fresh helper processes must each return the correct status marker, no credential stdout and active-session /signin guidance, while the total refresh POST count remains exactly2: one positive control and one fault. The consumed-token set difference must equal the exact faulted predecessor. A fresh resume process must exit before loading the conversation, show the shell restore command, and perform no network request. Explicit doctor verification must fail locally; only the existing unauthenticated health GET is allowed.

Preservation checks compare registry, config, binding, auth epoch and sibling environment files, retain the real conversation prefix, preserve the exact MCP native record and issuer generation, and forbid plaintext fallback or spontaneous replay of the completed tool. Checks include synthetic access/refresh token canaries without retaining their values in evidence.

Success is returned only after explicit inference/MCP cleanup, context teardown and native/tooling hash verification. Native queries identify only the binding created by the fixture or the randomly namespaced MCP accounts; no owner enumeration or real credentials are used. Failure output retains safe type/phase only and does not serialize terminal exceptions or callbacks. Fixture temporary state uses the supported private home-cache layout rather than rejected helper paths under /tmp.

## Release integration

`airs_release_acceptance.py` now passes both its verified npm launcher and verified native executable to installed stages. The new fault test selects the native path while other installed tests keep using the npm launcher. Existing `test_airs_harness*.py` discovery includes both cases in candidate and anonymous registry acceptance. Mocked subprocess tests assert both exact paths arrive; this prevents accidental validation of an unrelated development executable.

## Earlier review findings closed

- Plain doctor is not described as network-free. The final test uses explicit verification and permits only its health GET while forbidding authentication/inference requests.
- Native MCP credential presence is checked as well as issuer generation.
- The receipt accurately labels blocked resume startup with recovery, not successful in-session reauthentication.
- Uncertain MCP tools/call response loss, attended recovery and production idle/revocation remain explicitly outside this evidence scope.
- Redundant static untouched-marker tests were removed; meaningful generated-command parsing and unrelated-error preservation remain.

## Evidence inspected so far

- `lifecycle-unit-tests.log`:28 passed, including successful exchange and one-shot fault behavior.
- `release-handoff-tests.log`:17 passed, including verified native path propagation.
- `RUST-CHECKS.json` binds complete CLI959passed/0skipped and final focused14passed, reviewed inline snapshots, successful lint/format/build, and restoration of19unrelated formatting files. All seven referenced file SHA256 values were independently recomputed and matched. Source test logs confirm both counts.
- `git diff --check` passed for the inspected code before final formatter changes.

## Final installed evidence

`installed-faults-final.log` records both actual unittest-wrapper cases passed in131.799seconds. Both bind native SHA256 `da54e47800ddfe78a0d233508fe0f6b92776f93a804472470056948b20a1c79e` and tooling digest `a4fdad89630811d124c64d31f4bdcd122acfa9716bbcfe487108825c9b14ee68`.

Each case reports one normal refresh and exactly one faulted refresh request, the exact predecessor consumed, three independent helper checks and one earlier actual read-only MCP call. Response loss reaches issuer generation3 and retains refresh_outcome_unknown; rejection stays at generation2 and retains sign_in_required. Both prove resume startup is blocked with the usable shell recovery command; configuration, binding, sibling environment, conversation and independent native MCP grant remain preserved; doctor sends no authentication/inference request, no plaintext fallback appears, and native cleanup completes. Each receipt explicitly sets production_acceptance=false.

Canonical vault login and implementation-status lessons now label the shell recovery instructions as pending `0.1.2-alpha.1.mcp.1`, explain same-person `login --restore-session` followed by retry, and retain active-session `/signin`. Root reports22browser checks passed for that draft export. This review does not claim the draft is deployed or the package is already published.

## Final rubric

| Dimension | Score | Basis |
| --- | --- | --- |
| Completeness |3.0/3| Both prioritized fault modes, reproduced startup defect, narrow fix, safe leading-hyphen command handling, regression integration, preservation/cleanup and pending-version documentation completed. |
| Capability |2.6/3| Actual development-native positive controls, repeated helpers and blocked resume pass. Cross-platform exact-package runs and attended same-person restoration remain separately stated future evidence, not assumed from this run. |
| Best practices |1.9/2| Existing auth algorithm preserved; typed boundary mapping, allowlisted receipts, exact private native accounts, source/tooling stability, meaningful before/after oracles and independent corrections. |
| Optimization |2.0/2| Small CLI presentation fix; existing lifecycle/store fixtures reused, bounded real-expiry observations, no new authentication service or runtime dependency. |
| Total |**9.5/10**| Approved for sequential progression within this implementation scope; owner review remains pending. |

No unresolved product-code blocker remains. Preserve the final source/evidence in reviewable commits. Native next-version Linuxx64/ARM64 and signed Apple Silicon candidate/registry checks remain PRD4 gates. This is neither production lifecycle/reauthentication acceptance nor uncertain MCP tools/call-response acceptance. Keep stable `latest` at0.1.1; no stable promotion is authorized by this review.
