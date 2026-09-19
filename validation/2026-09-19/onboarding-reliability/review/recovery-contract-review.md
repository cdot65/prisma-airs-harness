# Focus 3 selected-environment recovery review

Read-only review of the full current CLI, launcher, snapshots, documentation and fixture diff. Score pending final evidence and resolution of one loaded-registry validation finding.

## Sound behavior

- `name_for_home` derives the public label from the process's already selected home, not the active default, so another process changing the saved default does not redirect guidance.
- Welcome flows carry the actual Selection through company setup, cancellation, verification and retry; no durable default is changed solely to render guidance.
- One command renderer handles valid leading-hyphen names through `--environment=NAME`. Parser tests include `-staging` and `--`.
- Doctor pending-cleanup output is scoped to the inspected environment and continues to inspect rather than retry deletion or mutate state.
- Explicit `env create --gateway-url` help/docs accurately describe configuration-only creation; workspace-key login is shown as a separate hidden-input step. Bundled CLI documentation now matches pinned 7.0.0.
- The new installed Linux fixture uses a private D-Bus and disposable encrypted Secret Service, local HTTPS and synthetic keys. It verifies denied inference shows the selected name, copied doctor command reaches that same denied gateway, cancellation has selected retry command, pending metadata is read-only, and saved default/other history are preserved. Both regular and leading-hyphen environments run. Timeout cleanup targets only its new process group. It does not assert live production authorization.
- Portable Node boundary tests now exercise the guard directly through a file URL in an isolated ESM package, without native shebang or platform skips. Actual Linux execution and simulated version limitations remain explicit; Windows execution is not claimed.

## Finding requiring small correction

P2: the new renderer's unquoted identifier safety relies on `validate_name`, but the registry loader currently accepts arbitrary JSON object keys. Normal create and rename validate names; a manually corrupted registry can therefore send shell metacharacters or control characters into a new copyable recovery command.

Recommended minimum fix: validate each `registry.environments` key through the existing `validate_name` inside `read()`, before returning names to select/resolve/choices/name_for_home. This is deterministic validation only, makes no file writes, preserves all supported names, requires no schema migration, and uses an existing fixed diagnostic without echoing the invalid value. Do not fall back to unscoped `airs`, and avoid POSIX-only quoting as a purported cross-platform solution.

Test a persisted malformed key containing shell/control canary and assert read/resolve fail without echoing it. Keep leading-hyphen and valid punctuation tests passing. Final score waits for source and acceptance after that correction.

## Resolution

The registry validation recommendation is implemented and independently reviewed. RED and full CLI GREEN evidence confirm rejection of unsafe loaded names without echoing or rewriting. Final scoped review is recorded in REVIEW.json and REVIEW.md.
