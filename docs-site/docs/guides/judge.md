---
title: TypeSafe Jev red-team judge
---

This page has two parts. The first explains how the judge works and what its
scores do and do not mean. The second is the procedure: save a key, run the skill
and read the output.

## How the judge works

The harness includes the `prisma-airs-asr-judge` skill. It scores exported
red-team records with TypeSafe Jev and computes an independent attack success
rate (ASR). It uses a Node entrypoint and the bundled TypeScript CLI; no Python
installation is required.

**Who does what.** Jev supplies typed judgments. Code computes the metrics and
applies the stated success threshold. Those judgments are estimates, not ground
truth, so a clean result from the API is not evidence that the assessment is
correct.

**Why a live run needs approval.** A live run sends scan records to TypeSafe and
needs the saved key, which lives in the native credential store. The sandbox blocks
that store for ordinary commands, so the harness asks you to approve the specific
judge command. This also means a sandboxed command failing to read a saved key
does not establish that the key is missing. The workspace sandbox stays on for
everything else.

**How the key travels.** The installed entrypoint retrieves the key belonging to
its owning environment and passes it only to the judge child. An inherited
`TYPESAFE_API_KEY` takes precedence. Product CLI tenants remain separate, so
fetching a job by `--job` additionally requires the selected tenant's AIRS
management credentials.

**Why it starts small.** The skill begins with a dry run, then a small recorded
probe, before a full paid evaluation. Each step lets you inspect the workflow
before it costs anything or sends data.

**What replay is.** Explicit replay reuses recorded answers without a new Jev
evaluation. It is never an automatic replacement for a requested live run.

## Run the judge

### 1. Save the key

Inside AIRS, open `/typesafe` and choose **Save or replace API key**. Paste the
key into the hidden field. For terminal automation, use
`airs --environment work env typesafe set`.

### 2. Check access

`/doctor` can verify access through the TypeSafe models endpoint. That does not
run a judgment.

### 3. Run it

Invoke `$prisma-airs-asr-judge attacks.json` in the conversation. Approve the
specific judge command when asked, so it can access the native key store and send
scan records to TypeSafe.

### 4. Read the results

The output contains `results.json`, `judgments.json` and `summary.md`. Inspect
output-level and attack-level ASR, the 95% Wilson interval, coverage, skipped
records, provider failures, uncertainty and agreement with AIRS threat flags.
Preserve response text verbatim and report actual coverage and errors rather than
inferring accuracy from a successful API response.

See the [CLI judge reference](https://cdot65.github.io/prisma-airs-cli/cli/redteam/judge/)
and [TypeSafe's programming model](https://docs.typesafe.ai/concepts/how-to-build-with-system-one).
