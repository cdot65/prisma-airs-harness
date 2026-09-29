---
title: Build and contribute
---

Forgejo is the canonical source and review location. GitHub mirrors source and
hosts this documentation. Clone the canonical repository:

```sh
git clone git@git-ssh.cdot.io:cdot/prisma-airs-harness.git
cd prisma-airs-harness
```

Read `AGENTS.md` for current repository conventions, the pinned Rust toolchain
and testing requirements. Build the native executable from the Rust workspace:

```sh
cd codex-rs
cargo build --locked --release -p codex-cli --bin airs-harness
./target/release/airs-harness --version
```

The native artifact is named `airs-harness`; npm provides the user-facing `airs`
launcher and bundled product CLI. A source binary alone is not the complete npm
distribution.

Use `just test` for Rust verification, scoped to affected crates, and `just fmt`
for formatting. Package contracts also exercise the npm launcher, immutable
release inputs and installed acceptance scripts. Linux sandbox tests need working
Bubblewrap namespaces; a skipped sandbox probe is not runtime acceptance.

## Documentation development

```sh
cd docs-site
npm ci
npm run check
npm start
```

Install Chromium once with `npx playwright install chromium` before browser tests.
The site uses the SDK/CLI Gruvbox palette, fonts, syntax theme, sidebar and
collapsible on-page navigation. Edit authored guides under `docs-site/docs`.

`sources.json` selects maintained repository guides and release receipts.
`npm run generate` exports them, rewrites source links and creates command pages
from the Rust help snapshots. Generated pages are ignored by Git and rebuilt on
every site build. Update the Rust help snapshots through their owning tests when
commands change; do not hand-edit generated reference pages.

Builds reject broken internal links and anchors. Browser tests cover every
published route, navigation, diagrams and mobile layout. `source.json` in the
published site records its commit, source package versions and input hashes.

See [deployment](deployment.md) for the Forgejo check and GitHub Pages handoff.
