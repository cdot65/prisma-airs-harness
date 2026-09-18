# AIRS environment and sign-in onboarding — PRD 02

Implementation gate: **9/10**, September 18, 2026, 03:45 UTC. This is the implementing agent's assessment; owner review remains pending. The published alpha.22 packages are unchanged.

The welcome screen now guides first environment creation, public company settings, browser/device/manual SSO, hidden workspace-key entry, native credential persistence and gateway verification. Returning signed-in users skip it. Environment selection resolves before the process binds its session home. A saved credential remains a distinct outcome when gateway access is denied or unavailable.

Review stages: `981e30a197` adds deferred resolution and progress/outcome adapters; `1dabf13be1` integrates the controller and public fields; `df1e1b5dec` adds the HTTPS/PTY fixtures; `127649ff60` adds native walkthroughs and regression fixture corrections. Each source stage is under 800 changed lines.

## Evidence

- Full CLI/TUI suite: **5,264 passed**, six existing skips. After the final recovery changes, **256 AIRS-focused tests passed**.
- Python harness regressions: **129 run**, 127 passed and two expected skips. The retained baseline is the published alpha.22 native binary.
- Native Linux walkthrough: **16 passed**, nine local OAuth exchanges with PKCE verified, eight gateway requests. Browser-launch failure completes authentication using the URL actually rendered by AIRS. Hidden workspace keys also persist and authorize a probe successfully.
- Terminal preview: **21 passed**, primary action available in **52 ms**, including animation, compact/tiny layouts, paste, resize, cancellation, signals and static colorless rendering.
- `just fix -p codex-cli -p codex-tui`, `just fmt`, explicit formatting of both Rust crates and Ruff for the new fixture code passed. No clippy warnings were emitted. Unrelated baseline formatting changes were restored.

`ONBOARDING-ACCEPTANCE.json` identifies the final development executable as `c49aa7707eed17ef6604cf54a260ec392680644672bee5ff88326007725c18d0`. Development binaries are not release artifacts; PRD 03 freezes and verifies separately identified distribution candidates. `PREVIEW-ACCEPTANCE.json` identifies the independent preview executable.

Open [gallery.html](gallery.html) for actual terminal cells, with dark/light backgrounds. The PNGs show the same native walkthrough. All values are synthetic; no production account, service, token or browser session was used. The fixture uses temporary HTTPS trust and a private D-Bus Secret Service, then removes its temporary state and stops the owned daemon.

## Reproduce

From `codex-rs`, with Rust 1.95 and the repository's prerequisites:

```sh
env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor just test -p codex-cli -p codex-tui
cargo build -p codex-cli --bin airs-harness
```

From the repository root, on Linux with `dbus-run-session`, `gnome-keyring-daemon`, `secret-tool`, OpenSSL and uv installed:

```sh
dbus-run-session -- uv run --with pyte==0.8.2 scripts/validate_airs_onboarding.py \
  --binary codex-rs/target/debug/airs-harness --output /tmp/airs-onboarding-review
```

Always use a fresh private bus. The fixture refuses a bus with an existing Secret Service owner. It never changes system certificate trust or uses production identity endpoints.

## Iterations and score

Initial fixture setup exposed a CA/leaf mismatch and a Secret Service activation race. The fixture now uses a proper signing CA and starts an unlocked private daemon before probing it. An existing plain-terminal test was explicitly assigned `TERM=dumb`; rich-screen behavior is covered by the rendered PTY driver. Migration fixtures now resolve named environment homes, and inject an actual valid digest mismatch when a recent baseline already uses canonical helper pins. These corrections preserve the before-send rejection assertions.

| Dimension | Score | Evidence and remaining review |
| --- | --- | --- |
| First-run and returning flow | 2/2 | Native creation, session entry and returning-user bypass |
| Authentication and persistence | 2/2 | Local HTTPS OAuth/JWT/PKCE and native Secret Service; hidden key success |
| Recovery | 1.5/2 | Browser failure, cancellation, invalid nonce, preflight storage failure, denial/offline retry; post-authorization storage faults retain existing Rust coverage rather than new native fault injection |
| Environment/command isolation | 2/2 | Default/history preserved, selected home used, scripted regression suite passes |
| UX and maintainability | 1.5/2 | Reviewed captures, focused source stages, snapshots and lint; owner visual review remains |

All hard gates pass. No attended production SSO, ServiceNow operation, Apple Silicon or Linux ARM64 acceptance is claimed by this evidence. Those platform checks belong to PRD 03; the live organization workflow remains an owner test.
