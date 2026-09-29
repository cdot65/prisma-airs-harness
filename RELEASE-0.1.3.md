# Prisma AIRS Harness 0.1.3

Stable 0.1.3 promotes the current alpha for Apple Silicon, Linux x64 and Linux
ARM64. It bundles Prisma AIRS CLI **7.2.0** and SDK **0.34.0**. Inference and remote
MCP traffic use AI Gateway; provider credentials and upstream OAuth stay at the
gateway.

## Install

```sh
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
airs --version
airs cli --version
```

The same release is available at `https://npm.cdot.io`. Node requires
`^22.13.0 || >=23.5.0`. Keep optional dependencies enabled; they select the native
package for your host. Restart existing sessions after upgrading. No uninstall,
forced executable replacement or deletion of credential stores is required.

## Changes since 0.1.2

- Conversation-scoped saved-config and model routing, with gateway policy
  controlling permitted overrides.
- MCP onboarding and recovery that distinguish saved configuration, sign-in,
  discovery and tool authorization. The gateway owns upstream OAuth.
- Terminal transcript/copy improvements, draft recovery, opt-in fullscreen and
  search, and clearer diagnostic and retry behavior.
- Gateway administration skills and CLI 7.2.0, including separate organization
  `admin-guardrails` and workspace `guardrails` commands.
- Standalone documentation for deployment, Keycloak/Entra OIDC, workspace keys,
  validation and hands-on operations, with the supplied Harness shield and
  cyan/navy visual theme.

## Authentication and setup

Follow [Getting started](https://cdot65.github.io/prisma-airs-harness/getting-started),
then the [Keycloak guide](https://cdot65.github.io/prisma-airs-harness/configuration/keycloak)
and optional [Entra federation](https://cdot65.github.io/prisma-airs-harness/configuration/entra).
The public examples sanitize the reference realm, stacks, users and groups.
Inference credentials and gateway MCP grants remain separate. Workspace API keys
require a gateway policy that supports opaque credentials.

The [command cheat sheet](https://cdot65.github.io/prisma-airs-harness/operations/cheat-sheet)
includes local enrollment, gateway workspace/integration provisioning, diagnostics,
updates and rollback. Management credentials belong to the bundled CLI tenant,
not the inference environment.

## Validation record

The native runtime differs from alpha.7 only in its version stamp; licensing
sidecars do not change runtime behavior. Apple Silicon is Developer ID signed,
hardened and notarized. Native candidate testing covers installation, OIDC and
native credential storage, terminal behavior, MCP management, doctor, bundled CLI,
upgrade/rollback and output integrity on all three platforms. Each installed
regression suite contains 60 tests with platform-specific skips. Extra checks
exercise credential renewal and actual-agent TypeSafe and gateway skill calls.

Fresh anonymous registry acceptance passed on Apple Silicon, Linux x64 and
Linux ARM64 for both npm distributions. Fresh unversioned default installs also
passed on all three platforms against each registry. The configured Ubuntu preparation helper
passed all six isolated keyring/readiness checks. The
[release evidence](https://github.com/cdot65/prisma-airs-harness/tree/airs-harness-v0.1.3/validation/2026-09-29/stable-0.1.3)
records binary hashes, source identities and platform results.

The original GNU workspace run recorded **18,992 passed, 6 failed and 35 skipped**.
It is not a green full-suite result. Three failures compared license sidecars
against generated schemas; a test-only correction passed all **302** focused
protocol tests. Three unchanged upstream remote-execution failures match the
reviewed baseline, and the installed AIRS command rejects that disabled service.
All original results are retained with a source-bound review.

An intermittent terminal-question fixture timed out in an initial Mac run and
one Ubuntu public-candidate run. Each received three passing targeted reruns;
the failed results are retained alongside the complete reruns. Do not interpret
retries as proof that the fixture is free of timing sensitivity.

Owner authorization requested publication of the current alpha. Automated
acceptance uses controlled identity, inference and MCP fixtures; it does not
claim a fresh attended Entra login, production ServiceNow call or paid Jev quality
evaluation. The site distinguishes these checks from your deployment's
[acceptance workflow](https://cdot65.github.io/prisma-airs-harness/validation/acceptance).

Runtime source: `900792f8707c8f2a47d9814575469ecc4817be75`.
Native packaging: `17d09f537833305bab8fd9cacc840cfd9e18c598`.
Public acceptance and promotion tooling: `b0016119da0f3acfa141436571bf43205307005d`.
Publication transport: `06e1c0ae09d07b762328fa7bf049e615943bd778`.

For these new public packages, npm creates an initial `latest` tag during first
publication. Fresh registry and default-install checks verify it; existing private
channels are preserved until explicit promotion.

Public and private distribution payloads share the same native binaries and CLI
bundle. Registry metadata and validation provenance are recorded independently.
Public upgrade tests fetch the previous stable 0.1.2 from its original registry;
0.1.2 was not republished to public npm.

The Ubuntu preparation helper contains a sanitized gateway endpoint. Configure
the downloaded copy with your actual listener as shown in the Ubuntu guide before
running readiness checks. An unchanged example hostname is not a working gateway.

## Roll back

```sh
npm install -g airs-harness@0.1.2 --registry=https://npm.cdot.io
```

Close running sessions first. Preserve environments, credential bindings and
conversation history. The `mac-preview` and other pre-existing tags are retained.
