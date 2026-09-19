# Focus 3 independent review — 9.3/10, passed

Completeness **3/3**, capability **2.5/3**, best practices **2/2**, optimization **1.8/2**. All scoped development gates pass. Three-platform distribution and live acceptance are separate gates, not inferred from this score.

Independently checked frozen binary SHA256 `d17b0b27ee9352728480c221509dedccaa3b6380d9a1f691921853b95f66d0c0`, source patch hash, all declared installed/recovery log hashes and every post-format owned source hash. Raw results establish **955/955 CLI tests**, **54 installed/doctor/MCP passes with 1 platform skip**, the new recovery fixture's **2 environment subcases** including optimized Python execution, and **8 terminal checks**. The npm suite passed **30 tests**, followed by the separate portable runtime boundary test. The independent729-case comparison against npm semver found no mismatches. Actual Node18 and Windows execution are not claimed.

The only substantive code finding is resolved: registry loading now validates all names through the existing allowlist before they can become copyable command text. RED evidence proves prior acceptance of unsafe names; current tests prove shell/newline/ESC rejection with no echo or registry rewrite. Existing leading-hyphen identifiers remain valid and correctly render with equals syntax.

Selected-home recovery, cancellation and doctor guidance preserve the saved default, credentials and history. The Linux fixture follows the displayed command to the selected denied gateway, verifying behavior beyond string matching. Node rejection precedes native, product and completion children, including the direct managed wrapper. Portable policy tests no longer inherit the POSIX child-fixture skip. New module syntax/import inspection supports Node18-compatible rejection but is not presented as a real Node18 test.

`just fmt` and `just fix -p codex-cli` passed; unrelated formatter changes were restored and lint made no additional owned source changes. Tests were appropriately not repeated after formatting/lint-only work. No blocking findings remain.

The0.5 capability reserve covers future exact native/npm acceptance on all three supported platforms; the0.2 optimization reserve reflects minor iteration overhead and duplicate but cheap guarded CLI checks. Final publication still requires the established native/signing/integrity/channel gates. The owner-specific Ubuntu incident and attended production login/ServiceNow checks remain deferred by the owner. No new publication or green full-workspace claim is made here.
