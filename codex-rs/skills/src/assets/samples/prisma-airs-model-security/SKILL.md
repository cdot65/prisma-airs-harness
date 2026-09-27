---
name: prisma-airs-model-security
description: "Manage Prisma AIRS model-security groups, rules, rule instances, scans and supply-chain findings for ML model artifacts."
---

# Model Security

Use the harness-managed Prisma AIRS CLI. On POSIX invoke `"$AIRS_MANAGED_CLI"`; on PowerShell invoke `& $env:AIRS_MANAGED_CLI`. Examples use `airs cli` from the user terminal; agent shell tools must use the absolute managed path. Do not substitute a global installation. Check the managed `--version` and command-specific help before choosing flags. Check the selected product tenant before operations; harness environment selection does not select a CLI tenant.

For missing credentials or setup, read [Prisma AIRS CLI setup](../prisma-airs-cli/SKILL.md). Use command-specific `--help` and structured output to establish exact flags and schemas. Existing task authorization applies; request additional authorization only when the proposed target or action falls outside it. Keep secrets out of prompts, command arguments, reports and debug logs.

Discover `airs cli model-security --help`, then `groups`, `rules`, `rule-instances`, `scans` and `models`. Inspect the group/rule schema before mutation, preserve unrelated policy settings and use stable IDs. Model catalog operations are read-only where the CLI indicates that boundary.

For a scan, establish the authorized artifact/source and scan configuration, submit with the supported command, record the ID, and inspect terminal status, evaluations, violations and file-level results. Distinguish submission from completion and unscanned content from clean findings.

`model-security install` provisions a separate Python scanner client; it is not required for browsing management resources. It requires `uv` or Python/venv/pip and must be explicitly part of the requested work. `install --dry-run` still obtains PyPI authentication and prints a potentially credential-bearing URL; `pypi-auth` also exposes authentication material. Do not use either as routine discovery. Keep provisioning output and credentials out of inference/history and verify supported platform behavior first; this CLI version's pip fallback assumes a Unix `.venv/bin/pip` path.
