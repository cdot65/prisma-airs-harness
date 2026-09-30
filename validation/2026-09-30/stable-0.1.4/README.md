# Stable 0.1.4 release evidence

`RELEASE-EVIDENCE.json` binds both registry distributions of
`@cdot65/prisma-airs-harness` to the three native binary hashes, candidate and
fresh registry acceptance, default installations, and exact-source workspace
review. Runtime source, native packaging and validation tooling have separate
revisions; the tooling revision advanced twice during acceptance for rename-specific
fixture defects while the native packages stayed unchanged.

The original full GNU run is [Forgejo run 4226](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/4226).
The snapshot-only correction passed [run 4245](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/4245).
The original five-failure result remains in `READINESS.json`; no green full-suite
claim is made. Each failure has an explicit disposition and evidence digest.

Both npm distributions contain the same native executables and bundled CLI/SDK.
Only registry and acceptance metadata differ. The upgrade path installs stable
0.1.3 as `airs-harness` from its original private registry and replaces it by
name; the only accepted configuration change is the stored native command path
moving between the two launchers' installed package trees.

`UNSCOPED-DEPRECATION.json` records the earlier same-day publication of these
binaries as the unscoped `prisma-airs-harness@0.1.4` with legacy-named native
packages, now deprecated on both registries with a pointer to the scoped launcher.

Native fixtures cover browser/device OIDC, OS credential storage, MCP access,
terminal behavior, managed CLI, renewal, upgrades, rollback and a continuous
installed process. Their model/identity responses are controlled fixtures. Owner
release authorization is recorded separately from attended production account
acceptance.

The Ubuntu helper was configured with the deployed gateway URL before its six
isolated keyring/readiness checks. `UBUNTU-HELPER.json` records that the checks
ran against the repository copy of the helper at this release: the copy bundled in
the npm package still names the unscoped launcher path and fails its
installed-version check. The helper intentionally ships an example endpoint; the
documentation includes the replacement step.

Original logs, manifests and failed/restarted acceptance evidence are retained in
the owner's release evidence cache, indexed by the manifest digest. No publishing
credentials are included in this directory.
