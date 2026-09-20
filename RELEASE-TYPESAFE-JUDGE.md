# TypeScript judge and in-session TypeSafe setup

Harness **0.1.2-alpha.4.mcp.1** is published under `mcp` at `https://npm.cdot.io`,
bundling CLI **7.1.5**. Stable `latest` remains **0.1.1**.

```bash
npm install -g airs-harness@0.1.2-alpha.4.mcp.1 --registry=https://npm.cdot.io
airs --version
# 0.1.2-alpha.4.mcp.1
airs cli --version
# 7.1.5
```

Restart `airs`, open `/typesafe`, and choose **Save or replace API key**. Enter the
key in the hidden field, then invoke `$prisma-airs-asr-judge attacks.json`.
Existing saved keys work automatically. The optional dialog also shows configuration
status and confirms removal; it preserves the conversation and draft. Saving does
not perform a paid judgment or verify remote access. Never paste a key into chat.

The bundled JavaScript entrypoint invokes the one TypeScript judge and official
TypeSafe JavaScript SDK. Python is no longer required. A native helper resolves
the skill's owning environment and supplies its saved key only to the judge child.
Missing credentials fail explicitly; replay is used only when selected and is
clearly labeled. Model output strings remain verbatim, even when they contain JSON.

Exact candidate and fresh anonymous registry checks passed on Ubuntu x64, native
Linux ARM64 and signed/notarized Apple Silicon, including actual hidden-key dialog
interaction, native storage, a fresh SDK request to a loopback fixture, explicit
replay and upgrade/rollback preservation. No paid Jev calls or model-accuracy
claims are part of these checks. This is not native Windows package acceptance.

Runtime source: `640f30b05ffb149f18be62b74a56804bcda72e62`.
Acceptance/packaging tooling: `d64ccd3fa87eae928d66bf7dce704e4f98b46965`.
See [the release evidence](validation/2026-09-20/typescript-judge-setup/README.md).
The [immutable Git audit](validation/2026-09-20/typescript-judge-setup-git-audit.json)
passed across 327 files and 56 stage receipts at `5eb9594c78`. The CLI Docusaurus
guide is deployed at `fe80cec`; all five checked live pages match the validated build.

# Verbatim model-output judgment

Harness **0.1.2-alpha.3.mcp.1** is published under `mcp` at `https://npm.cdot.io`
and bundles CLI **7.1.4**. Stable `latest` remains **0.1.1**.

```bash
npm install -g airs-harness@0.1.2-alpha.3.mcp.1 --registry=https://npm.cdot.io
airs --version
# 0.1.2-alpha.3.mcp.1
airs cli --version
# 7.1.4
```

Restart `airs` to load the updated embedded skill. Exact candidate and fresh
anonymous registry checks passed on Ubuntu x64, native Linux ARM64 and
signed/notarized Apple Silicon, including verbatim-response replay through both
the bundled CLI and the embedded skill. These tests made no paid Jev calls.
Runtime, tooling and packaging source: `5c2a3c01e10b8b37845333bcea56e5a829840082`.
See [the retained evidence](validation/2026-09-20/verbatim-model-output/README.md).
The [immutable evidence audit](validation/2026-09-20/verbatim-model-output-git-audit.json)
passed across 275 files and 56 stage receipts. The updated CLI Docusaurus guide
is deployed at `1792b14`; all five checked live pages match the validated build.

The CLI and native skill treat `output` as the model's direct response string.
They preserve the complete string, including nested JSON, serialized messages,
whitespace and escapes. Prompt envelope normalization is separate. Non-string
response values are rejected rather than coerced. No response is automatically
rejected or assigned a verdict because it resembles the prompt.

All 4,362 supplied outputs and original prompts remain unchanged, with identical
response hashes across the two implementations. The existing 36 oversized units
are excluded explicitly. This verifies the input contract, not Jev accuracy or a
particular ASR. `unrelated_or_error` may legitimately describe the model output.

Use a new output directory and recording for a small probe after upgrading.
Recordings created from extracted inner response text will fail hash validation
against the complete response string.

## Previous release: TypeSafe Jev judge

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

The [immutable evidence audit](validation/2026-09-20/typesafe-judge-git-audit.json)
passed across 331 retained files and 59 stage receipts.
