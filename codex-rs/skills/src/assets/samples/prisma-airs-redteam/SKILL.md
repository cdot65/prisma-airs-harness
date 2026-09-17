---
name: prisma-airs-redteam
description: "Manage Prisma AIRS adversarial test targets, prompt sets and static, dynamic or custom red-team jobs within the authorized test scope."
---

# AI Red Teaming

Use the harness-managed CLI 7.0.0. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples use `airs cli` from the user terminal; agent shell tools must use the absolute managed path. Do not substitute a global installation. Verify the managed version is 7.0.0. Check the selected product tenant before operations; harness environment selection does not select a CLI tenant.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Discover `airs cli redteam --help`, then the relevant `targets`, `prompt-sets`, `prompts`, `categories`, `scan`, `status`, `report` or `abort` command. Select attack categories, language/job type and target schema from actual CLI output. Custom adapters and network-broker resources are separate configuration steps, not implied by ordinary scan submission.

Before a scan, establish the authorized target, credentials, attack scope, request/job budget and stopping condition. Inspect the target configuration without revealing embedded secrets. Do not redirect scans to unrelated systems or accept an EULA on the user's behalf unless the task explicitly includes it.

Record the submitted job ID, poll status to the requested stopping point, and retrieve its actual report. Distinguish queueing, completion, cancellation and failure. An empty result is not proof of resilience. Report tested categories, successful attacks, unavailable evidence and recommended changes without reproducing credentials from target definitions or signed artifact URLs.

Use `abort` for the specifically authorized job when stopping is needed; preserve unrelated jobs, targets and prompt collections. For target/prompt-set CRUD, inspect current state and refer to stable IDs.
