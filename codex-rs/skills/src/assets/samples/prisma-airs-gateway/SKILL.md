---
name: prisma-airs-gateway
description: "Configure Prisma AIRS AI Gateway workspaces, routing, guardrails, integrations, MCP and telemetry using schema-checked management commands."
---

# AI Gateway

Use the harness-managed CLI 7.1.1. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples use `airs cli` from the user terminal; agent shell tools must use the absolute managed path. Do not substitute a global installation. Verify the managed version is 7.1.1. Check the selected product tenant before operations; harness environment selection does not select a CLI tenant.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Use `airs cli aigateway --help` and the exact resource help. CLI 7.1.1 includes `workspaces`, `configs`, `guardrails`, providers/integrations, MCP resources, deployment resources, telemetry and inference. Use supported pagination flags for complete inventories and disclose any limit or unavailable page.

Follow copy-and-adjust workflows: read the intended workspace/object, capture its current configuration, build a narrowly changed candidate using the documented JSON/schema or `--set` fields, inspect the diff, then apply the already-authorized change and read it back. Preserve unrelated integrations, policies, audiences and routing entries. Redact API keys, provider credentials and tokens from saved evidence.

Separate management OAuth credentials from inference credentials and from the harness's Keycloak JWT. Validate the selected workspace and permission scope. For this harness, the default route omits `model`; an explicit authorized route is `@provider/model`. Do not silently insert a concrete model into default-route requests.

When testing a changed gateway, verify the actual request path/authentication, response or stream, remote-tool behavior if relevant, and policy decision. A successful configuration write or green doctor does not establish inference compatibility. Keep the previous configuration and a concrete supported rollback available. Preserve the target deployment’s existing realm, issuer and JWKS configuration unless the task authorizes changing it; verify that topology from trusted deployment configuration.
