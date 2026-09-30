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

Pending. This section is completed with the platform results, the readiness run
and the registry acceptance before publication; the release is not ready until
then.

## Roll back

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.4
```

Close running sessions first. Preserve environments, credential bindings and
conversation history. The 0.1.4 helper defect described above applies to the
rolled-back package; use the 0.1.5 helper from the repository if you need it.
