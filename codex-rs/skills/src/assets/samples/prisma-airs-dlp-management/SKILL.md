---
name: prisma-airs-dlp-management
description: "Manage Prisma AIRS DLP filtering profiles, patterns, data profiles and dictionaries while preserving supported API boundaries."
---

# DLP Management

Use the harness-managed CLI 7.1.2. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples use `airs cli` from the user terminal; agent shell tools must use the absolute managed path. Do not substitute a global installation. Verify the managed version is 7.1.2. Check the selected product tenant before operations; harness environment selection does not select a CLI tenant.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Discover `airs cli runtime dlp --help` and the exact resource command. `patterns` and `dictionaries` support CRUD; `filtering-profiles` supports list/get/replace. Inspect current definitions and the supported payload schema before preparing a change. Replacement may replace the whole resource: preserve unrelated patterns, dictionaries, references and profile settings.

Use synthetic test values when validating a pattern or dictionary. Keep actual sensitive matches and dictionary contents out of shared reports unless the task expressly requires them and the destination is appropriate.

DLP data-profile deletion is unsupported in CLI 7.1.2 and exits 2 without sending a request. Do not retry it through guessed REST calls or represent a profile as retired without verified evidence. Use only a supported and authorized alternative.

Read back changed resources and validate the requested detection behavior with a supported scoped test. Use DLP Testing for file/image corpus generation; successful configuration CRUD alone does not establish multimodal detection coverage.

All DLP commands use the selected tenant config, including management OAuth and endpoint overrides. Dictionary creates accept SCM display region labels such as `United States`; a generic 400 does not establish missing entitlement. CSV consumes a header; TXT does not. Backup/restore must preserve keyword content, pagination and duplicates/count evidence. Inspect transfer dry-run output and use the supported create-only transfer contract; never guess a destructive replacement.
