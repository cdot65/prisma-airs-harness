# Alpha.8 rename evidence

This directory records the standalone Prisma AIRS Harness rename and npm package
preparation. Verdaccio publication is outside this acceptance step.

`source-checks.json` records test/lint log digests and actual summaries. The
selected library run passed 7,360 tests; a later core-only run passed 2,436 after
the saved-catalog branding correction. These overlapping counts are not added
together. The 27 development executable checks include new and legacy homes,
legacy project settings, wire identification, and saved-catalog instructions.

`platform-ci.json` records successful native credential-store and npm launcher
jobs on Linux, macOS and Windows at source commit 7ae3d865a. Later runtime changes
correct a generated catalog greeting and the model-visible repository URL; they
do not change the native identity implementation or launcher behavior.

The full workspace attempt could not build V8 150.4.0 because its musl archive
returns HTTP 404. The product disables code mode. This is not a full workspace
pass. Scanner and gateway-identity tests cover source compatibility; this rename
does not redeploy those services or reconfigure Keycloak.

The optimized binary and installed command each passed a seven-turn live
workflow with three real scans. The optimized executable passed 27 checks.
The installation preserved all 11 checked state files; npm artifact acceptance
is recorded separately after package construction. No new owner OIDC E2E is claimed from workspace-key tests.
