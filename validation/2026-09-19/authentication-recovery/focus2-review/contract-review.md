# Focus 2 independent contract review

Read-only reviewer: delivery_plan. Reviewed CLI credential transaction, native diagnostic adapter, recovery enum, welcome error rendering, main helper recovery downcast and regression tests. No application edits by reviewer.

## Required behavior

1. The initiating save/read/install failure remains an anyhow source so existing typed recovery downcasts continue to work.
2. A secondary cleanup failure is reported alongside the initiating failure. If only cleanup supplies CredentialRecovery, that typed enum remains available. If both supply one, the initiating typed recovery wins.
3. Both rollback operations are attempted even when one fails; report each failure without discarding the original installation failure.
4. Diagnostics include operation/category/native numeric status only for keyring errors. Never include raw backend errors, tokens, snapshots, config contents or provider response bodies.
5. Pending metadata journal remains on incomplete cleanup; recovery retries only the uncommitted account and preserves unrelated account, existing binding and config.
6. An unavailable credential service is not evidence that a keyring is locked. Neutral session-aware wording is accurate; keep machine marker unchanged.

## Initial findings

The proposed cleanup branch correctly retains primary.context(...) and copies cleanup CredentialRecovery only when the primary lacks it. Existing NativeStore.report supplies safely classified bounded text rather than raw keyring error content. Fixed-depth composition here adds no retry loop or repeated remote work.

The original rollback branch had the same diagnostic-loss pattern: config_restore? then binding_restore? could omit the original install cause and second rollback cause. Both restore functions already run before either ?; preserve that execution property while composing errors. Added filesystem fixture makes both rollback paths fail using directories in place of credential files; this exercises real I/O rather than implementation-mirroring mocks.

The welcome screen uses full `{error:#}` formatting, so safe composition at the source is essential. main.rs directly downcasts anyhow for the machine-readable recovery marker; flattening either primary or selected typed recovery into one formatted string is a functional regression.

## Review gates before score

- Expected RED demonstrates initial-cause masking with previous binding/config/journal preserved.
- GREEN targeted transaction tests cover read/delete cause retention, typed primary-only/secondary-only/both precedence, later successful cleanup, unrelated credential preservation, and rollback cause retention.
- Existing CLI and login scoped suites pass; changed installed missing-bus assertion passes against newly built runtime, with no provider request and no plaintext fallback.
- No token canary in diagnostic or journal; no credential format, identity binding, durable state schema or security boundary change.
- Formatting, clippy and diff review completed under repository policy.

This note records contract analysis, not completed acceptance or a final score. Attended production and owner-specific Ubuntu checks remain owner-deferred.

## Final implementation read (before broad validation)

No blocking code findings in the reviewed patch. The shared private `preserve_primary_failure` helper preserves the original error chain and first recovery classification, copies a secondary recovery classification only when absent, and adds only classified store/local filesystem diagnostics. Both rollback operations still execute before either result is composed. The cleanup path preserves its pre-existing success/ownership behavior; no credential deletion decision, account identity, format or journal schema changed.

The final tests exercise primary-only, cleanup-only and combined typed recovery, preserve existing binding/config, retain metadata-only journal without token canary, retain both rollback failures and original typed installation failure, and demonstrate later cleanup plus unrelated-key preservation. Neutral StoreUnavailable text keeps its existing machine marker. The installed missing-bus case checks absence of the unjustified unlock diagnosis and retains doctor/no-plaintext guidance with no network request.

Recorded exact reviewed file/diff hashes in `code-review.json`; `reviewed.patch` is the reviewed diff, not an application edit. Final acceptance must use a copied frozen executable with recorded checksum. Checks against a debug executable path while another build can replace it are interim evidence only. Score remains pending the final broad checks and frozen-runtime installed evidence.
