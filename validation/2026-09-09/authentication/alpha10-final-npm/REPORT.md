# Alpha10 combined signed review distribution

Status update: owner confirmed all three repository associations already exist.
The earlier blocker below was caused by treating omitted REST metadata as null.
The corrected publisher records owner confirmation separately from API evidence.
Publication run34339198627 succeeded: all three exact alpha10 archives are now
published under auth-review and downloaded registry bytes match SHA256/SRI.
Fresh real-registry acceptance is running; no full-authentication completion claim. This is an owner-test distribution, not completion of
the full authentication release.

## Verified

- Runtime source b012ba55e1404b498ad513cd15efa52a9a60530f; npm packaging source
  00931ab58143bcb81b9b1f60bb6d9f565af7945c. No native recompilation or re-signing.
- Signed Apple Silicon Mac acceptance on macOS15.7.9 and26.6.2; final review
  packaging run34336173227. Actual Apple trust, online notarization and native
  Keychain lifecycle pass; owner's own device still requires hands-on retest.
- Final aggregate fresh Linux npm12.0.2 install:38/38 integration tests pass,
  native digest preserved, no unexpected registry requests. Managed CLI5.2.0
  passes20 capability help contracts and real synthetic corpus generation.
- Exact archive restore/replan passes. Bundle has72 packages,3102 archived files
  and68 licenses. Four declared optional changelog/lock files present in staging
  are omitted by npm; planner fix f458d3806 reports actual archive count while
  preserving required and optional-file integrity checks.
- Three tarballs, NPM-PACKAGES.json and PUBLICATION-PLAN.json uploaded to private
  draft release airs-harness-alpha10-signing-intake (385353104). Every upload
  response digest matches local SHA256. UPLOADED-ASSETS.json records identifiers.

## Remaining publication prerequisite

Historical read-only run34337733343 incorrectly rendered omitted repository
metadata as repository=null. Diagnostic34338874564 proves the field is absent.
The owner subsequently confirmed all three are already bound to cdot65/airs-harness:

- @cdot65/prisma-airs-harness
- @cdot65/prisma-airs-harness-linux-x64
- @cdot65/prisma-airs-harness-darwin-arm64

The repository field in package.json does not prove the registry association.
The earlier inspection made no mutations. Publication subsequently succeeded
after the metadata correction, with auth-review as the only requested dist-tag.
No non-owner install is claimed. The publisher now checks exact API package identity and private visibility, rejects
contradictory repository metadata, and explicitly records owner confirmation when
the REST field is omitted. Actual publishing and downloads verify workflow access.

## Exact operator continuation

After association is fixed, dispatch airs-harness-review-publish.yml on main
with mode=publish, release_tag=airs-harness-alpha10-signing-intake and
approved_plan_sha256=0661946b1c3174464b255306cabce8000cbd5e74617bf420ec9a6220fbf29583.
The only allowed publish tag is auth-review. The workflow rechecks exact archive
integrity and evidence, publishes native packages before launcher, downloads and
verifies registry bytes, and retains complete/partial publication receipts.

After successful publication, dispatch airs-harness-registry-acceptance.yml with
that publication_run, its exact signed-review-publication artifact ID, and the
same approved plan hash. It tests fresh real-registry installations on Linux and
Apple Silicon macOS with npm10.9.8 and12.0.2, including installed signature,
Keychain and bundled CLI checks. It does not establish teammate permissions or
live owner-device OIDC acceptance; retain those as separate gates.

The full authentication score remains unassigned. Prior independent9/10 scores
apply only to the bound owner-test distribution scope. Native Windows and Intel
Mac builds are not part of this distribution.
