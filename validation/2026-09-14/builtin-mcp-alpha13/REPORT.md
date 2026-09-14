# Built-in MCP alpha.13 release acceptance

Published September 14, 2026 as `airs-harness@latest` and `@alpha` on Verdaccio.
Runtime source: `6892b94a13b456911bb59c12c70e7d94b75a0e4d`. The npm launcher and
both native packages are version `0.1.0-alpha.13`. See `npm-packages.json` and
`publication.json` for immutable archive hashes; source and packaging commits
are recorded separately inside the packages.

The normal executable uses existing Codex MCP OAuth. The gateway adapter flattens
tool declarations only for the HTTP wire and restores namespaces for dispatch.
Explicit read scopes and refresh grant preservation are included. Inference and
MCP keep separate credentials; adding native MCP preserves inference history.

| Acceptance | Linux x64 | Apple Silicon |
| --- | --- | --- |
| Real browser PKCE, native credential store, eight production read tools | Passed | Passed |
| Two expiry intervals, two concurrent fresh processes each | Passed | Passed |
| History preservation and separate local logout | Passed | Passed |
| npm launcher, compact and verbose interactive MCP inventory | Passed | Passed |
| Upgrade from alpha.12, no force/uninstall/manual removal | Passed | Passed |
| Both legacy symlink layouts; old executable and persistent settings retained | Passed | Passed |
| Anonymous fresh registry download, package/native/bundle integrity | Passed | Passed |
| Developer ID, hardened runtime, Apple notarization and downloaded signature | Not applicable | Passed |

Native executable hashes:
- Linux: `823f9e73d7df721379ba2b1b0f8c68b12cdb457aac3acb3618f1e6c92a8bb412`
- Mac: `7514bf987fe1f4da62f24c3b0a9880f3c5863ec8c80052edc2d1b619e6286d32`

Forgejo run 3399 (UI 98) compiled and tested the frozen Mac runtime. Run 3404
(UI 99) signed/notarized it and passed full native OAuth acceptance. Run 3411
(UI 104) passed npm, upgrade, Keychain, 44 executable checks (one platform skip),
and live interactive MCP acceptance. Linux executable checks passed after the
isolated executable-replacement fixture was corrected to resolve the installed
native payload; the other 43 checks had already completed with one platform skip.
Scoped Rust checks: 303 MCP-client, 208 CLI, 221 API/provider tests passed.

The first Linux expiry run called profile tools successfully but the model did
not call the requested workspace tool. That failed receipt is preserved as
`linux-initial-model-selection-failure.json`. The repeated full run explicitly
instructed direct tool use and disabled agent delegation, retained the same
required workspace-call assertions, and passed both cycles. No failure receipt
was relabeled as successful. An earlier SDK refresh-scope defect was corrected
before the frozen release build and has an HTTP regression test.

Upgrade fixture corrections exclude only transient `tmp/arg0` shims, retain
persistent-state comparisons, and resolve native bytes for the binary-replacement
PTY test. The interactive test checks `/mcp` compact status and `/mcp verbose`
tool names, matching the existing UI. These are test-driver corrections.

Both MCP deployments are Synced and Healthy at service revision `a69ee105`.
The disposable account, both temporary policy bindings and local credential
copies are deleted. Anonymous requests still receive 401; only Calvin's policy
binding remains. The Linux host migration preserved 15 configuration/history
files. Both hosts use the normal npm command; obsolete MCP aliases are removed.

This is measured acceptance for the authenticated-MCP internal alpha channel.
Full Rust workspace validation, independent review, Windows distribution and
Calvin's personal browser login are not claimed. Calvin completes his own
`work-calvin` MCP login using [MCP.md](../../../MCP.md).
