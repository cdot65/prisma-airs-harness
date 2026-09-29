---
title: TypeSafe Jev red-team judge
---

Stable 0.1.2 includes the `prisma-airs-asr-judge` skill and bundled CLI 7.1.5.
The skill scores exported red-team records with TypeSafe Jev and computes an
independent attack success rate. It uses a Node entrypoint and the bundled
TypeScript CLI; no Python installation is required.

## Configure and run

Inside AIRS, open `/typesafe` and choose **Save or replace API key**. Paste the
key into the hidden field. For terminal automation, use
`airs --environment work env typesafe set`. `/doctor` can verify access through
the TypeSafe models endpoint; that does not run a judgment.

Invoke `$prisma-airs-asr-judge attacks.json` in the conversation. The skill starts
with a dry run, then a small recorded probe before a full paid evaluation. Live
runs require approval for the specific judge command to access the native key
store and send scan records to TypeSafe. A sandboxed command failing to read a
saved key does not establish that the key is missing.

The installed entrypoint retrieves the key belonging to its owning environment
and passes it only to the judge child. An inherited `TYPESAFE_API_KEY` takes
precedence. Product CLI tenants remain separate; fetching a job by `--job`
additionally requires the selected tenant's AIRS management credentials.

## Read the results

The output contains `results.json`, `judgments.json` and `summary.md`. Inspect
output-level and attack-level ASR, the 95% Wilson interval, coverage, skipped
records, provider failures, uncertainty and agreement with AIRS threat flags.
Jev supplies typed judgments; code computes metrics and applies the stated
success threshold. Those judgments are estimates, not ground truth.

Explicit replay reuses recorded answers without a new Jev evaluation. It is
never an automatic replacement for a requested live run. Preserve response text
verbatim and report actual coverage and errors rather than inferring accuracy
from a successful API response.

See the [CLI judge reference](https://cdot65.github.io/prisma-airs-cli/cli/redteam/judge/)
and [TypeSafe's programming model](https://docs.typesafe.ai/concepts/how-to-build-with-system-one).
