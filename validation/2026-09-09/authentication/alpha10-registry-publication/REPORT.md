# Published signed alpha10 review acceptance

All three private scoped0.1.0-alpha.10 packages are published to GitHub Packages
under auth-review. Publication34339198627 verified downloaded archive bytes;
approved plan SHA256 is0661946b1c3174464b255306cabce8000cbd5e74617bf420ec9a6220fbf29583.
Fresh registry acceptance34339734562 passed all four Linux/Mac × npm10/npm12 jobs.
Linux passed38/38 tests per npm. Mac passed37 with one Linux-only skip per npm,
plus actual installed Keychain lifecycle, Developer ID/team, hardened runtime
and online notarization. npm workflow34339716168 passed on Linux and Mac too.

Independent published review awards9/10 for signed-prerelease-distribution and
finds no blockers to owner hands-on testing. It rechecked exact evidence ZIP
hashes and registry/native identities, not just workflow labels. Full project
acceptance, owner-device incident resolution, live owner Mac OIDC and a separate
teammate's package access remain distinct, unproven gates.

The package-association blocker was our checker error. GitHub's REST response
omitted repository; the owner confirmed links already existed. Corrected checks
record owner evidence separately, validate exact API package identity/private
visibility, and reject explicitly conflicting metadata. Actual successful
publication and registry installations establish this workflow's access.

First acceptance failures are preserved under first-acceptance-failure. They
were corrected by npm12 metadata normalization and established CI sandbox/search
prerequisites; no approved native or package archive changed. The unrelated
fast-CI actionlint installer was replaced with checksum-pinned upstream binaries
because the old action incorrectly fell back to searching Cargo for actionlint.

Use MACOS.md and AUTHENTICATION-ONBOARDING.md for installation and guided login.
