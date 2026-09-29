# Harness documentation

Docusaurus site for https://cdot65.github.io/prisma-airs-harness/.
Node 24: `npm ci`, `npx playwright install chromium`, then `npm run check`.
Use `npm start` for local development.

The Gruvbox CSS, Prism palette, logo and DocItem layout are reused from the
Apache-2.0 Prisma AIRS CLI documentation. Source guides and Rust help snapshots
are exported by `scripts/generate.mjs`; edit their originals, not `docs/generated`.
Builds publish the exact source commit and input hashes in `source.json`.

Forgejo validates changes. Only `airs-docs-<full SHA>` tags deploy through the
owned GitHub Pages workflow; see `docs/guides/deployment.md`. Source reference
pages track the checkout, while user guides explicitly distinguish stable from
preview behavior.
