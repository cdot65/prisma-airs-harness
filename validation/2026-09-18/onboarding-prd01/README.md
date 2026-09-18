# PRD 01 presentation acceptance

Source checkpoint: `ac80867195496fe299525d787c4c53d3f6b8c2a7` on `feat/airs-branded-onboarding`. This stage adds a synthetic preview and a reusable presentation layer; CLI authentication integration follows in PRD 02. Published alpha.22 remains unchanged.

Agent score: **9/10** (visual 1.5, interaction 2, terminal compatibility 1.5, responsiveness/implementation 2, regression/reproducibility 2). Owner visual approval and native platform acceptance remain pending. No authentication is performed by this preview.

Open [gallery.html](gallery.html) for captured terminal animation and normal, compact, tiny, input, recovery and progress screens. Colors and cells come from a real PTY parsed with pyte; browser font metrics and dim rendering can differ from a native terminal. Screenshots show both light and dark backgrounds.

Validation: 11 new unit/snapshot tests; full `just test -p codex-tui --test-threads 8`: **4,331 passed, six existing skips**; 21 real-PTY checks passed; first action visible in 53 ms on Linux x64. Scoped `just fix -p codex-tui` passed; `just fmt` plus explicit TUI formatting completed. Tests precede final comment/format-only changes per repository policy. All new production modules are below 500 lines.

Earlier iterations found missing Braille font glyphs (replaced with block glyphs), four existing cursor tests affected by inherited `NO_COLOR=1`/`TERM=dumb` (passed unchanged under a normal color terminal), and an intermittent existing background-exit race (passed on retry and in the final complete run). No existing shutdown logic was changed. Logs are retained in `/home/cdot/.cache/airs-onboarding-20260918/`.

The review stages are presentation with its snapshots, separately reproducible PTY evidence, then CLI integration. The presentation commit is larger than the preferred change-size guideline because rendering, its terminal ownership guard and tests form one usable unit; production modules stay focused and authentication is a later stage.

Reproduce from the repository root:

```sh
env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor just test -p codex-tui --test-threads 8
cd codex-rs
cargo build -p codex-tui --example airs-onboarding-preview
cd ..
uv run --with pyte==0.8.2 scripts/validate_airs_onboarding_preview.py \
  --binary codex-rs/target/debug/examples/airs-onboarding-preview \
  --output /tmp/airs-onboarding-preview
```

The accepted preview binary hash is in [PREVIEW-ACCEPTANCE.json](PREVIEW-ACCEPTANCE.json). It identifies the captured development bytes, not a signed/distributable harness release.
