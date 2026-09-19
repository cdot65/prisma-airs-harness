# Focus 2: authentication recovery hardening

The change preserves the initiating credential save/read/install error when rollback or cleanup also fails. Both file restores are still attempted before errors are combined. Existing typed recovery remains available, with the initiating recovery category taking precedence; a cleanup category is retained when the initiating failure has none. Journal formats, account ownership, credential namespaces and retry authorization are unchanged.

Generic native-store guidance now describes unavailable access in the current user session. It no longer assumes that an unreachable service proves the store is locked. The packaged README describes diagnostic and retry behavior.

## Evidence

- `auth-baseline/regression-red.json`: the new primary-read + failed-cleanup regression failed in both CLI test binaries for the intended missing-primary-context assertion. Existing binding/config and metadata-only journal checks passed before that assertion.
- `auth-baseline/rollback-red.json`: the separate installation + two rollback failures regression failed for missing initial installation context. The implementation then changed only error composition, preserving operation order.
- `auth-baseline/transaction-green.json`: all 42 transaction/namespace cases passed in both CLI test binaries. New tests exercise primary/secondary recovery precedence, only-primary/only-secondary errors, both rollback failures, secret canaries, retained metadata, retries and unrelated-account preservation.
- `auth-baseline/unavailable-guidance-red.json`: published mcp.2 fails the missing-D-Bus behavioral assertion because it instructs the user to unlock despite an unknown lock state. Its updated check passes in the final executable regression suite.
- `focus2-packages.json`: **5,603 passed, 6 skipped** in CLI, login, TUI, model-provider, AIRS identity and keyring-store packages. The first run had one temporary-path snapshot failure; an identical targeted run outside the home directory passed, then the complete affected selection passed with that layout. Neither source snapshot nor application behavior was changed to satisfy it.
- `focus2-executable-checks.json`: **46 installed regressions, 4 doctor and 4 MCP checks passed**, with one macOS-only skip. The isolated encrypted Secret Service fixture also passed with kernel credential syscalls denied, including save/read, read-only doctor and preserved binding.
- The final executable is frozen at native SHA-256 `e0c277cd8811669feeebf4ef80de700bf217d74599eed914dc81a585eab12f43`. `BUILD.json` and `SOURCE.patch` bind this development build to exact source files and patch. It still reports the baseline development version mcp.2; it is **not a published release candidate**. Later release acceptance must use newly versioned exact native packages on all three platforms.
- `focus2-interim-*` results are excluded from final acceptance: they used the preceding development binary before the ordinary executable was rebuilt. The final frozen-byte checks above supersede them without changing the historical record.

Formatting, scoped lint and diff checks passed. Independent review passed at **9.5/10** with all scoped gates satisfied. Runtime commit `d5b43416af3ac3d0c0d69c894d096de2798888cd` exactly matches the validated source patch; see `focus2-committed-source.json`. This is development acceptance, not package publication.

## Limits

The owner deferred the Ubuntu session investigation and attended production SSO/ServiceNow acceptance. Those remain deferred. The historical complete workspace result is still 18,097 passed / 150 failed / 34 skipped; this focused result does not relabel it. This work changes no gateway configuration, native credential owner session, helper protocol markers, authentication schema or upstream integration routing.
