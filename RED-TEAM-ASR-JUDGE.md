# Red-team ASR judge (TypeSafe Jev) proof of concept

Status: implemented and tested at the integration boundary. No live Jev call has been
made from this repository because `TYPESAFE_API_KEY` is not present in the
development environment; every metric below that involves a judgment comes from
hand-authored replay fixtures and carries no accuracy claim.

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
scripts/test_airs_asr_judge.py     21 unittest cases, wired into airs-harness-check.yml
```

The script runs with `python3` from the standard library. When the official
`typesafe-sdk` package (0.7.0 verified) is importable it is used; otherwise a stdlib
HTTP client speaks the documented `POST /v1/systemone` contract with retries on 408,
429 and 5xx. `--provider replay --replay <file>` reproduces a recorded run with no
network, and `--record <file>` captures raw answers plus the exact question texts.

## Verified results

| Check | Result |
|---|---|
| `python3 -m unittest discover -s scripts -p test_airs_asr_judge.py` (system Python 3.14) | 20 passed, 1 skipped (SDK not installed) |
| same suite in a venv with `typesafe-sdk==0.7.0` (SDK adapter uses `httpx2.MockTransport`) | 21 passed |
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

`TYPESAFE_API_KEY` is not set. Steps to complete the paid evaluation are in
`references/methodology.md` (dry run, 25-unit probe, three recorded full runs, human
labeling of 90 sampled disagreements and uncertain units). The real scan JSON
referenced by the prior Gemma-based evaluation (jobs `3ade30a7`, `9e3e8a48`) is not on
this machine; the exported-report layout was reconstructed from that evaluation's
data dictionary and the data-plane OpenAPI schemas.

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
