# TypeScript judge and in-session TypeSafe setup

Release: `0.1.2-alpha.4.mcp.1`, preview channel `mcp`, registry `https://npm.cdot.io`.
Bundled Prisma AIRS CLI: `7.1.5`, official TypeSafe JavaScript SDK `0.6.0`.
Stable harness `latest` remains `0.1.1`.

Runtime source: `640f30b05ffb149f18be62b74a56804bcda72e62`.
Acceptance and packaging tooling: `d64ccd3fa87eae928d66bf7dce704e4f98b46965`.
CLI source: `12ffb2ce5df59719a71e576173490f9a1b237a93`.

`SOURCE-FREEZE.json`, `SPEC.json`, `ACCEPTANCE-TOOLING.json`, the native extraction receipts and the individual stage receipts bind the source, tooling, packages and actual tested binaries. Publication and post-publication outcomes are recorded in `publication/PUBLICATION.json`, `OWNER-CANDIDATE-VERIFIED.json` and `OWNER-REGISTRY-VERIFIED.json`. Read those receipts for status; file presence alone is not acceptance.

## Behavior

The embedded JavaScript entrypoint delegates to the one bundled TypeScript judge. The native helper resolves the skill's owning environment and supplies its saved key only to the child. Missing credentials fail explicitly, and replay is clearly labeled; there is no automatic replay fallback. The model's output string is preserved verbatim, even if it contains JSON. Prompt normalization is separate.

`/typesafe` provides hidden key entry, configuration status and confirmed removal inside the conversation. It preserves the draft and session, uses the existing native credential transactions, and sends no paid judgment while saving. The key travels through a private input channel and subprocess stdin, not model/chat events or command arguments.

## Checks

- CLI: 2,029 tests, type checking, lint, formatting, build, release contracts and public-registry dependency audit.
- Initial affected Rust packages: 1,054 tests. TUI: 4,367 passed, six existing skips, plus scoped lint. Seven focused TypeSafe UI tests include reviewed snapshots and private credential handling.
- Node launcher/managed bridge: 33 tests; bundle contracts: 26; native packaging: 13.
- Exact candidate and fresh anonymous registry acceptance cover Ubuntu x64, native Linux ARM64 and signed/notarized Apple Silicon. Installed TypeSafe checks exercise hidden save/cancel/remove, saved-key retrieval despite a different default environment, a fresh SDK request against a loopback fixture, dry-run, explicit replay, missing-key failure, removed-environment rejection and literal output preservation. The judge child has an empty PATH and runs with an absolute Node executable, proving no Python or shell runtime dependency.
- Upgrade/rollback checks retain credentials and conversation state.
- The representative scan was inspected read-only: 4,362 records, zero missing prompts/error outputs and 36 oversized inputs. Only aggregate counts and file hash are retained here.
- CLI Docusaurus deployment and five live-page comparisons are recorded in `implementation/CLI-DOCS715-LIVE.json`.

## Retained failures and limits

The initial full TUI run inherited `NO_COLOR`, causing four color snapshot failures. The complete rerun without that setting passed; unrelated snapshots were not changed to hide the failure. Both logs are retained.

Mac run 277 built successfully and passed native tests, but installed npm acceptance still expected CLI 7.1.4. The installed package correctly reported 7.1.5. The tooling-only one-line correction is retained with the failed logs under `mac-packaging-failure`. The exact native build was preserved for signing and complete installed acceptance; the original failed run is not reported as passing.

The first bundle assembly stopped on an archive integrity mismatch. A separate fresh verification passed all 76 dependency archives against the unchanged lockfile, followed by a fresh assembly with all integrity checks still enforced. The failed assembly receipt and diagnosis are retained; no dependency pins or integrity values were changed to accommodate the failed download.

The Ubuntu candidate upgrade initially ran out of disk space. Duplicate transfer archives and old package input trees were removed only after their hashes matched retained local archives. Failed logs remain under `ubuntu-disk-failure`; the unchanged stage inputs resumed through the release tool’s receipt-verifying `--resume` path. Owner profiles and credentials were not modified.

No owner API keys, real scan text or paid Jev calls are in these tests. Fixture judgments are not model-accuracy, calibration, cost or real-account acceptance evidence. Node command handling supports Windows conventions; this release does not claim a native Windows package. Historical test skips remain visible in their logs.

The immutable Git audit is stored beside this directory after the evidence commit, so it can verify committed bytes without self-reference. `SHA256SUMS` covers all retained files except itself.
