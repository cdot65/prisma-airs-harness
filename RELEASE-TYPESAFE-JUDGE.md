# TypeSafe Jev judge preview

Harness **0.1.2-alpha.2.mcp.1** is published under `mcp` at
`https://npm.cdot.io` and bundles Prisma AIRS CLI **7.1.2**. Stable `latest`
remains **0.1.1**, whose bundle is CLI 7.0.0. Installing the standalone CLI
separately does not change an older harness's bundled CLI.

```bash
npm install -g airs-harness@0.1.2-alpha.2.mcp.1 --registry=https://npm.cdot.io
airs --version
airs cli --version
airs cli redteam judge --help
```

Expected versions: harness `0.1.2-alpha.2.mcp.1`, CLI `7.1.2`. No separate
product CLI installation is needed for `airs cli ...`.

## Configure and try the judge

The managed CLI reads TypeSafe credentials from its selected CLI tenant. List
registered tenants, select the intended one, then set `typesafeApiKey` through
the hidden prompt:

```bash
airs cli tenant list
airs cli tenant switch development
airs cli tenant set development typesafeApiKey
```

Replace `development` with your tenant's name. Harness environments and CLI
tenants retain independent credential configuration. For the native
`prisma-airs-asr-judge` skill instead, `airs --environment work env typesafe set`
saves an environment-scoped key in the native credential store. The skill's
`env typesafe exec` path exposes it only to the chosen child process; ordinary
inference and MCP sessions do not load it. Existing TypeSafe shell variables
remain supported for the native skill, but the managed CLI ignores credential
environment variables.

Inspect a local scan without a credential or network request:

```bash
airs cli redteam judge ./scan.json --out ./judge-preview --dry-run
```

After reviewing the ingestion notes, run a bounded live evaluation and retain
answers for replay. This sends eligible scan text to TypeSafe:

```bash
airs cli redteam judge ./scan.json --out ./judge-run-1 \
  --limit 25 --record ./judge-recording-1.json

airs cli redteam judge ./scan.json --out ./judge-replay-1 \
  --limit 25 --provider replay --replay ./judge-recording-1.json --threshold 0.6
```

Use new output paths for every run. Existing files are never overwritten. Inputs
with a prompt or response over 24,000 characters are excluded explicitly rather
than truncated. Reports disclose missing objective/identity proxies, coverage,
provider errors and the selected ASR threshold. Model probabilities are not
proven calibrated ASR or ground truth.

## Validation and release provenance

Candidate acceptance passed on Ubuntu x64, native Linux ARM64 and signed/notarized
Apple Silicon. Each platform ran the 53-test installed regression suite (three
platform-specific skips on Mac and one on each Linux host), onboarding, terminal,
MCP, doctor, managed CLI, command-output and upgrade checks. Extra lifecycle and
native TypeSafe key rotation/isolation/cleanup checks passed on all three.

The actual installed `airs cli redteam judge` command also passed help, local
dry-run and synthetic replay on all three platforms, including explicit
oversized-input exclusion. The original 4,362-row export was tested read-only
through deterministic adapters; 4,326 rows were eligible and 36 oversized rows
were excluded. No live Jev accuracy or calibration claim is made.

Fresh anonymous registry acceptance passed on all three native platforms,
including the installed regression suite, upgrade preservation and the direct
Jev command dry-run/replay/oversized-input checks. The CLI Docusaurus site is
deployed at `c49a892`; all five checked live pages match the validated build.

Runtime: `835b32803152fcd7d4dabbdaed983de9321be3b6`.
Acceptance tooling: `7b03cbeafe574aef32004df44b7f35c9b89e2ff9`.
Packaging: see `validation/2026-09-20/typesafe-judge/SPEC.json`.
The failed assembly, first onboarding attempts and their corrections are retained
alongside successful results in that evidence directory.

See [the judge methodology](RED-TEAM-ASR-JUDGE.md) and the
[published CLI judge guide](https://cdot65.github.io/prisma-airs-cli/cli/redteam/judge/).
