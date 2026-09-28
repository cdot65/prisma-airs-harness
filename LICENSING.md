# Licensing and distribution

Prisma AIRS Harness uses Apache-2.0 for Codex-derived and project-owned code.
Preserve LICENSE and NOTICE, including OpenAI and Ratatui attribution. Calvin
Remsburg's copyright covers project-owned additions and modifications, not
upstream or third-party work. Product names and logos require separate rights.

## Modified upstream files

Adjacent `FILENAME.license` files prominently identify files changed relative to
the integrated Codex baseline `6b9826e3aa83b1a5947db50f4332cb9c65f1b340`.
These companion notices keep generated schemas, snapshots and upstream source
bytes free of annotation-only edits. Distribute each companion with its source
file. They identify fork-distribution changes, including later upstream imports;
they do not assign authorship of upstream changes to the fork owner or override
an existing file license. Git history retains detailed attribution.

Run `python3 scripts/check_airs_license_notices.py --write` after changing an
upstream file, then run without `--write` to verify. A missing baseline is an error;
use a full checkout for this check. This is a modification-notice check, not a
claim of complete REUSE certification or a legal opinion.

## Package boundaries

The wrapper and native packages include LICENSE and NOTICE. Native releases
include the resolved Cargo dependency notices and Rust distribution notices.
Vendored components retain their own license texts; Linux packages also carry
the bundled Bubblewrap source and build files under `licenses/bubblewrap-source`.
Do not replace third-party MIT, BSD or LGPL notices with Apache-2.0.

The separately versioned CLI and SDK source were changed to Apache-2.0 with the
owner's authorization on September 28, 2026. Existing MIT package versions,
including CLI 7.2.0 and SDK 0.34.0 currently pinned by this harness, retain MIT.
Only a future dependency update can bring their Apache-licensed releases into a
published harness. Never rewrite existing release artifacts or evidence.
