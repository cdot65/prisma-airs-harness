---
id: start-here
title: "Start here"
sidebar_label: "Start here"
---

For a practical first session, start with [Sign in with SSO and connect ServiceNow](./login.md). It walks through environment creation, company sign-in for inference, adding the gateway ServiceNow MCP connection, a second authorization with the same SSO identity, and a read-only incident query. It starts with one harness installation and explains the `airs` / `airs cli` commands, migration from the old standalone CLI, and optional product tenant setup.

## The goal

This course follows one authenticated AI assistant from a question to a real utility result. The harness sends its model requests through an approved inference route, and it calls a small, authorized set of MCP tools through the same AI Gateway. The interesting part is how many separate services have to agree before a simple calculation is allowed to happen, and what each of them actually checks.

Our running example is a platform engineer, Alex, asking:

> Use mcp server 1 to multiply 12 by 7, then give me the server's current UTC time.

If you have built against HTTP APIs, your first instinct is probably one request with one credential in a header. That is where this system diverges from ordinary API work. The Prisma AIRS Harness runs the conversation and its tools on Alex's computer. Keycloak authenticates Alex and issues access tokens. Prisma AIRS AI Gateway receives both the inference traffic and the MCP traffic, invokes the configured inference checks, and proxies the upstream MCP service. mcp server 1 performs eight utility operations locally on its own host, which is separate from Alex's workstation, and its tool execution needs no downstream management API. CIE Directory Sync and CAS support the gateway-facing MCP user and workspace authorization. Each component makes a different decision with a different credential, and none of them can make the others' decisions for them.

## What this course teaches

By the end, you should be able to trace one question from login through inference and tool execution, explain the two credentials the harness holds (one for inference and one for gateway MCP access) and the separate upstream tokens the gateway holds on the user's behalf, locate the service that makes each authorization decision, distinguish SCIM provisioning from SAML authentication, and investigate a failed request without exposing credentials.

You need basic familiarity with HTTP, JSON, command-line applications, and containers. OAuth, OIDC, SAML, SCIM, and MCP are introduced where they enter the story rather than up front, because each is easier to hold once you can see which message it carries and who receives it. The first labs use fictional records and need no tenant or production credentials. An optional integration lab requires an isolated, licensed environment and a maintainer-provided configuration.

## Read in this order

| Stage | Lesson | What you should be able to explain |
| --- | --- | --- |
| Orient | [System architecture](./architecture.md) | Components, boundaries, and two request paths |
| Understand the client | [Harness and agent execution](./harness.md) | Who calls the model and who executes tools |
| Understand identity | [Keycloak and token contracts](./keycloak.md) | Issuer, subject, client, audience, scopes, roles |
| Understand identity integration | [Cloud Identity Engine and provisioning](./cie.md) | Directory Sync, CAS, realm boundaries |
| Trace login | [Login from browser to authorized tools](./login.md) | Discovery, PKCE, callback, credential persistence |
| Trace inference | [AI Gateway and AIRS enforcement](./gateway.md) | Workspace routing and input/output checks |
| Trace tools | [MCP utility tools and authorization](./mcp.md) | Utility grants, input validation and local execution |
| Put it together | [End-to-end question walkthrough](./walkthrough.md) | One complete model/tool/model cycle |
| Maintain identity | [Refresh revocation and identity changes](./lifecycle.md) | Rotation, logout, delayed revocation |
| Operate | [Deployment and operations](./operations.md) | Independent delivery, health and recovery |
| Investigate | [Troubleshooting by trust boundary](./troubleshooting.md) | Evidence-driven diagnosis |
| Practice | [Labs and answer keys](./labs.md) | Demonstrated understanding |
| Review | [Glossary and misconceptions](./glossary.md) | Precise vocabulary |

The component lessons give you the pieces, the interaction lessons show them exchanging messages over time, and the operations lessons ask you to find the one piece that failed. Allow roughly three hours for the reading path and another two for the tabletop labs. These are editorial estimates, not measured learner completion times.

## How to read the evidence labels

A course about a real system has to say which claims come from where, so the lessons label them. **Implemented** describes the reviewed source. **Observed deployment** describes a dated live check against a running environment. **Release acceptance** applies only to the exact installed package and its recorded lifecycle tests, because a behavior in source and a behavior in the binary a user actually downloaded are two different facts. The utility server is deployed. The current command migration has source and installed-package checks. Package publication and timed lifecycle acceptance are separate observations, recorded in [Implementation status and public sources](./evidence.md).

All example domains, subjects, workspaces, and user records in these lessons are fictional. Example identifiers illustrate the contract; they are not runnable configuration or signed credentials. Read [Implementation status and public sources](./evidence.md) for dated findings, limits, and primary references.

The most useful habit throughout the course is to ask two questions at every arrow in every diagram: **what data crosses this boundary, and what authorizes the receiver to act?** Most of the failures in the troubleshooting lesson come from one of those questions having a different answer than someone assumed.
