# Review prerelease distribution assessment

Date: 2026-09-08. Reviewer: `/root/release_review`.
Scope: P5/P6 release-path assessment, legacy publisher guard work, and a
22:40 UTC update reflecting separately implemented private bundle acceptance,
followed by the Windows command-wrapper source-stage implementation. This is not
a publication receipt or release approval.
The reviewer authored the guard implementation; root performed its separate review.

## Plan interpretation and remaining work

AUTHENTICATION-PLAN P5 explicitly permits immutable prerelease publication for P6.
P7 production promotion remains contingent on mandatory acceptance. The owner
subsequently deferred native Windows in plan v0.2. Current P5 requires signed
Apple Silicon and verified Linux packages; Windows evidence is retained for a
later milestone and does not block this release. Existing source154 private
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
   notarization and stable upgrade identity remain unverified. Windows signing is deferred.
   Hosted ad-hoc Keychain success is not the owner's Mac incident reproduction or fix.
   A proposed unsigned public diagnostic tier would need a separate explicit plan
   decision; the current guards and P5 do not provide one.
4. The scoped package-name install implementation now exists as opt-in
   `--scoped --bundle-cli`, preserving exact native/CLI/SDK versions and candidate
   guards. [Maintained Linux bundle acceptance](maintained-bundle-d0c/REPORT.md)
   passed on npm10 and npm12, including actual CLI diagnostics and image/document
   generation. The bundle selects Sharp payloads from supplied native targets,
   preserves licenses, and verifies installed files against recorded hashes and
   the documented npm shebang normalization. Hosted bundled npm12 acceptance
   passed on [macOS 15](bundled-mac-719/macos-15/README.md) and
   [macOS 26](bundled-mac-719/macos-26/README.md), including installed native
   Keychain lifecycles and actual managed CLI document/image generation.
   The final [macOS 26 npm10.9.8 baseline](bundled-mac-719/macos-26-npm10/README.md)
   also passed, matching the npm version shown in the owner's installation transcript.
   Those trials reused the original source154 native artifact without compilation.
   Windows command-wrapper validation now has a constrained, byte-exact
   implementation with retained upstream templates and local negative tests;
   see [its separate source-stage evidence](windows-command-wrappers/REPORT.md).
   Actual Windows installed execution remains unverified and deferred. Real GitHub
   access, Mac signing and migration remain current release gates.
   The [earlier synthetic experiment](install-feasibility/REPORT.md) is historical
   feasibility evidence, and the shrinkwrap proposal remains superseded.
5. Test fresh package-name/tag installation with an empty cache and intended scope
   mapping, actual native selection and hashes, managed CLI pin, installed Keychain
   lifecycle, alpha.9 credential/session migration, and relocation. Verify teammate
   read-only package access/inheritance separately from Keycloak roles. No Intel Mac.
6. Publish only the validated review version to a non-primary tag, then execute
   manual P6 trials on those exact bytes. Keep production promotion and its score
   separate. A changed runtime or signature invalidates affected acceptance.

Older npm documents described shrinkwrap for published CLI applications, but npm12
removed that mechanism and denies direct remote-URL dependency specs by default.
The local controls preserve those failures and test a bundled alternative without
weakening npm security defaults. See the [experiment report](install-feasibility/REPORT.md)
and [npm12 documentation](https://github.com/npm/cli/blob/latest/docs/lib/content/configuring-npm/package-lock-json.md).

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
performed for the legacy publisher guard stage. The separately recorded bundle
acceptance used hosted artifact-only dispatches. Full authentication release
readiness remains FAIL.
