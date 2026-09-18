# MCP manager npm publication — September 18, 2026

The owner explicitly instructed **“PUBLISH IT NOW”** after the archive-only
handoff and npm ETARGET. This authorizes the versioned npm test release despite
pending attended production gateway/upstream ServiceNow acceptance. The scope is
`owner-requested-mcp-manager-testing`; full production acceptance stays false in
each packaged VALIDATION.json. No release validator or repository policy was
rewritten to manufacture that acceptance.

Version: `0.1.0-alpha.22.mcp.1`. Registry: `https://npm.cdot.io`. Publication tag:
`mcp`. Existing `latest`, `alpha`, and `onboarding` tags are preserved. The launcher
includes CLI 7.0.0 / SDK 0.33.0 and exact native packages for Linux x64, Linux ARM64,
and Apple Silicon. Runtime source is `15a7f229bcfd08677ca9f193b8ff05db41f30737`.
The optimized release builds differ from the earlier unoptimized Linux review
archive; both retain their own source/profile/hash receipts.

Owned Forgejo runs: Linux x64 221, Linux ARM64 220, Apple Silicon build/acceptance
219, Developer ID signing/notarization 222. Linux ARM64 installed checks ran
natively in Jadzia's ARM64 Docker VM. Mac installed checks ran in its GUI session.
The first Mac/ARM onboarding test attempts lacked the copied test-only JWKS
fixtures; copying the fixtures fixed the acceptance setup without changing or
rebuilding executables. Final checks all passed.

Per platform: 46 installed regressions passed, one platform-inapplicable skip;
four MCP manager terminal cases passed; three upgrade cases from onboarding.4
preserved configuration; managed CLI and 15 command-output checks passed. Native
onboarding/credential-store checks passed (18 Linux x64, 18 Linux ARM64, 12 Mac).
`ACCEPTANCE.json` binds all results to native hashes. `INSTALLED-MAC-SIGNATURE.json`
verifies Developer ID and notarization on the exact installed Mac executable.

`stage.py` preserves the original build-candidate metadata, adds the explicit
owner-publication scope and limitations, and removes npm's private marker. It
checks that only package/build/validation metadata changed inside archives;
executables, launchers, managed CLI and other runtime files must remain identical.
`publish.py` publishes native packages before the launcher and verifies registry
integrity for every immutable version. Fresh anonymous registry-install receipts
are recorded separately after publication.

The owner-specific Linux credential-store failure remains unresolved. Isolated
fixtures do not establish production ServiceNow consent or tool results. The
prior full-workspace failures are not recast as passing; this feature's affected
CLI/TUI suite passed 5,280 tests with six skips.

Publication completed. Fresh anonymous registry installs passed on all three
platforms, including four in-session MCP terminal fixtures per installed package,
environment lifecycle, bundled CLI version and command-output checks. Each native
hash matches its accepted artifact. All four registry tarball integrities match
staging. `mcp` points to the new version; every pre-existing tag is preserved.

```sh
npm install -g airs-harness@0.1.0-alpha.22.mcp.1 --registry=https://npm.cdot.io
airs --version
```

Start `airs`, then enter `/mcp` to manage gateway MCP connections.
