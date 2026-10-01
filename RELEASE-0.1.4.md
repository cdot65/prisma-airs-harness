# Prisma AIRS Harness 0.1.4

Stable 0.1.4 is the first release published as `@cdot65/prisma-airs-harness`, for Apple
Silicon, Linux x64 and Linux ARM64. It bundles Prisma AIRS CLI **7.2.0** and SDK
**0.34.0**. Inference and remote MCP traffic use AI Gateway; provider credentials
and upstream OAuth stay at the gateway. The command remains `airs`.

## Install

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.4 --registry=https://registry.npmjs.org
airs --version
airs cli --version
```

The same release is available at `https://npm.cdot.io`. Node requires
`^22.13.0 || >=23.5.0`. Keep optional dependencies enabled; they select the native
package for your host. Restart existing sessions after upgrading.

An existing `airs-harness` installation migrates once. Uninstall the old package to
release its command links, then install the renamed package; AIRS environments,
credential bindings and conversation history are preserved:

```sh
npm uninstall -g airs-harness
npm install -g @cdot65/prisma-airs-harness@0.1.4
```

Never use `--force`. The native packages are `@cdot65/prisma-airs-harness-<platform>-<arch>`
and the Apple Silicon signing identifier remains `airs-harness`.

## Changes since 0.1.3

- In-place credential replacement for an environment: `airs env auth [NAME]`, or
  `airs login` on a signed-in environment, offers replacing the workspace key,
  switching between key and SSO, re-signing in, or testing the current
  credential. The CLI form is `airs --environment NAME login --replace`. Every
  replacement is verified with one minimal inference request, retires the old
  native entry, rotates the auth generation and re-pins saved conversations.
- Fullscreen transcript by default, with an inline override in `settings.toml`.
- Apple Silicon binaries are Developer ID signed and Apple notarized again, with
  the same identifier and team as earlier signed releases.
- The npm launcher is renamed to `@cdot65/prisma-airs-harness`; the stable release
  tooling accepts and verifies the by-name migration from `airs-harness`.
- A fresh conversation shows the Prisma AIRS mark in unused terminal rows; a
  click replays its spin. It is hidden while drafting, dismissed by activity, and
  follows `tui.animations` and `tui.whimsy`.
- Documentation: Getting started leads with the welcome flow, real screenshots,
  the `/doctor` screenshot, and an administrator guide to provisioning with the
  bundled CLI.

## Validation record

The native binaries are built from the tagged source; Apple Silicon is Developer
ID signed, hardened and notarized. Native candidate testing covers installation,
OIDC and native credential storage, terminal behavior, MCP management, doctor,
bundled CLI, upgrade/rollback and output integrity on all three platforms. Each
installed regression suite contains 61 tests with platform-specific skips. Extra
checks run a continuous installed process against synthetic HTTPS OIDC and
gateway MCP fixtures.

The upgrade round trip installs stable 0.1.3 as `airs-harness`, replaces it by
name with `@cdot65/prisma-airs-harness` (uninstall, then install, never forced),
resumes a real conversation, then rolls back to the exact previous bytes. The
only configuration change it accepts is the stored native command path moving
between the two launchers' installed package trees; the migration re-points that
path and re-serializes unchanged values.

Fresh anonymous registry acceptance passed on Apple Silicon, Linux x64 and
Linux ARM64 for both npm distributions. Fresh unversioned default installs also
passed on all three platforms against each registry. The configured Ubuntu
preparation helper passed all six isolated keyring/readiness checks **using the
repository copy at the release tag**: the copy bundled inside the npm package
still names the unscoped launcher path in its installed-version and registry
checks and reports `NOT READY` against a correct installation. Follow the Ubuntu
guide's download instruction rather than extracting the helper from the tarball. The
[release evidence](https://github.com/cdot65/prisma-airs-harness/tree/airs-harness-v0.1.4/validation/2026-09-30/stable-0.1.4)
records binary hashes, source identities and platform results.

The original GNU workspace run recorded **19,011 passed, 5 failed and 35
skipped**. It is not a green full-suite result. Two failures compared stale
help-text snapshots of the standalone `codex` binary; a test-only snapshot
correction passed all **1,044** focused CLI tests. Three unchanged upstream
remote-execution failures match the reviewed baseline, and the installed AIRS
command rejects that disabled service. All original results are retained with a
source-bound review.

Candidate acceptance was restarted twice after release-tooling defects specific
to the scoped rename (an interactive-terminal fixture that resolved the native
package by its legacy name, and an upgrade rewrite bound that allowed only the
launcher directory to move), and once more on Ubuntu after the host ran out of
disk during the upgrade round trip. Each restart used corrected tooling against
unchanged native packages; the failed runs are retained with the passing ones.

Earlier the same day these native binaries were published as the unscoped
`prisma-airs-harness@0.1.4` with legacy-named native packages. Those versions
are deprecated with a pointer to the scoped package and are not the supported
0.1.4 distribution.

Owner authorization requested a stable publication. Automated acceptance uses
controlled identity, inference and MCP fixtures; it does not claim a fresh
attended Entra login, production ServiceNow call or paid Jev quality
evaluation. The site distinguishes these checks from your deployment's
[acceptance workflow](https://cdot65.github.io/prisma-airs-harness/validation/acceptance).

Runtime source: `946dc6e34c9bd7f439c5fa00a9c05a11bee53673`.
Native packaging: `5d8bf7a56981132e79d73e13c2dc2777e405add7`.
Acceptance, publication and promotion tooling: `b9b9c14745f0fc65d0841f52dd786e4bdea64b19`.

For these new scoped packages, npm creates an initial `latest` tag during first
publication. Fresh registry and default-install checks verify it; existing
private channels are preserved until explicit promotion.

Public and private distribution payloads share the same native binaries and CLI
bundle. Registry metadata and validation provenance are recorded independently.
Public upgrade tests fetch the previous stable 0.1.3 from its original registry.

The Ubuntu preparation helper contains a sanitized gateway endpoint. Configure
the downloaded copy with your actual listener as shown in the Ubuntu guide before
running readiness checks. An unchanged example hostname is not a working gateway.

## Roll back

```sh
npm uninstall -g @cdot65/prisma-airs-harness
npm install -g airs-harness@0.1.3
```

Close running sessions first. Preserve environments, credential bindings and
conversation history. Existing tags of the previous package name are retained.
