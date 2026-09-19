# Independent final test-publication review

Read-only review of exact `0.1.0-alpha.22.mcp.3` publication. Use frozen validation
tooling from `release/validation-tooling`, bound to the specification's full
commit IDs. Do not publish, retag, change source, or use owner credential state.

## Baseline before publication

- Verify all copied tooling files against the manifest **and** the declared git
  commit, plus the transferred tooling archive digest.
- Re-run frozen candidate acceptance collection for all three native targets.
- Re-run frozen `verify_staged`: staged metadata must bind the exact accepted
  archives and preserve native/launcher payloads.
- Retain immutable specification, native hashes, staged-plan digest and the
  prepublication registry snapshot digest. Record installed suite totals and
  skips without converting skipped tests into passes.

## After root reports publication and fresh registry acceptance complete

- Require `PUBLICATION.json` to be complete, scoped `owner-authorized-test`, bound
  to the exact specification/staged plan, and ordered native-first/launcher-last.
- Verify candidate and staged evidence again with the frozen tools; reject
  identity or output changes since this baseline.
- Collect all three fresh `registry` acceptances using the frozen verifier.
  Candidate receipts must not substitute for registry receipts.
- For every target, check exact installed native hash/source/version, successful
  anonymous fresh installation and isolated npm configuration/cache, and channel
  checks before/after install. Reconcile required fixture suites, managed CLI,
  terminals, upgrade, command guidance and Mac signature/notarization.
- Compare registry acceptance package-set hashes to the **staged** package set,
  not to the pre-staging candidate tarballs (metadata changes alter archives).
- Read anonymous registry metadata independently. All four exact versions and
  immutable archive integrities must match staging; every `mcp` selector must
  resolve to the new version. Compare **all** non-`mcp` tags with REGISTRY-BEFORE,
  not only `latest`, `alpha`, and `onboarding`.
- Preserve the distinction between installed fixture acceptance and attended
  production SSO, user workspace keys, ServiceNow operations, or the owner's
  Ubuntu credential-session issue. Those real-account items remain deferred.

## Readiness gate

Do not issue a final ready-for-test judgment until the publication receipt,
independent registry observations and all three fresh installed registry
acceptances agree. Do not claim stable promotion or real-account acceptance.
A missing/failed target, changed identity, changed protected tag or mismatched
registry archive is a blocking finding rather than a scored-away limitation.
