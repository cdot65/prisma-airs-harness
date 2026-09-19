# Prisma AIRS Harness 0.1.1

This release brings the tested onboarding and MCP workflows to the default
installation. Install the `airs-harness` package and run `airs`; no separate
Prisma AIRS product CLI installation is needed.

## What is included

- Guided local environments with independent credentials and gateway settings.
  Environment names do not need to match AI Gateway workspace names.
- Company SSO, including inference device sign-in for browserless hosts, or a
  workspace API key for inference. Workspace API keys do not replace MCP OAuth.
- `/mcp` connection creation, sign-in, reconnection, sign-out and removal inside
  the harness. Desktop browsers open automatically; remote users can paste a
  callback privately. Authorization, native credential storage and tool discovery
  show separate progress.
- `/doctor` for diagnostics and explicit gateway verification, with actionable
  credential-store and authorization recovery messages.
- Prisma AIRS CLI 7.0.0 and eight product skills bundled as `airs cli ...`.

## Install or upgrade

Use Node `^22.13.0 || >=23.5.0` and your organization's npm registry:

```sh
npm install -g airs-harness@0.1.1 --registry=https://npm.cdot.io
airs --version
airs cli --version
airs
```

Unversioned installs select the release promoted through `latest`. Optional dependencies are included by npm by default; `--include=optional`
is only needed if your npm configuration omits them. Restart existing AIRS
processes after upgrading. Environments, credential bindings and conversation
history are preserved; existing MCP storage choices are not silently migrated.

Supported packages: Linux x64, Linux ARM64 and signed/notarized Apple Silicon.
Windows and Intel Mac packages are not part of this release. Linux requires a
working native Secret Service and Bubblewrap setup; see [Ubuntu preparation](UBUNTU-TEST-HOST.md).

## Authentication boundaries

Inference and remote MCP connect through Prisma AIRS AI Gateway. MCP sign-in
receives its own OAuth grant even if the browser reuses the same company SSO
session. The gateway handles upstream ServiceNow authorization. Native device
sign-in for MCP remains dependent on gateway support; remote callback entry is
available today. No new authentication proxy service is introduced.

Follow [Getting started](GETTING-STARTED.md) for a complete SSO or workspace-key
inference setup followed by MCP sign-in and a ServiceNow read.

## Verification

The owner confirmed all four real-account Apple Silicon mcp.6 checks: inference
sign-in, MCP sign-in, a read-only ServiceNow query, and credential reuse after
restart. Stable changes preserve that runtime behavior. Version stamping, release
tooling, test isolation and documentation are verified separately. Release
evidence records exact-package, full-workspace and default-install results.
A prior Alpine workspace failure count is not presented as a pass.
