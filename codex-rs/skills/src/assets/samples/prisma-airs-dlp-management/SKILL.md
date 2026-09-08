---
name: prisma-airs-dlp-management
description: "Manage Prisma AIRS DLP filtering profiles, patterns, data profiles and dictionaries while preserving supported API boundaries."
---

# DLP Management

Use the harness-managed CLI 5.2.0. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples below use `airs` as shorthand for that executable. Do not substitute a global installation. Outside the npm harness, verify `airs --version` is 5.2.0 first.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Discover `airs runtime dlp --help` and the exact resource command. `patterns` and `dictionaries` support CRUD; `filtering-profiles` supports list/get/replace. Inspect current definitions and the supported payload schema before preparing a change. Replacement may replace the whole resource: preserve unrelated patterns, dictionaries, references and profile settings.

Use synthetic test values when validating a pattern or dictionary. Keep actual sensitive matches and dictionary contents out of shared reports unless the task expressly requires them and the destination is appropriate.

DLP data-profile deletion is unsupported in CLI 5.2.0 and exits 2 without sending a request. Do not retry it through guessed REST calls or represent a profile as retired without verified evidence. Use only a supported and authorized alternative.

Read back changed resources and validate the requested detection behavior with a supported scoped test. Use DLP Testing for file/image corpus generation; successful configuration CRUD alone does not establish multimodal detection coverage.
