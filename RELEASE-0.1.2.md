# Prisma AIRS Harness 0.1.2

Stable candidate for the owner-confirmed alpha.5 TypeSafe judge workflow.
Publication and default-channel promotion are pending release verification.

## Changes since 0.1.1

- Bundled Prisma AIRS CLI 7.1.5 and the native `prisma-airs-asr-judge` skill.
  The skill runs through Node and the TypeScript CLI using the official TypeSafe
  SDK; no Python installation is required by the product workflow.
- Optional `/typesafe` hidden API-key entry, status and removal inside AIRS.
  Credentials belong to the selected harness environment.
- Live judging requests per-command approval for native credential-store and
  TypeSafe network access. The key remains out of model context and command
  arguments. Dry-run and explicit replay retain normal sandbox permissions.
- Prompt envelopes are normalized separately; model output strings are preserved
  verbatim. Fresh judging never silently falls back to recorded judgments.
- Installation repair, redacted diagnostics and shell recovery guidance from
  the previously validated 0.1.2 previews.

## Installation after publication

```sh
npm install -g airs-harness@0.1.2 --registry=https://npm.cdot.io
airs --version
airs cli --version
airs
```

Restart AIRS after upgrading. Existing environments, credential bindings and
conversations are preserved. Optional dependencies are included by normal npm
installs. Supported packages are Linux x64, Linux ARM64 and signed/notarized Apple
Silicon, with Node `^22.13.0 || >=23.5.0`.

## Jev workflow

Inside AIRS, use `/typesafe` to configure a key if this environment does not
already have one. Invoke `$prisma-airs-asr-judge attacks.json` and approve the
specific live judge command. Saving a key or a successful doctor probe does not
grant native-store access to an ordinary sandboxed shell.

The owner confirmed the alpha.5 live workflow works on September 21. Exact stable
candidate, registry, workspace and default-install results will be recorded here
before completion. Automated provider fixtures establish execution/credential
behavior, not model accuracy or calibrated ASR. Follow [Getting started](GETTING-STARTED.md)
and the [judge guide](https://cdot65.github.io/prisma-airs-cli/cli/redteam/judge/).
