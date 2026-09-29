---
title: Releases and installation channels
---

Install the npm package `airs-harness`; run the command `airs`. Stable **0.1.3**
is distributed through public npm and `https://npm.cdot.io` with CLI **7.2.0** and
SDK **0.34.0** bundled. The private registry may require your organization's
network or VPN.

| Channel | Version | Platforms | Bundled CLI |
| --- | --- | --- | --- |
| Stable / `latest`, both registries | 0.1.3 | Linux x64, Linux ARM64, Apple Silicon | 7.2.0 |
| `mac-preview`, private registry | 0.1.3-alpha.7.mcp.1 | Apple Silicon | 7.2.0 |
| Previous stable, private registry | 0.1.2 | Linux x64, Linux ARM64, Apple Silicon | 7.1.5 |

For a reproducible installation:

```sh
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
airs --version
airs cli --version
```

Use `--registry=https://npm.cdot.io` when selecting the original distribution.
Normal npm installs include the optional native package for the host. Supported
platforms are Apple Silicon, Linux x64 and native Linux ARM64; there is no Windows
or Intel Mac release. Node requires `^22.13.0 || >=23.5.0`.

Restart the harness after upgrading. Ordinary upgrades and tested rollback
preserve environments, credential bindings and conversation history; no uninstall
or forced executable replacement is required. To return to the previous stable:

```sh
npm install -g airs-harness@0.1.2 --registry=https://npm.cdot.io
```

Version 0.1.2 is only available from the original registry. Public npm acceptance
explicitly tests against that baseline rather than assuming it exists on both
registries.

Stable 0.1.3 includes conversation config/model routing, MCP recovery, terminal
polish, opt-in fullscreen/search and updated Gateway administration guidance.
See the [0.1.3 release record](../generated/stable-013.md) for exact runtime and
tooling provenance, platform results and known fixture limitations. The
[0.1.2 record](../generated/stable.md), [alpha.7 record](../generated/preview.md)
and [terminal preview record](../generated/terminal-preview.md) remain available
as historical evidence.

Automated identity and provider fixtures do not establish a fresh attended Entra
login, real ServiceNow call or paid Jev evaluation. The full GNU run retains its
reviewed failures; it is not represented as wholly green. Run your own
[deployment acceptance](../validation/acceptance.md) with the intended user and
permissions before production rollout.

The generated command reference follows the repository revision used to build
this site. Check local `--help` and `--version` when using another release.
