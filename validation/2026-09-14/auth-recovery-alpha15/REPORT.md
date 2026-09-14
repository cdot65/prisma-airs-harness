# Alpha.15 authentication recovery candidate

Both installed packages passed native executable, secure credential-store and
alpha.14 npm upgrade checks. The Apple Silicon binary is Developer ID signed
and notarized. These receipts do not establish production OAuth lifecycle
acceptance or publication. Alpha.14 remains the published release.

The frozen runtime source is `2f5a1ad306d379c23e21b60173b87876a79045c4`. Later
changes through `e4d169e8faf58a756a877066ac4ed82da7e8b14e` affect acceptance
tooling only. Both binaries still use the required AI Gateway destinations for
inference and MCP. The 30-minute SSO idle policy is unchanged.

## Validation

- Each installed platform ran 44 executable tests with one platform skip.
- Linux native credential/history migration preserved bindings and resumed old
  conversation history. Managed helper timeout migration from 60 to 30 seconds
  is intentional; its earlier rejected test expectation is preserved separately.
- Apple Silicon native credential checks passed in the desktop login session.
  The earlier SSH-only Keychain failure remains preserved.
- Auth/TUI scoped validation passed 5,833 tests with 14 skips, plus four focused
  core MCP authentication-boundary tests. Scoped Clippy and formatting passed.
- Broad core validation reported 4,055 passes, 104 failures and nine skips. A
  gateway fixture expectation was corrected and the focused cases then passed.
  A subsequent run against the published alpha.14 source reproduced all 103
  remaining failures. The candidate-only gateway fixture now passes in the
  focused rerun. See `CORE-BASELINE-COMPARISON.json`; full-suite success is not claimed.
- Full workspace compilation failed because Rusty V8 150.4.0 has no available
  Linux musl archive. Full workspace success and independent review are not claimed.

`INSTALLED-PACKAGES.json` binds the evidence hashes to the exact binaries. The
original logs and receipts are retained at their listed private artifact paths.
Apple Silicon build/acceptance is Forgejo run 125; signing is run 127.

## Production expiry failures

Both exact installed candidates completed browser inference login, gateway MCP
consent, native credential persistence and all eight production tools. Neither
completed an accepted frontend expiry cycle. See `PRODUCTION-EXPIRY-FAILURES.json`.

Linux's first concurrent check reached the upstream MCP service, where one
management workspace read returned HTTP 403 and another succeeded. The backend
cause is unresolved. A transport HTTP 200 does not turn an MCP error into success.

Linux failure cleanup then revoked the inference client session shared with Mac.
Keycloak logged `Session doesn't have required client` for the Mac refresh.
Mac MCP renewal and Keychain saves succeeded before inference failed. Unique MCP
keyring names isolate local credentials, but not the browser's shared Keycloak
client session. Parallel acceptance must defer issuer revocation until every peer
has finished its workflow. No 30-minute idle-policy change is needed for this
coordination defect.

The original Mac startup-challenge failure is also preserved. Expiry acceptance,
registry promotion and owner upgrades remain pending; alpha.15 is unpublished.
The alpha.14 release exception does not apply.

## Backend diagnostics and coordinated restart

The supplied Forgejo SSH key restored write access. The bounded backend denial
diagnostics are deployed to development and both production replicas. All 55
backend tests passed and the image scan reports zero High/Critical findings.
See `BACKEND-DIAGNOSTICS-DEPLOYMENT.json` for image and evidence bindings.

The production diagnostic observed a guardrail HTTP 403 with 890 seconds left
on a service token; subsequent identical and concurrent reads passed. Its failed
receipt is preserved. This does not establish a fixed backend denial or justify
refreshing every HTTP 403. The operator log retains only a request UUID, an
allowlisted IAM code when present, and remaining token lifetime.

A later Mac startup failure occurred after successful gateway-facing login and
native persistence. Keycloak rejected the gateway-held upstream refresh grant
as inactive. A fresh gateway/upstream consent flow obtained a new grant and
restored all eight tools; no gateway runtime fix is claimed.

Fresh coordinated Mac and Linux runs completed production sign-in, native
persistence and all eight tool results. Their two real frontend expiry cycles
remain pending. The gateway has renewed the shared upstream grant after actual
expiry without another browser prompt. Issuer cleanup waits for both workflows
to finish. Native tool-result checks are required in addition to process exit
status and gateway transport telemetry.
