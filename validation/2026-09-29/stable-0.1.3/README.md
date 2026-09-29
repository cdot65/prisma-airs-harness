# Stable 0.1.3 release evidence

`RELEASE-EVIDENCE.json` binds both registry distributions to the three native
binary hashes, candidate and fresh registry acceptance, default installations,
and exact-source workspace review. The source version stamp is the only runtime
change from alpha.7; documentation and validation tooling have separate revisions.

The original full GNU run is [Forgejo run 333](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/333).
The schema-only correction passed [run 339](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/339).
The original six-failure result remains in `READINESS.json`; no green full-suite
claim is made. Each failure has an explicit disposition and evidence digest.

Both npm distributions contain the same native executables and bundled CLI/SDK.
Only registry and acceptance metadata differ. The public registry upgrade path
uses the previous stable package from its original private registry.

Native fixtures cover browser/device OIDC, OS credential storage, MCP access,
terminal behavior, managed CLI, renewal, upgrades, rollback and actual-agent skill
execution. Their model/identity responses are controlled fixtures. Owner release
authorization is recorded separately from attended production account acceptance.

The Ubuntu helper was configured with the deployed gateway URL before its six
isolated keyring/readiness checks. The npm helper intentionally ships an example
endpoint; the documentation includes the replacement step.

Original logs, manifests and failed/retried fixture evidence are retained in the
owner's release evidence cache, indexed by the manifest digest. Public examples
omit production realm, stack, user and group names. No publishing credentials are
included in this directory.
