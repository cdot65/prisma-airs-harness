# Focus 3 — interrupted inference refresh and startup recovery

Scope: inference refresh failure while an independent native MCP grant and real completed read remain preserved. This does not claim coverage of an uncertain MCP `tools/call` response, production revocation, attended reauthentication, or an active-session sign-in dialog.

## Reproduced behavior

Two development-native observations passed initial signed inference/MCP login, a real read-only tool turn, normal refresh, consumed-response fault and repeated helper checks. Their original oracle incorrectly expected resume to open the conversation; both timed out waiting for the TUI header. Source tracing identified the deliberate pre-TUI identity check in `airs_session_binding::validate_locked`.

At that boundary, the existing typed error offered `/signin`, which requires an open TUI. The fix changes only startup presentation: a named `airs ... login --restore-session` command followed by retrying the original command. It preserves in-session helper markers and `/signin`, and preserves the same identity/epoch requirement of restore-session.

## Required installed rows

For both `response-lost` and `rejected`:

1. A signed native-store inference identity and independent native MCP OAuth identity complete one actual nonce-checked tool result and persist a real conversation.
2. A normal refresh succeeds first. The next exact predecessor is consumed before either the response is dropped or an explicit `invalid_grant` is returned.
3. Three independent helper invocations retain the proper recovery classification with no repeated refresh POST and no token on stdout.
4. A new resume process fails before loading the saved conversation and offers the selected-environment shell command. No inference/tool request occurs on failed startup.
5. Explicit doctor access verification fails locally; only its existing unauthenticated health GET is allowed. Configuration, binding, auth epoch, other environment and real history remain preserved; no automatic tool replay or plaintext fallback.
6. The independent MCP native record remains present and its grant untouched. Native cleanup succeeds before a success receipt is returned.
7. Executable and all validator hashes remain unchanged throughout observation.

`test_airs_harness_refresh_faults.py` is part of the existing `test_airs_harness*.py` discovery gate on every supported candidate and anonymous registry install. The release driver now provides the already-verified direct native path separately as `AIRS_HARNESS_NATIVE_BIN`; other installed tests continue using the npm launcher.

## Evidence status

- Lifecycle fixture unit suite: **28 passed**, `lifecycle-unit-tests.log`.
- Release handoff/receipt suite: **17 passed**, `release-handoff-tests.log`.
- Initial before-fix observations: `lost-first-error.json`, `lost-second-error.json` (safe phase/type only; no raw terminal transcript).
- Root reports complete CLI suite **959 passed**, final focused snapshot suite **14 passed**, lint/format clean and native build passed; root owns those receipts.
- Final installed fault discovery wrapper: **2 passed in 131.799 seconds**, `installed-faults-final.log`; individual receipts `installed-response-lost.json` and `installed-rejected.json`. Native SHA256 `da54e47800ddfe78a0d233508fe0f6b92776f93a804472470056948b20a1c79e`; frozen validator SHA256 `a4fdad89630811d124c64d31f4bdcd122acfa9716bbcfe487108825c9b14ee68`. Both cases satisfy the seven installed rows above on the development Linux native build.
- Exact next-version Linux x64, native ARM64 and signed Apple Silicon candidate/registry observations: release gate; development hashes alone cannot satisfy those rows.

Independent final implementation review: **9.5/10**, `FINAL-REVIEW.md`, with no unresolved implementation blocker. Exact all-platform publication acceptance, attended restore and uncertain MCP tool-response behavior remain outside this development acceptance.
