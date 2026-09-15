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

## Backend policy-denial investigation

The supplied Forgejo SSH key restored write access. Bounded backend diagnostics
and the SDK-aligned gateway tenant header are deployed to development and both
production replicas. All 59 backend tests, type checking, build and image scan
passed. The current image is
`sha256:b57dbdebe53c9fa36b69fea8559a53ee92198a48dfc3595f84ff54e5125df77f`.

A later Mac startup failure occurred after successful gateway-facing login and
native persistence. Keycloak rejected the gateway-held upstream refresh grant
as inactive. Fresh gateway/upstream consent obtained a new grant and restored
all eight tools; no gateway runtime fix is claimed.

Fresh coordinated Mac and Linux runs passed sign-in, persistence, all eight tools
and two activity intervals. Their third interval failed on management HTTP 403
with 896 seconds remaining on a service token. Neither reached frontend expiry.
Cleanup waited for both workflows, then passed both native MCP and inference
logout checks. `COORDINATED-ACCEPTANCE-FAILURE.json` binds the failed receipts.

The backend denial recurred after the tenant-header change, with
`x-opa-decision: false` and 894 seconds remaining on the token. IAM readback
confirmed the intended workspace scope and six read permissions. A stopped peer
soak is preserved as incomplete, not passed. Paired header and coexisting-token
diagnostics do not establish a token-expiry cause or a fixed policy decision.
No broader permissions or blanket HTTP 403 retries were introduced.

`BACKEND-POLICY-INCIDENT.md` is a prepared, unsent incident report containing
request IDs and the SCM authorization trace needed next. Alpha.15 remains
unpublished; exact native frontend lifecycle acceptance and owner upgrades are
blocked by the unresolved backend denial. The alpha.14 exception does not apply.
