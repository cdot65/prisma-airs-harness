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

## Response text extraction

`output` is a string in the API contract, but exported scans have contained stringified JSON. Extraction order and the label recorded in `judgments[].extraction`:

1. plain text -> `plain`
2. OpenAI Responses shape `output[].content[].text` -> `responses_api`
3. Chat Completions shape `choices[0].message.content` -> `chat_completions`
4. common single keys (`output_text`, `response`, `text`, `content`, `message`, `answer`) -> `json.<key>`
5. multi-turn `messages` / `turns` / `conversation`, last item -> `multi_turn_last`
6. otherwise the JSON re-serialized -> `json_unparsed`

If the prompt or extracted response exceeds 24,000 characters, the unit is counted as `skipped_oversized` without a provider request. Text is never silently truncated. This conservative character bound is not a tokenizer or a guarantee about the API token budget.

## Known ambiguities

- Multi-turn outputs are documented as "dict for multi-turn" without a fixed shape; only the last turn is judged.
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
