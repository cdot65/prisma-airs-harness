# Scan JSON fields the judge reads

The script infers the layout from the file rather than trusting a declared schema. Field names below come from the Prisma AIRS AI Red Teaming data-plane OpenAPI (`AttackListSchema`, `AttackDetailResponseSchema`, `AttackOutputResponseSchema`) and from records previously exported with `GET /v1/report/{job_id}/download?file_format=JSON`.

## Accepted layouts

| Layout | Detection | Unit id |
|---|---|---|
| Downloaded report | top-level array of attack records with `prompt` and `output` | `<uuid>#0` |
| API list or detail | object with `data` / `attacks` / `results` / `records` / `items` array | `<uuid>#<output uuid or index>` |
| Detail with outputs | record carries `outputs: [...]` | one unit per output |

Records without a non-empty `prompt` are skipped and counted in `ingestion.skipped_no_prompt`.

## Fields per record

| Field | Use | Notes |
|---|---|---|
| `uuid` / `attack_id` / `id` | attack identity | falls back to `index-N`; counted in `ingestion.fallback_row_ids` |
| `job_id`, `target_id` | provenance only | |
| `prompt` | sent to Jev as `attack.prompt` | required |
| `category` | breakdown | `SECURITY`, `SAFETY`, `BRAND`, `COMPLIANCE` |
| `sub_category`, `sub_category_display_name` | breakdown, objective text | display name preferred |
| `goal` | `attack.objective` when present | null for static library attacks; the script then uses a disclosed proxy from category, goal_category (or sub-category), and the prompt; counted in `ingestion.objective_proxies` |
| `severity` | carried through | not used in ASR |
| `output` or `outputs[].output` | target response | see extraction |
| `threat` or `outputs[].threat` | AIRS verdict per output | `true` means AIRS counted a successful attack |
| `asr` | AIRS per-attack rate 0 to 100 | carried through, not recomputed |
| `error`, `error_message` | skip unit | `error: true` or empty text -> `skipped_error` |
| `marked_safe` | carried through | manual analyst override; not applied to verdicts |

## Response string contract

`output` and `outputs[].output` are the model's direct output strings. The entire
string is sent as `target_response` without parsing JSON, Python literals, A2A
messages, Responses/Chat Completions shapes, or multi-turn content inside it.
Whitespace, escapes, role labels and wrapper keys remain part of the response.
Nested or malformed JSON is model content for Jev to evaluate. An output that
repeats the prompt is not automatically rejected or assigned a verdict.

Non-string values other than missing/null output reject ingestion with a schema
error; they are not coerced into text. Missing/null, whitespace-only strings and
explicit `error: true` records retain the existing skipped-error behavior.
`judgments[].extraction` is `plain` for nonempty strings and `empty` otherwise.

The 24,000-character size bound and replay hashes use the complete output string.
Oversized inputs are excluded explicitly, never truncated. This bound is not a
tokenizer or a guarantee about the provider's token budget.

## Known ambiguities

- Multi-turn content inside an output string is judged as supplied; no last-turn extraction is performed.
- The exported report may omit `outputs[]` and `asr`; the attack-level ASR is then identical to the output-level ASR.
- `threat: null` outputs are excluded from the agreement matrix but still judged.
- File-modality attacks (`attack_modality: FILE`) are judged on prompt text only; the attached document is not fetched.

## Representative export verified September 20, 2026

The supplied export is a flat array of 4,362 records with plain-text outputs, boolean
threat flags, category/sub_category/goal_category, severity, ASR, taxonomy tags and
multi-turn fields. It omits attack IDs, job IDs, target model IDs and explicit goals.
All multi-turn flags are false and turn/generation/multi_turn_prompt are null. Both
normalizers accept all rows; 36 exceed the character bound and are excluded from
judging. Source-row identities and goal-category proxies are disclosed in reports.
Taxonomy tags are not used to infer success or treated as expected attack objectives.

### Prompt normalization is independent

An explicit `kind: message` prompt envelope is decoded as bounded data and its
ordered text parts form the attack prompt. JSON inside a prompt text part stays
literal. Unsupported prompt parts are skipped and counted in `skipped_no_prompt`.
Ordinary JSON prompt strings remain unchanged. Dry runs report
`normalized_prompt_envelopes` when applicable. These rules never apply to `output`.

The representative export contains string outputs that resemble Python-style A2A
messages. They are preserved in full, consistent with the owner's confirmed
response-string contract. This structural observation does not establish a
model-response defect or justify changing Jev's verdicts.
