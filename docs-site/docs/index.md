---
title: Prisma AIRS Harness
slug: /overview
---

A local terminal agent for coding and Prisma AIRS operations. It works with your
files, runs your project tools, includes the bundled product CLI, and reaches
remote tools through Prisma AIRS AI Gateway.

```sh
npm install -g prisma-airs-harness
airs env create work
airs
```

These docs are split in two, because the questions are different. One set
explains how the harness works: what runs where, which credential does what, and
why a request is allowed or denied. The other set is for operating it: the steps,
commands and checks you run to get something done. Start in whichever matches
what you are trying to do.

## Operate the harness

If you have a task in front of you, start with [installation and
sign-in](generated/getting-started.md). The gateway connection details and
access grants come from your administrator, so you'll need those before you can
sign in.

| Task | Guide |
| --- | --- |
| Install, sign in with a workspace key and send a first request | [Getting started](generated/getting-started.md) |
| Sign in with company SSO and connect ServiceNow | [SSO and ServiceNow walkthrough](generated/sso-servicenow.md) |
| Separate accounts, destinations and history | [Environments](guides/environments.md) |
| Authorize ServiceNow and other remote tools | [Gateway MCP](guides/mcp.md) |
| Resume work, select routes and configure the terminal | [Terminal workflow](guides/terminal.md) |
| Provision a workspace with the bundled CLI (administrators) | [Provision a workspace](generated/bundled-cli.md) |
| Score exported red-team results | [TypeSafe Jev judge](guides/judge.md) |
| Look up a command or flag | [Command cheat sheet](operations/cheat-sheet.md) and [CLI reference](generated/reference/airs.md) |
| Check that a deployment works end to end | [Acceptance validation](validation/acceptance.md) |
| Build and publish from source | [Development](guides/development.md) |
| Install, upgrade or roll back | [Release channels](guides/releases.md) |

Administrators setting up the gateway side will also want the configuration
guides for [workspaces and API keys](configuration/gateway.md),
[Keycloak](configuration/keycloak.md), [Entra](configuration/entra.md) and
[MCP authorization](configuration/mcp.md).

## Understand how it works

If you want the model before the steps, or something is behaving in a way you
can't explain, start here. The main thing to know is where the line sits between
the local runtime and the gateway.

Inference and remote MCP both go through AI Gateway. Files, shell execution,
approvals and history stay in the local runtime. Selected file contents and tool
results can become model context, though, so something read locally can end up
in a request that goes to the gateway. The gateway, in turn, controls model
access, policy and upstream MCP authorization.

| Question | Page |
| --- | --- |
| What runs locally, what runs at the gateway, and which credential covers each path? | [Architecture](guides/architecture.md) |
| How do a skill and a tool call move through the agent loop? | [Skills and the local agent loop](guides/skills.md) |
| What has to be deployed around the harness, and in what order? | [Deploy the gateway platform](platform/deployment.md) |

Supported platforms are Linux x64, Linux ARM64 and Apple Silicon. There are no
Windows or Intel Mac distributions. Channels and how to install or roll back
are on the [release channels](guides/releases.md) page, and each release's
contents and test results are in its release record.

This is the harness product documentation. The separate
[reference architecture curriculum](https://cdot65.github.io/prisma-airs-reference-architecture/)
explains the wider identity and infrastructure system.
