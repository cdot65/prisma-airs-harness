# 0.1.1 release evidence

The exact version is published under `stable-candidate` at `https://npm.cdot.io`.
All four packages are also promoted to `latest`. Fresh unversioned default
installs passed on all three platforms; other prerelease tags are preserved.

Candidate acceptance and fresh anonymous registry acceptance passed on Linux
x64, native Linux ARM64 and signed/notarized Apple Silicon. Each platform passed
installation, onboarding, terminal, installed regression, MCP management,
diagnostics, bundled CLI, upgrade and command-output checks. Native MCP storage,
reuse in a second process, and logout were exercised with isolated fixtures.

Additional exact-candidate lifecycle checks completed two expiry/renewal cycles
for inference and MCP and three verified synthetic read-only tool turns per
platform. Both mcp.6 and onboarding.4 upgrade paths preserved checked local
configuration and legacy command targets. The lifecycle fixtures do not assert a
production issuer's long-duration idle or revocation behavior.

The owner reports that inference sign-in, ServiceNow sign-in through `/mcp`, a
read-only query, and reuse after restart all passed on Apple Silicon with mcp.6.
That binary was built from `a0a1b5c8d4306e23e75816139ba8538d95da0b92`.
The 0.1.1 runtime source is unchanged except for its version stamp; test-fixture,
release-tooling and documentation changes are tracked separately.

`SPEC.json` binds the published binaries, packaging and acceptance tooling to
`b91d706e292791848394f30128b5f97aa53c4b9d`. `PROMOTION-TOOLING.json` identifies the
separately reviewed promotion policy. Source/version/hash checks apply to every
platform. Candidate and registry receipts remain distinct.

The original GNU diagnostic is preserved as a failed run: 18,302 passed,
17 failed and 34 skipped. Its source is `61a985`, not the final native build.
The final native-source GNU run 253 recorded **18,313 passed, 6 failed,
34 skipped**. It is retained as a failed full-workspace run. The six failures
have individual dispositions in `READINESS.json` and `baseline-evidence/`:
three inherited remote-shell tests concern an explicitly disabled upstream
service; two ambient test fixtures and the voice test runtime passed focused
follow-up in run 255. No application runtime change was needed. The focused
run does not replace the full run or claim the disabled upstream service passes.

`WORKSPACE-ARTIFACT-*.json` binds raw log hashes and source receipts; compressed
logs are included. `STABLE-PROMOTION.json` retains the nonzero result, package
identities, original tags and completed native-first promotion. The promotion
policy permits only these named, evidenced 0.1.1 dispositions. Unknown failures
remain blocking.

The release source integrates CI/test corrections after the native build source.
Temporary remote-executor diagnostic logging was removed before main integration;
the diagnostic commit remains reachable for reproducing run 255. See
`UPSTREAM-BASELINE-IDENTITY.json` for unchanged implementations and test-only diffs.
