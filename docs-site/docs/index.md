---
title: Prisma AIRS Harness
slug: /overview
---

A local terminal agent for coding and Prisma AIRS operations. Work with files,
run project tools, use the bundled product CLI, and connect remote tools through
Prisma AIRS AI Gateway.

```sh
npm install -g airs-harness@0.1.3 --registry=https://registry.npmjs.org
airs env create work
airs
```

Start with [installation and sign-in](generated/getting-started.md), including
Node requirements, native credential storage, workspace API keys and company SSO.
Your administrator provides the gateway connection details and access grants.

| Task | Guide |
| --- | --- |
| Connect to a gateway and sign in | [Getting started](generated/getting-started.md) |
| Separate accounts, destinations and history | [Environments](guides/environments.md) |
| Authorize ServiceNow and other remote tools | [Gateway MCP](guides/mcp.md) |
| Resume work, select routes and configure the terminal | [Terminal workflow](guides/terminal.md) |
| Run product administration commands | [Bundled CLI and skills](generated/bundled-cli.md) |
| Score exported red-team results | [TypeSafe Jev judge](guides/judge.md) |
| Find flags and command usage | [CLI reference](generated/reference/airs.md) |
| Build and publish from source | [Development](guides/development.md) and [deployment](guides/deployment.md) |

**Release 0.1.3** brings the current gateway routing, terminal and diagnostic
work into the stable distribution, with CLI 7.2.0 and SDK 0.34.0. Supported
platforms are Linux x64, Linux ARM64 and Apple Silicon. Windows and Intel Mac
distributions are not available. See [release channels](guides/releases.md).

Inference and remote MCP both go through AI Gateway. Files, shell execution,
approvals and history live in the local runtime; selected file contents and tool
results can become model context. The gateway controls model access, policy and
upstream MCP authorization. [Read the architecture](guides/architecture.md).

This is the harness product documentation. The separate
[reference architecture curriculum](https://cdot65.github.io/prisma-airs-reference-architecture/)
explains the wider identity and infrastructure system.
