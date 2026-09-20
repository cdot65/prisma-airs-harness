---
name: prisma-airs-runtime
description: "Scan prompts and manage Prisma AIRS runtime profiles, API keys, customer applications, topics and runtime monitoring with the managed CLI."
---

# Runtime Security

Use the harness-managed CLI 7.1.2. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples use `airs cli` from the user terminal; agent shell tools must use the absolute managed path. Do not substitute a global installation. Verify the managed version is 7.1.2. Check the selected product tenant before operations; harness environment selection does not select a CLI tenant.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Discover with `airs cli runtime --help`. Use `scan` for one prompt, `bulk-scan` for prompt datasets and `resume-poll` for an existing asynchronous state file. Select the actual security profile and endpoint from the requested environment. A submitted job is not a completed scan: poll to the requested terminal state and report actual action, scan/report ID and evidence gaps.

CLI 7.0.0 can map normalized scan failures back to `allow`. An Allow summary alone is insufficient: check complete scan/report evidence for errors, timeout and successful completion before declaring the prompt safe. If the command omits that evidence, report the result as unverified rather than a successful scan.

Runtime management includes `profiles`, `topics`, `api-keys`, `customer-apps`, `deployment-profiles` and `dlp`. Inspect the current object and accepted schema before changing it. Preserve unrelated settings; keep generated API keys local and out of model-visible reports. Profile names and IDs are not interchangeable across tenants.

Use current `sessions`, `dashboard` or `report` commands for monitoring after checking their help. `scan-logs` is marked broken legacy retrieval; use the verified `runtime sessions` command and do not invent a replacement query. HTML/Markdown reports are written to the working directory: select an explicit output path in the task workspace, then inspect the report's stated evidence gaps.

For iterative topic refinement use the Guardrail Generation skill. For synthetic document/image generation use DLP Testing; `bulk-scan` accepts prompt text/CSV, not arbitrary PDF/image uploads.
