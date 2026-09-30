# Stable 0.1.5 release evidence

`RELEASE-EVIDENCE.json` binds both registry distributions of
`@cdot65/prisma-airs-harness` 0.1.5 to the three native binary hashes, candidate
and fresh registry acceptance, default installations, and exact-source workspace
review. The runtime source is the published 0.1.4 runtime plus its version stamp;
native packaging and validation tooling have separate revisions, recorded in
`DISPATCH.json` (build runs) and `READINESS.json`.

The full GNU run is [Forgejo run 4281](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/4281).
Its three-failure result remains in `READINESS.json`; no green full-suite claim is
made. Each failure is a reviewed disabled upstream remote-execution case with an
evidence digest, and `DISABLED-UPSTREAM.json` shows the installed command rejects
that service.

This release corrects the Ubuntu preparation helper bundled in the 0.1.4 launcher
tarball. `UBUNTU-HELPER.json` records the readiness checks of the helper copy
shipped inside the published launcher, run from the fresh public default
installation. `UBUNTU-HELPER-CANDIDATE.json` is the earlier run against the
pre-publication candidate installation; its single failed check is registry
availability of a version that was not yet published, retained as such rather
than re-labelled.

Both npm distributions contain the same native executables and bundled CLI/SDK.
Only registry and acceptance metadata differ. The upgrade path installs stable
0.1.4 from its registry and upgrades in place; no configuration rewrite is
accepted for a same-name upgrade.

Native fixtures cover browser/device OIDC, OS credential storage, MCP access,
terminal behavior, managed CLI, renewal, upgrades, rollback and a continuous
installed process. Their model/identity responses are controlled fixtures. Owner
release authorization is recorded separately from attended production account
acceptance.

Original logs, manifests and the restarted acceptance attempt are retained in the
owner's release evidence cache, indexed by the manifest digest. No publishing
credentials are included in this directory.
