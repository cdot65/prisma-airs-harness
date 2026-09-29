---
title: Build and contribute
---

This page is for people working on the harness or on this site. The first part
explains how the repository is organized and why the build has the shape it does.
The second part is the procedure: clone, build, test and run the docs locally.

## How the repository fits together

Forgejo is the canonical source and review location. GitHub mirrors source and
hosts this documentation.

**Two artifacts, two names.** The Rust workspace builds a native executable named
`airs-harness`. Users do not run that directly. The npm package provides the
user-facing `airs` launcher, selects the native package for the host, and bundles
the product CLI. A source binary alone is therefore not the complete distribution:
it lacks the launcher and the bundled CLI.

**Why sandbox tests can mislead.** Linux sandbox tests need working Bubblewrap
namespaces. If the probe is skipped, the sandbox behavior went untested, so a skip
is not runtime acceptance.

**Why the docs are partly generated.** Command pages come from the Rust help
snapshots, and some guides come from repository files listed in `sources.json`.
Generating them keeps the site in step with the code. It also means generated
pages are ignored by Git and rebuilt on every site build, so an edit made directly
to one is lost on the next build. Builds reject broken internal links and anchors
for the same reason: the site is assembled from several sources, and a link that
worked in one place can break in another.

**What the published site records.** `source.json` in the published site records
its commit, source package versions and input hashes, so you can tell exactly what
a deployed site was built from.

## Build and test

Clone the canonical repository:

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

Use `just test` for Rust verification, scoped to affected crates, and `just fmt`
for formatting. Package contracts also exercise the npm launcher, immutable
release inputs and installed acceptance scripts.

## Develop the documentation

```sh
cd docs-site
npm ci
npm run check
npm start
```

Install Chromium once with `npx playwright install chromium` before browser tests.
Edit authored guides under `docs-site/docs`. The site uses the SDK/CLI Gruvbox
palette, fonts, syntax theme, sidebar and collapsible on-page navigation.

`npm run generate` exports the guides selected in `sources.json`, rewrites source
links and creates command pages from the Rust help snapshots. When commands
change, update the Rust help snapshots through their owning tests. Do not
hand-edit generated reference pages. Browser tests cover every published route,
navigation, diagrams and mobile layout.

See [deployment](deployment.md) for the Forgejo check and GitHub Pages handoff.
