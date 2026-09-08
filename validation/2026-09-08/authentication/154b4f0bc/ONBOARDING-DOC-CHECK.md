---
title: Candidate onboarding documentation check
status: reviewed-with-known-ui-limitation
created: 2026-09-08
source_commit: 154b4f0bcef7528844e85177d7e4dce611982044
tested_cli_commit: 3acf1f045
tested_cli_sha256: 39d238e30af3ef0398abacfc3bac6efdde6404dca0cf49eb0a876c72f2c50eda
scope: Candidate guide and command help; not native desktop E2E acceptance
---

Reviewed `AUTHENTICATION-ONBOARDING.md` against the captured Linux executable's
successful help output for the root command, `login`, `doctor`, `setup`, `env`,
`resume`, and `setup-mcp`. Source review checked saved company-setting prompts,
Windows Ctrl+Enter submission, metadata-only native status, gateway verification,
and package launcher dispatch. This review did not execute native macOS/Windows
UI flows or new authenticated requests.

Corrected the guide's source reference, returning company-sign-in instructions,
credential preservation wording, Linux service/recovery limitation, and new
environment requirement for identity changes. The guide remains explicitly for
an unpublished private candidate; historical alpha.9 installation instructions
were not changed.

Known UI follow-up: root CLI help still describes `status` as **“Show the selected
gateway and local credential availability”**. This wording is stale: native-bound
status reports saved metadata and explicitly says availability and gateway access
are not checked. Production help was left frozen for the current build. Do not
claim that all help text is aligned until this wording is corrected and tested.

All commands shown in the candidate guide exist in the native CLI. The npm
launcher additionally dispatches `airs-harness airs` to its pinned Prisma AIRS
CLI dependency; the raw native binary does not expose that launcher command.
The guide does not claim otherwise or promise registry publication, signing,
headless recovery, or completed owner-Mac acceptance.
