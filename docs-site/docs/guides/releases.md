---
title: Releases and installation channels
---

This page has two parts. The first explains how releases are organized and what a
release record does and does not tell you. The second is the procedure: install,
check what you have, upgrade and roll back. It names no specific version, so it
stays correct as releases change. The exact versions are in the release records
and in your own `--version` output.

## How releases are organized

Install the npm package `prisma-airs-harness` (named `airs-harness` before 0.1.4); run the command `airs`. Each release
bundles a Prisma AIRS CLI and SDK, so a separate product CLI installation is not
needed. `airs cli --version` shows the bundled CLI.

**Channels.** Versions are published under npm dist-tags:

| Channel | What it is | Platforms |
| --- | --- | --- |
| Stable (`latest`) | The current supported release | Linux x64, Linux ARM64, Apple Silicon |
| Preview (`mac-preview`) | Early access to changes before they reach stable | Apple Silicon |
| Previous stable | The release before the current one, kept for rollback | Linux x64, Linux ARM64, Apple Silicon |

Which channels you can reach depends on the registry you install from. A registry
run by your organization may need its network or VPN, and it may carry versions
that the public registry does not, including the previous stable. Public
acceptance tests against that baseline explicitly instead of assuming it exists
everywhere.

**Platforms.** Normal npm installs include the optional native package for the
host. Supported platforms are Apple Silicon, Linux x64 and native Linux ARM64.
There is no Windows or Intel Mac release. Node requires `^22.13.0 || >=23.5.0`.

**What a release record covers.** Each release has a record with its exact runtime
and tooling provenance, platform results and known fixture limitations. Read the
[current release record](../generated/stable-014.md) for what the stable release
includes and how it was tested. The
[previous stable records](../generated/stable-013.md) and [0.1.2](../generated/stable.md),
[Apple Silicon preview record](../generated/preview.md) and
[terminal preview record](../generated/terminal-preview.md) remain available as
historical evidence.

**What the evidence does not show.** Automated identity and provider fixtures do
not establish a fresh attended Entra login, a real ServiceNow call or a paid Jev
evaluation. The full GNU run retains its reviewed failures; it is not represented
as wholly green. So a passing release record is a starting point, and your own
[deployment acceptance](../validation/acceptance.md) with the intended user and
permissions comes before production rollout.

**Upgrades keep your state.** Ordinary upgrades and tested rollback preserve
environments, credential bindings and conversation history. No uninstall or forced
executable replacement is required.

## Install, upgrade and roll back

### Install and check

```sh
npm install -g prisma-airs-harness
airs --version
airs cli --version
```

To install a specific version for a reproducible setup, name it:
`npm install -g prisma-airs-harness@<version>`. If your organization publishes through its
own registry, add `--registry=<registry URL>`. Check which versions and tags a
registry offers with `npm view prisma-airs-harness dist-tags`.

### Upgrade

Install the version you want, then restart the harness. Running sessions keep the
old version until they restart.

### Roll back

Close running sessions first. Then install the previous version by number:

```sh
npm install -g prisma-airs-harness@<previous-version>
```

Versions before 0.1.4 were published as `airs-harness`: uninstall
`prisma-airs-harness` first, then `npm install -g airs-harness@<previous-version>`.
Use the same registry the previous version was published to. If it is not found,
the registry you chose may not carry it.

### Check the command reference

The generated command reference follows the repository revision used to build
this site. Check local `--help` and `--version` when using another release.
