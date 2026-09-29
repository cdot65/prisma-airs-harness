---
title: Releases and installation channels
---

The npm package is `airs-harness`; the command is `airs`. Packages are distributed
through `https://npm.cdot.io`, which may require your organization's LAN or VPN.
The published release records establish these versions:

| Channel | Version | Platforms | Bundled CLI |
| --- | --- | --- | --- |
| Stable / `latest` | 0.1.2 | Linux x64, Linux ARM64, Apple Silicon | 7.1.5 |
| `mac-preview` | 0.1.3-alpha.7.mcp.1 | Apple Silicon | 7.2.0 |

For reproducible installation, pin the version:

```sh
# Stable
npm install -g airs-harness@0.1.2 --registry=https://npm.cdot.io
# Apple Silicon preview, when explicitly selected
npm install -g airs-harness@0.1.3-alpha.7.mcp.1 --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Use one installation command for your intended channel. Restart AIRS after
upgrading. To return to stable, install `airs-harness@0.1.2` again. Ordinary
upgrades and tested rollback preserve environments, credential bindings and
history; no uninstall or forced executable replacement is required.

The preview includes conversation config/model routing, MCP recovery, terminal
polish, opt-in fullscreen/search and updated Gateway administration guidance.
Linux preview distributions remain deferred in the alpha.7 release record.

The [stable receipt](../generated/stable.md), [alpha.7 receipt](../generated/preview.md)
and [terminal preview receipt](../generated/terminal-preview.md) distinguish
source tests, exact installed package checks and owner acceptance. Fixture tests
do not establish a fresh human SSO login, real ServiceNow call or paid Jev
evaluation. Recorded GNU runs include inherited failures and are not wholly green.

The generated command reference follows the repository revision used to build
this site. Always check local `--help` and `--version` when using a different
release. Publishing these docs does not publish a new binary or move an npm tag.
