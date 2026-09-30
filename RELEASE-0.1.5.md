# Prisma AIRS Harness 0.1.5

Stable 0.1.5 is a corrective release of `@cdot65/prisma-airs-harness` for Apple
Silicon, Linux x64 and Linux ARM64. The runtime is unchanged from 0.1.4 apart from
its version stamp; it bundles Prisma AIRS CLI **7.2.0** and SDK **0.34.0**.
Inference and remote MCP traffic use AI Gateway; provider credentials and upstream
OAuth stay at the gateway. The command remains `airs`.

## Install

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.5 --registry=https://registry.npmjs.org
airs --version
airs cli --version
```

The same release is available at `https://npm.cdot.io`. Node requires
`^22.13.0 || >=23.5.0`. Keep optional dependencies enabled; they select the native
package for your host. Restart existing sessions after upgrading.

Upgrading from 0.1.4 is an ordinary in-place update. An existing `airs-harness`
installation (0.1.3 or earlier) migrates once: uninstall the old package to
release its command links, then install the renamed package; AIRS environments,
credential bindings and conversation history are preserved:

```sh
npm uninstall -g airs-harness
npm install -g @cdot65/prisma-airs-harness@0.1.5
```

Never use `--force`. The native packages are `@cdot65/prisma-airs-harness-<platform>-<arch>`
and the Apple Silicon signing identifier remains `airs-harness`.

## Changes since 0.1.4

- The Ubuntu preparation helper bundled in the npm package (`scripts/prepare_airs_ubuntu.sh`)
  installs, checks and fetches `@cdot65/prisma-airs-harness`. In 0.1.4 its
  installed-version and registry checks still named the unscoped launcher path,
  so it reported `NOT READY` against a correct installation. The helper now
  derives every path from a single package declaration.
- The release tooling verifies the helper as packaged: the packager checks the
  declaration against the published launcher and version, and stable staging
  reads the archived copy and refuses a helper that names another launcher.
- No runtime changes: the native binaries are built from the 0.1.4 runtime source
  plus this version stamp.

## Validation record

The native binaries are built from the tagged source; Apple Silicon is Developer
ID signed, hardened and notarized. Native candidate testing covers installation,
OIDC and native credential storage, terminal behavior, MCP management, doctor,
bundled CLI, upgrade/rollback and output integrity on all three platforms. Each
installed regression suite contains 61 tests with platform-specific skips. Extra
checks run a continuous installed process against synthetic HTTPS OIDC and
gateway MCP fixtures.

The upgrade round trip installs stable 0.1.4 from its registry, upgrades in place
to 0.1.5 (same launcher name, no uninstall, no configuration rewrite), resumes a
real conversation, then rolls back to the exact previous bytes.

The Ubuntu preparation helper was verified **as packaged**: the packager and the
stable staging gate both read the shipped `scripts/prepare_airs_ubuntu.sh` and
require it to name `@cdot65/prisma-airs-harness` at 0.1.5, and the configured
helper from the fresh public default installation passed all isolated
keyring/readiness checks. A pre-publication run against the candidate
installation passed every check except registry availability, which cannot pass
before the version is published; that receipt is retained as such.

Fresh anonymous registry acceptance passed on Apple Silicon, Linux x64 and
Linux ARM64 for both npm distributions. Fresh unversioned default installs also
passed on all three platforms against each registry. The
[release evidence](https://github.com/cdot65/prisma-airs-harness/tree/airs-harness-v0.1.5/validation/2026-09-30/stable-0.1.5)
records binary hashes, source identities and platform results.

The full GNU workspace run recorded **19,013 passed, 3 failed and 35 skipped**.
It is not a green full-suite result. The three unchanged upstream
remote-execution failures match the reviewed baseline, and the installed AIRS
command rejects that disabled service. The stale `codex` help snapshots that
failed in the 0.1.4 run were corrected before this source and pass here.

Candidate acceptance was restarted once after two release-tooling defects in
the same-name upgrade path (the by-name uninstall introduced for the 0.1.4 rename
ran for every upgrade, and the installed-launcher check counted one name twice).
The restart used corrected tooling against unchanged native packages; the failed
run is retained with the passing one. The public registry verification was
repeated once on Apple Silicon and Linux ARM64: minutes after the 0.1.5 versions
went live, npm served package documents from which the optional 0.1.4 native
was silently skipped during the previous-version install; the repeat, with
unchanged tooling and packages, passed. Public publication itself waited on
npm's automated review of each new version.

Owner authorization requested this corrective stable publication. Automated
acceptance uses controlled identity, inference and MCP fixtures; it does not
claim a fresh attended Entra login, production ServiceNow call or paid Jev quality
evaluation. The site distinguishes these checks from your deployment's
[acceptance workflow](https://cdot65.github.io/prisma-airs-harness/validation/acceptance).

Runtime source: `312fe9c5fda63332c148670327db2620b4d6e82f` (the 0.1.4 runtime
`946dc6e34c9bd7f439c5fa00a9c05a11bee53673` plus the version stamp).
Native packaging: `df0ea51003705c4180710f1d3722896115657074`.
Acceptance, publication and promotion tooling: `a153f60a383944410085d9e06e11b1606f211252`.
Build runs: Linux x64 4275, Linux ARM64 4280, workspace 4281, macOS 4282, signing 4295.

Public and private distribution payloads share the same native binaries and CLI
bundle. Registry metadata and validation provenance are recorded independently.
Public upgrade tests fetch the previous stable 0.1.4 from its original registry.

The Ubuntu preparation helper contains a sanitized gateway endpoint. Configure
the downloaded copy with your actual listener as shown in the Ubuntu guide before
running readiness checks. An unchanged example hostname is not a working gateway.

## Roll back

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.4
```

Close running sessions first. Preserve environments, credential bindings and
conversation history. The 0.1.4 helper defect described above applies to the
rolled-back package; use the 0.1.5 helper from the repository if you need it.
