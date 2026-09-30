# Security policy

## Reporting a vulnerability

Report security vulnerabilities in Prisma AIRS Harness privately, through
[GitHub private vulnerability reporting](https://github.com/cdot65/prisma-airs-harness/security/advisories/new).
Do not open a public issue or pull request on Forgejo or GitHub, and do not post
details in discussions.

Include what you can of the following:

- the affected version (`airs --version`) and platform;
- the steps to reproduce, and what an attacker gains;
- whether the issue needs a malicious repository, a malicious MCP server, a
  compromised gateway, or local access to the user's machine.

Never include working credentials, access tokens, authorization callbacks or
customer data in a report. If a report needs a sample, use a revoked credential
or a redacted one.

## Supported versions

Security fixes are made for the latest stable release of
`@cdot65/prisma-airs-harness` on npm. The earlier package name `airs-harness` is
frozen at 0.1.3 for rollback and does not receive fixes; upgrade to the scoped
package.

## Scope

In scope is the harness itself: the `airs` command, its native packages, its
handling of credentials, environments, sandboxing and approvals, and its
connections to Prisma AIRS AI Gateway and MCP servers.

Report these elsewhere:

- **Prisma AIRS, AI Gateway and other Palo Alto Networks products:** to
  Palo Alto Networks through its product security process at
  <https://security.paloaltonetworks.com>.
- **Behavior inherited unchanged from OpenAI Codex:** to OpenAI, following the
  [Codex security policy](https://github.com/openai/codex/security/policy). If you
  are unsure whether the harness changed the affected code, report it here.

## How the harness is meant to be secured

The [architecture guide](https://cdot65.github.io/prisma-airs-harness/guides/architecture/)
describes what runs locally, what runs at the gateway, and which credential covers
each path. Knowing those boundaries helps decide whether an issue is in the
harness, in the gateway, or in a configuration choice.
