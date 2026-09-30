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

Pending. This section is completed with the platform results, the readiness run
and the registry acceptance before publication; the release is not ready until
then.

## Roll back

```sh
npm uninstall -g @cdot65/prisma-airs-harness
npm install -g airs-harness@0.1.3
```

Close running sessions first. Preserve environments, credential bindings and
conversation history. Existing tags of the previous package name are retained.
