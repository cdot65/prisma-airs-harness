# Prisma AIRS Harness 0.1.2

Published at `https://npm.cdot.io` under `latest`, bundling CLI **7.1.5**.
Exact candidates, fresh anonymous registry installs and unversioned default
installs passed on Linux x64, native Linux ARM64 and Apple Silicon.

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

## Installation

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

The owner confirmed alpha.5 inference, ServiceNow through `/mcp`, a read-only
query, restart/reuse and the Jev workflow. The stable native runtime changes only
its version stamp. The Ubuntu preparation helper now defaults to 0.1.2 and passed
all six isolated keyring/readiness checks against the published package without
a version override.

Both alpha.5 and stable 0.1.1 upgrade/rollback paths passed with preserved state.
Apple Silicon is Developer ID signed, hardened and notarized. Automated Jev tests
exercise the actual agent approval UI, saved keys, inherited-key precedence and
denied approval with no provider request. Their provider and inference responses
are fixtures; they do not establish Jev accuracy or calibrated ASR.

The full GNU workspace run recorded **18,374 passed, 3 failed and 34 skipped**.
It is not a green full-suite result. All three failures exercise unchanged
upstream remote-execution code; the installed AIRS command rejects that disabled
service. The original failures and exact-source release review are retained in
[release evidence](validation/2026-09-21/stable-0.1.2/README.md).

Native source and frozen validation tooling: `aee13a272b5e79fe0ea6fe62485ca7c164927028`.
Packaging and installation-guide alignment: `8a1dd5324c4c6c2345b8d8737e0400308fe67cab`.
Promotion policy: `9402e7ec7543a44210883950c5f338903472fd43`.
An initial package set with stale guide versions was never published; the corrected
set received fresh acceptance. See `PACKAGING-CORRECTION.json` in the evidence.

Follow [Getting started](GETTING-STARTED.md) and the
[judge guide](https://cdot65.github.io/prisma-airs-cli/cli/redteam/judge/).
