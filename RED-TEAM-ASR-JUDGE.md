# TypeScript judge and environment-scoped credentials

The **0.1.2-alpha.4.mcp.1 candidate**, with CLI **7.1.5**, replaces the Python
proof of concept below. Publication status is in [the release record](RELEASE-TYPESAFE-JUDGE.md).

The embedded skill invokes `scripts/asr_judge.mjs`, a small JavaScript entrypoint
that delegates to the pinned TypeScript CLI. The CLI owns ingestion, questions,
the official `@typesafe-ai/sdk` 0.6.0 adapter, replay and metrics. This workflow
requires Node, already provided by the npm installation, and no Python runtime.

The entrypoint derives the owning environment from its installed location. The
native helper verifies that home is still registered before retrieving the key.
A changed default cannot select another environment. Keys are passed only to the
judge child; they never appear in arguments or get copied into CLI tenant files.
Existing shell keys take precedence. Help, dry-run and explicit replay remain
credential-free. Missing credentials fail; replay is never an automatic fallback.
Replay reports clearly say that old answers were reused, with no new evaluation.

Standalone CLI invocations still read the selected tenant's `typesafeApiKey`.
Only the harness's explicit child credential mode consumes injected variables.
The SDK has logging disabled and follows the adapter's existing retry and
redirect rules. Output strings remain verbatim. No new live Jev accuracy claim
is made. Native Windows acceptance remains separate from portable Node tests.

The [JavaScript SDK documentation](https://docs.typesafe.ai/sdk/javascript.md)
and v0.6.0 client source were checked on September 20, 2026.

---

The following is the historical Python proof-of-concept record. Its Python
commands and separate script implementation are retired by the candidate above.

# Red-team ASR judge (TypeSafe Jev) proof of concept

Status: implemented and tested at the integration boundary. No live Jev call has been
made from this repository because `TYPESAFE_API_KEY` is not present in the
development environment; the illustrative ASR metrics below come from
hand-authored replay fixtures and carries no accuracy claim.

Use `/typesafe` inside AIRS to save, replace, inspect or remove the optional key.
Entry is hidden and the conversation and draft remain intact.

## What was built

A native system skill, `prisma-airs-asr-judge`, embedded like the other
`prisma-airs-*` skills under `codex-rs/skills/src/assets/samples/` and installed into
`CODEX_HOME/skills/.system` on startup. The `include_dir!` embedding picks up the new directory. Rust CLI additions provide optional per-environment key storage, setup, doctor, and an explicit child-command wrapper; core agent behavior is unchanged.

```
codex-rs/skills/src/assets/samples/prisma-airs-asr-judge/
  SKILL.md                         agent guidance and routing
  scripts/asr_judge.py             ingest -> normalize -> judge -> aggregate (stdlib only)
  references/scan-schema.md        fields read, layouts accepted, extraction rules, ambiguities
  references/methodology.md        questions, success policy, metrics, limits, evaluation plan
  references/fixtures/sample-scan.json        synthetic 10-record scan in the exported-report shape
  references/fixtures/sample-judgments.json   replay answers in the /v1/systemone wire shape
scripts/test_airs_asr_judge.py     22 unittest cases, wired into airs-harness-check.yml
```

The script runs with `python3` from the standard library. When the official
`typesafe-sdk` package (0.7.0 verified) is importable it is used; otherwise a stdlib
HTTP client speaks the documented `POST /v1/systemone` contract with retries on 408,
429 and 5xx. `--provider replay --replay <file>` reproduces a recorded run with no
network, and `--record <file>` captures raw answers plus the exact question texts.

## Verified results

| Check | Result |
|---|---|
| `python3 -m unittest discover -s scripts -p test_airs_asr_judge.py` (system Python 3.14) | 21 passed, 1 skipped (SDK not installed) |
| same suite in a venv with `typesafe-sdk==0.7.0` (SDK adapter uses `httpx2.MockTransport`) | 22 passed |
| affected Rust packages (CLI, TUI, home, skills, V8) | 5,416 passed; 6 skipped |
| managed CLI coverage suite | 1,997 passed; 97.72% lines/statements |
| npm launcher tests | 31 passed |
| `ruff format` / `ruff check` via the repo's `scripts` project | clean |
| replay run on the sample fixture | output-level ASR 66.7% (Wilson 95%: 35.4% to 87.9%), AIRS threat-flag ASR 33.3%, agreement 66.7%, 1 error output skipped, 2 uncertain |

The fixture numbers only demonstrate the arithmetic; they say nothing about Jev.

## Configuring the TypeSafe key

`TYPESAFE_API_KEY` in the shell is used as is. To bind a key to a harness environment,
answer **y** at the optional prompt during `airs env create` (plain and branded
flows), or run `airs env typesafe set` (hidden input; `--stdin` for automation;
`--model` and `--base-url` optional). The key is stored in the OS credential store
under the namespace `io.cdot.airs-terminal.typesafe`, with public settings and a
fingerprint in the environment's `typesafe.json`. Use `airs env typesafe exec -- python3 /path/to/asr_judge.py scan.json --out new-results` to pass `TYPESAFE_API_KEY`, `TYPESAFE_BASE_URL`, and `TYPESAFE_DEFAULT_MODEL` only to that child process. Existing shell variables take precedence. Unrelated harness sessions never load the optional key; failed key changes retain a cleanup journal for retry. `airs env typesafe status|clear`
inspect and remove the binding. Doctor adds the row `typesafe_judge`; with
`--verify-access` it calls `GET /v1/models`, the endpoint the official SDK uses,
which consumes no judgment tokens. The managed CLI exposes the same judgment as
`airs cli redteam judge` and reads `typesafeApiKey` from its selected tenant file,
never from the environment.

## Blocker for a live run

`TYPESAFE_API_KEY` is not set. The owner supplied `/var/tmp/attacks.json` on September 20: 15,049,332 bytes, 4,362 rows, SHA-256 `79318e7caddfc3a64fcb97a33a14524387eefa85853bc56240732ecf714826e0`. Both adapters normalize it identically and the source hash remains unchanged. A deterministic fake adapter exercised 4,326 eligible units; 36 oversized units were excluded before any provider call. This verifies ingestion and plumbing, not Jev accuracy.

The export has no attack/job IDs or explicit objectives. All rows use disclosed row-index IDs and category/goal-category/prompt objective proxies. Grouped ASR therefore describes source rows, not independently identified attacks. No prompt or response text was copied into repository fixtures or receipts. Complete the paid evaluation and representative held-out human labeling in `references/methodology.md` after configuring a TypeSafe credential.

## Research summary (verified 2026-09-20)

Jev is TypeSafe's hosted System One model. The official [HTTP API](https://docs.typesafe.ai/api.md) documents bearer authentication and `POST /v1/systemone`; [models](https://docs.typesafe.ai/models.md) documents the authenticated `GET /v1/models` probe and model aliases. Pin a version for evaluation and retain the returned model ID. Rate limits and pricing can change; consult that page when budgeting.

[Noul](https://docs.typesafe.ai/primitives/noul.md) returns a yes probability. [Choice](https://docs.typesafe.ai/primitives/choice.md) and [Score](https://docs.typesafe.ai/primitives/score.md) return typed values and probability distributions. Their [confidence](https://docs.typesafe.ai/confidence.md) is distribution concentration, not measured correctness on red-team scans; Noul has no separate confidence. The [Python SDK](https://docs.typesafe.ai/sdk/python.md) provides `TypeSafeClient`, `Noul`, `Choice`, and `Score`; the POC tests that adapter with a mock transport.

The [self-consistency cookbook](https://docs.typesafe.ai/cookbooks/consistency_noul_cookbook.md) motivates repeat runs and an explicit uncertain band. Its task and results do not establish red-team accuracy. The POC keeps thresholds, aggregation, and coverage in ordinary code. Three questions share state but do not consume each other's answers.

The original social posts and missing LangChain experiment URL are not evidence for this implementation. No comparative accuracy, cost advantage, or calibration claim is made. No live Jev response has been observed here.

## Suitability assessment

Jev fits the shape of the task: one prompt-response pair, one binary judgment plus
two explanatory judgments, thousands of units, and a need for probabilities rather
than prose. Three properties argue for caution and are encoded in the methodology:
red-team responses are adversarial text and TypeSafe documents that such text can
steer the model; the model cannot abstain; and it gives no rationale, so disagreements
with AIRS must be human-reviewed. The proof of concept is therefore an independent
estimator under a stated threshold policy, not a replacement for the AIRS verdict
until the human-labeled sample in the evaluation plan has been completed.
