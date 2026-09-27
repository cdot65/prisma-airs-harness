# Shared bounded tool previews — fullscreen prerequisite

Adapted upstream `4f3867123f`. Agent-command and MCP output now share a three-row preview with a count of hidden logical lines; partially visible logical lines count as hidden. Wrapping is bounded to 16 KiB per line and hard-wraps long URL tokens. Full transcript and raw output retain the result, including trailing failure diagnostics. User-shell output retains its existing presentation.

The three-way MCP conflicts came from upstream's separate computer-use and image-title changes. Preserved AIRS image summary/side-output behavior, CredentialHelper status, bounded titles and gateway status rendering. Added applicable code-mode preview tests without treating cua_repl as a special enabled computer-use service. No inference, MCP transport, OAuth, secure-store, model-context or raw protocol behavior changed.

Local history/exec/replay/tool-output subset: 234/234 passed, zero retries. New snapshots and intentional bounded-preview snapshot changes were inspected, including full MCP transcript preservation and omission hints. Existing gateway status tests pass. The connected fullscreen/native/release gate remains pending; this prerequisite is not assigned a standalone feature score.

Scoped config/core/TUI lint passed with zero warnings and no fixes. Formatting passed; unrelated Python formatter churn was restored.
