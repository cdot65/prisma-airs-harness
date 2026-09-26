# Terminal.app over SSH: review in progress

Adapted upstream `2925d06f5a91703548575e6b52f17591e8af85b2` onto AIRS's existing inline startup rather than importing its fullscreen startup architecture. The pinned crossterm delta (`45fecb9..efa1778`) is one commit that discards secondary device-attribute replies in both Unix input sources. Its own tests cover every input split and preservation inside bracketed paste.

## Required behavior

- Only Unix SSH without a detected multiplexer queries identity. Complete DA1 `1;2` and DA2 `1;95;0` identify Terminal.app; missing replies stay unknown. Pasted signatures cannot select terminal policy.
- Auto uses native scrollback for that terminal; always, never and `--no-alt-screen` remain explicit overrides.
- Cache the probe for startup dialogs, session selection and final configuration. AIRS's initial composer remains inline; no fullscreen default or provider/authentication change.
- Preserve interleaved draft input and swallow actual device replies. The ordinary composer and overlay snapshots should remain unchanged.

## Adversarial findings

1. The first adaptation set policy only after startup session selection, leaving the resume/fork picker outside the fix. Apply the initial config immediately after terminal handoff and refresh it after onboarding config reload, then retain the existing final config application.
2. Initial PTY fixtures mistook the provisional loading composer for the active app. All seven new cases failed because startup intentionally ignores action keys. Wait for the real loaded model header before requesting the transcript overlay. Original failures remain evidence.
3. Cargo's real update also reconciled fifteen existing Windows dependency edges without changing package versions. Retain the generated resolution rather than hand-editing a lock to hide it. Native `just bazel-lock-update` and strict lock check completed; the Bazel delta is the crossterm source key.

## Gate status

Scores withheld. Full local and native TUI tests, final lint, startup-picker executable checks and source parity are pending. This is a self-review with executable evidence, not independent certification. There is no new signed or published Mac preview from this slice yet.

Local final TUI: 4,456/4,456 executed passes, six skips, retries disabled. All seven PTY scenarios pass. Fresh local `codex` and `airs-harness` debug binaries built. The actual `codex resume` picker passed all four auto/always/never/CLI-override cases. Dependency regression tests passed 2/2 in each Unix source (mio and tty), including all byte splits and paste preservation. Its initial invocation failed because the dependency checkout has no repository `local` nextest profile; using its `default` profile fixes the invocation without changing tests. Native and lint gates still pending.

The initial Mac suite stopped at 664/4,461 when its temporary fixtures filled disk (nextest reporting error; 3,797 cases unrun). This is an invalid validation run. Cleanup preserved linked binaries/libraries and evidence, removing closed test fixtures, idle incremental caches and intermediate object files from the task-owned target. A scheduling error started the next rebuild before cleanup ended; the archive build failed on a removed intermediate object, with no tests executed. Both failures are retained. Cleanup completed with 41 GiB free, and the fresh full rerun now gives each test subprocess a separate temporary root, cleaning successful cases immediately and retaining failed cases. No Rust process was killed, no test assertions or product logic changed to mask these infrastructure failures.
