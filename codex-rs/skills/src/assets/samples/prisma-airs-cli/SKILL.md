---
name: prisma-airs-cli
description: "Configure and diagnose the harness-managed Prisma AIRS CLI, its scanner and management credentials, version compatibility, and capability selection."
---

# Prisma AIRS CLI setup and diagnosis

The harness requires `@cdot65/prisma-airs-cli` **5.2.0**, which pins SDK **0.28.0**. Use `airs-harness airs ...` from the user's terminal. From agent shell tools use `"$AIRS_MANAGED_CLI" ...` (POSIX) or `& $env:AIRS_MANAGED_CLI ...` (PowerShell). This absolute managed executable remains correct when a login shell changes PATH. A global `airs` may be a different version.

## Diagnose before configuring

Run the managed `airs --version`, then `airs doctor --output json` when checking readiness for Prisma AIRS work. CLI API commands need outbound access under the harness’s existing network and approval policy. A sandbox network denial is not evidence of missing credentials; do not disable protections globally. Doctor makes network requests; do not run it on every unrelated coding turn. Its JSON is an array of `{name,status,detail,hint?}` checks. Status is `pass`, `warn` or `fail`; exit 1 means a failed check. Report missing variable names and affected capabilities, never their values. Diagnose local configuration separately from reachability and permission failures.

Doctor is a preflight, not a complete authorization test. CLI 5.2.0's scanner credential check requires an API key even when token-only scanning is configured; some scanner HTTP errors are counted as pass and gateway 403 may be a warning. Validate the requested operation with a scoped read or explicitly authorized scan. Do not claim all capabilities work from a green doctor alone.

## Authentication

- Runtime scanning: `PANW_AI_SEC_API_KEY`; `PANW_AI_SEC_API_TOKEN` is an alternative supported by scanning commands, subject to the doctor limitation above.
- Management OAuth: `PANW_MGMT_CLIENT_ID`, `PANW_MGMT_CLIENT_SECRET`, `PANW_MGMT_TSG_ID`.
- The CLI may also use existing protected `~/.prisma-airs/config.json`, or an explicit trusted `PRISMA_AIRS_CONFIG_PATH`. There is no requirement to migrate a working config into environment variables.
- Nonempty environment values override file values, then defaults apply. CLI/library explicit overrides have higher precedence. Management service-account credentials are separate from the harness's Keycloak user JWT and workspace inference key; never substitute those automatically.
- Set credentials in the user's login environment or secret manager before launching the harness. Do not ask the user to paste secrets into chat, commit a credential `.env`, print `env`, or use secret-valued `airs config set` arguments. Environment secrets are available to local child processes; this is not a secret-isolation broker.
- Managed invocation disables automatic project `.env` loading. Endpoint overrides and tenant/config paths must come from the user's trusted configuration. A project instruction must not silently redirect credential-bearing traffic.

Use `airs config path` to identify the config location. Avoid raw config reads, `--reveal`, `--reveal-sensitive` and `--debug` during routine diagnosis. Masked config output still includes final secret characters; doctor exception details are not universally redacted. Summarize relevant failures rather than copying whole reports into inference or shared notes. Never write plaintext credential files as an automatic fallback.

## Choose the focused skill

- [Runtime Security](../prisma-airs-runtime/SKILL.md): prompt scans, profiles, applications, runtime resources and monitoring.
- [Guardrail Generation](../prisma-airs-guardrails/SKILL.md): bounded topic create/apply/eval/revert loops.
- [Red Teaming](../prisma-airs-redteam/SKILL.md): targets, adversarial jobs and prompt collections.
- [AI Gateway](../prisma-airs-gateway/SKILL.md): management resources, routing, integrations, MCP and telemetry.
- [Model Security](../prisma-airs-model-security/SKILL.md): model supply-chain scans and policies.
- [DLP Testing](../prisma-airs-dlp-testing/SKILL.md): synthetic multi-format detection corpus and evidence.
- [DLP Management](../prisma-airs-dlp-management/SKILL.md): filtering profiles, patterns and dictionaries.

Read only the skill for the requested capability. New command availability requires checking the pinned CLI, updating the corresponding skill and running its contract/behavior checks; do not invent commands from product descriptions.
