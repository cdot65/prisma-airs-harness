# Review prerelease distribution assessment

Date: 2026-09-08. Reviewer: `/root/release_review`.
Scope: read-only P5/P6 release-path assessment, followed by separately requested
legacy publisher guard work. This is not a publication receipt or release approval.
The reviewer authored the guard implementation; root performed its separate review.

## Plan interpretation and remaining work

AUTHENTICATION-PLAN P5 explicitly permits immutable prerelease publication for P6.
P7 production promotion remains contingent on mandatory acceptance. P5 also requires
signed Apple Silicon/Windows and verified Linux packages. Existing source154 private
artifacts embed alpha.9 and must not overwrite the published alpha.9 versions.
Current ad-hoc/unsigned private artifacts cannot become publishable by deleting flags.

The remaining path is:

1. Define a distinct review-publication manifest for a new, unused prerelease version.
   Record exact source, included targets, signed final binary hashes, automated
   evidence and pending manual P6 gates. Do not fabricate a prior completed publication.
2. Bump the native AIRS_HARNESS_VERSION and npm manifest/lockfile coherently, freeze
   source, and rebuild every included target from it. Keep CLI 5.2.0 and SDK 0.28.0.
   Repackaging the current binaries would not change their reported alpha.9 identity.
3. Establish required signing access and verify final extracted bytes. Developer ID,
   notarization, stable upgrade identity and Windows signing remain unverified.
   Hosted ad-hoc Keychain success is not the owner's Mac incident reproduction or fix.
   A proposed unsigned public diagnostic tier would need a separate explicit plan
   decision; the current guards and P5 do not provide one.
4. Implement ordinary scoped npm installation with correct public dependency
   resolution. Current instructions deliberately use a separately resolved tarball:
   the harness, public CLI and public SDK share @cdot65. Changing only the CLI to
   a tarball is insufficient and breaks the current literal-version comparison in
   resolvePrismaCli. A generated published npm-shrinkwrap preserving exact public
   CLI/SDK and native URLs is a candidate solution to test, not an implemented fix.
5. Test fresh package-name/tag installation with an empty cache and intended scope
   mapping, actual native selection and hashes, managed CLI pin, installed Keychain
   lifecycle, alpha.9 credential/session migration, and relocation. Verify teammate
   read-only package access/inheritance separately from Keycloak roles. No Intel Mac.
6. Publish only the validated review version to a non-primary tag, then execute
   manual P6 trials on those exact bytes. Keep production promotion and its score
   separate. A changed runtime or signature invalidates affected acceptance.

npm documents shrinkwrap as a publishable lockfile suitable for globally installed
CLI applications; package-lock alone is not the distribution solution:
https://docs.npmjs.com/cli/v11/configuring-npm/npm-shrinkwrap-json/

Existing workflow GitHub authentication uses temporary GITHUB_TOKEN with
packages:write; acceptance uses packages:read. Authenticated operator API access
has worked. No additional long-lived publisher token is inherently required.
Teammate inheritance/read access and release signing credentials remain unverified.
GitHub package authentication is separate from harness identity:
https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry

## Implemented bounded legacy guard stage

The legacy copy publisher still requires a previously published, complete receipt
for exactly the launcher, Linux x64 and Apple Silicon. It is not the new P5 candidate
publisher. It now requires one explicit tag: auth-review, review or preview. It
cannot promote latest/stable/production. New publication passes --tag explicitly;
existing immutable versions are retagged only after all byte checks pass. The final
receipt records the tag after checking its registry mapping. Literal CLI pin and
native byte verification remain intact.

Before registry calls it rejects private/candidate/unsigned flags, negative or
malformed publication/validation booleans, release_status markers and unknown
status markers in receipt records, manifests and present BUILD-INFO provenance.
It verifies every source archive and native dependency set before publication.
Absent modern markers remain compatible with the historical completed release
receipt; these checks do not establish signing or full P5 acceptance.

The workflow passes the required tag, runs the mocked publisher suite before
registry access and selects explicit Bash so piped acceptance failures propagate.
Local tests: five passed, zero skipped, including 30 candidate-marker subcases,
invalid/missing tags, late-source checksum/dependency rejection with zero registry
calls, explicit publish tagging, immutable retry/tamper behavior, unchanged mocked
latest and literal CLI pin, plus the failing Bash pipeline boundary. YAML parsed
locally. This proves mocked command behavior, not live registry tag preservation.

No publication, version bump, runtime build, remote dispatch or signing change was
performed for this stage. Full authentication release readiness remains FAIL.
