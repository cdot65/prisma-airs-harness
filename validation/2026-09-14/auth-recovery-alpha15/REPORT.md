# Alpha.15 authentication recovery candidate

Both installed packages passed native executable, secure credential-store and
alpha.14 npm upgrade checks. The Apple Silicon binary is Developer ID signed
and notarized. These receipts do not establish production OAuth lifecycle
acceptance or publication. Alpha.14 remains the published release.

The frozen runtime source is `2f5a1ad306d379c23e21b60173b87876a79045c4`. Later
changes through `1deab7e4ca0eacb85864f38fbcc5321e4cc2fea4` affect acceptance
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
  No clean baseline comparison is claimed for the remaining failures.
- Full workspace compilation failed because Rusty V8 150.4.0 has no available
  Linux musl archive. Full workspace success and independent review are not claimed.

`INSTALLED-PACKAGES.json` binds the evidence hashes to the exact binaries. The
original logs and receipts are retained at their listed private artifact paths.
Apple Silicon build/acceptance is Forgejo run 125; signing is run 127.

## Production acceptance in progress

Linux completed browser inference login, gateway MCP consent, native credential
persistence and all eight production tools. Its two real frontend expiry cycles
are running with normal read activity and a quiet interval before expiry.

The first Mac login persisted correctly, but MCP initialization was challenged
and no tools became available. That failed run was cleaned up and preserved;
its cause is not established. The replacement Mac run passed all eight tools
and is also running two real frontend expiry cycles. Credential-read diagnostics
confirmed successful native Keychain reads without recording token values.

Each acceptance run uses a unique MCP server name. Isolated harness homes alone
do not isolate the OS-user MCP keyring. Tests must not replace or log out the
owner's `prisma-airs` credential. Gateway observations track only the acceptance
clients and Calvin's existing production upstream binding; no session policy or
credential state is changed by the observer.

Production expiry results, correlated gateway/upstream renewal evidence and
registry promotion are pending. The alpha.14 release exception does not apply.
