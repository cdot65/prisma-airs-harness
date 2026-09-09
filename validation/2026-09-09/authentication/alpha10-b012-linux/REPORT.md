# Alpha10 Linux executable acceptance

Status: Linux checks recorded below passed; full release remains incomplete.
Runtime source: b012ba55e1404b498ad513cd15efa52a9a60530f.
Native SHA-256: a9166568623ce602da606a668359003e1f0a1380a202790da77f0a83dd1dc6ff.

The release build completed in 19m42s using the recorded profile. 388 scoped Rust
tests passed before compilation. The native integration suite passed 38 tests
with one macOS-only skip. Four managed-helper executable tests passed. The
published-alpha9 upgrade fixture passed all nine checks with old helper
executables absent, preserving credential bindings and existing history.
MCP relocation fixtures exercise the actual saved helper but disable remote
MCP transport; revision-first recovery is simulated state, not a process crash.
Old-client missing-helper rejection is not a downgrade compatibility claim.

The live workspace-key fixture passed native storage, authenticated probe,
default-route inference, local file effects, separate-process resume and logout.
Wire omission is established by loopback tests, not inferred from live replies.
Fresh scoped npm12 installation verified the exact native checksum, 70 bundled
packages, 3091 installed files and 67 license files, with four staged registry
requests and no unexpected requests or public redirects. CLI5.2.0/SDK0.28.0
remain pinned. No package has been published.

Two operator invocation errors were corrected without source changes: native
packaging initially lacked rustc on PATH; the first live OIDC invocation lacked
a private session D-Bus and failed before creating test users. Their raw logs
remain at /var/tmp/airs-alpha10-package.log and
/var/tmp/airs-alpha10-live-oidc.log. Subsequent command results are recorded
separately. No failed attempt is counted as passing.

All 38 installed-runtime tests passed. The managed Prisma AIRS CLI contract
passed, including real corpus generation; no live CLI management operation or
document-detection claim is made by that fixture. The live Keycloak/MCP fixture passed all 29 checks, including two interactive
token-expiry cycles, resource-bound identities, local tools, actual remote
scanner calls, logout and device reauthentication. Its temporary users were
removed; shared client availability and scanner policy were unchanged. The
receipt identifies exact native bytes; its null source field is resolved by
the independently recorded BUILD.json provenance, not rewritten in place.
Apple Silicon compilation, owner signing/notarization, and installed signed
Mac acceptance remain separate requirements. No native Windows or Intel Mac
acceptance is claimed.
