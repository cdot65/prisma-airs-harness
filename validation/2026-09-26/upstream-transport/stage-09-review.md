# Embedded startup policy propagation — in progress

Adapts upstream `9d4d34d436` to the AIRS startup flow. One shared policy controller covers clients created before embedded app-server startup and persists across TUI configuration/session/permission-profile rebuilds. Bootstrap auth remains limited to exact endpoints under local policy; effective application requirements govern content clients.

Fork adaptations:

- The local-daemon variant lacks upstream's newer fallback flag; the existing fork always permits embedded fallback, so it receives the same policy binding as embedded startup. This does not enable the daemon feature or change target selection.
- Preserve the existing startup/picker flow instead of importing absent daemon-selection and late reload loops. Bind the final config at the actual fork reload point.
- Existing test-only picker delegates to managed startup and uses isolated loader overrides; absent upstream test cases are not imported to satisfy argument changes.
- Keep the existing configuration worker return shape; add policy retention without an unrelated allocation refactor.

Production app-server/exec/TUI/CLI compilation passes. Ten targeted Linux activation/reload/session rebuild/bootstrap redirect cases pass; the native combined embedded/CAS run passes 449 cases. The corrected CAS/native pass (441 cases) is recorded separately. Scoped lint and formatting pass. Full affected native package coverage is being retried after the host ran out of disk space; compiled artifacts were retained. Scores remain unassigned until the connected transport feature and workspace gates are complete.
