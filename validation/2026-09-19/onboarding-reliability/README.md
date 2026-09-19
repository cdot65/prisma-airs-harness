# Onboarding reliability — September 19, 2026

The launcher now checks its declared Node engine requirement before starting a
native command or bundled CLI. Login, cancellation and doctor recovery commands
retain the selected local environment, including valid leading-hyphen names.
Loaded registry names must satisfy the same rules as newly created names before
they can appear in copyable commands. Explicit environment creation and bundled
CLI version documentation now match the implementation.

## Evidence

- `node/`: five entrypoint regressions failed before the runtime guard. The
  existing and expanded Node suite passed 30 tests, followed by one portable
  boundary test with 23 cases. The actual host ran Node 22.23.2; simulated version
  metadata does not constitute execution on Node 18 or Windows.
- `recovery/`: published mcp.2 reproduced unscoped recovery for `staging` and
  `-staging`. Unsafe loaded registry names also failed the new regression before
  the loader fix. The final affected CLI suite passed **955 tests, no skips**.
- `executable/`: the frozen development native passed **54 existing installed
  command/doctor/MCP tests with one Mac-only skip**, plus all **eight terminal
  scenarios** through bash, zsh and fish. The separate maintained recovery test
  passed two environment subcases with Python optimization enabled. Registry,
  credential binding, default environment, history and terminal modes were
  preserved, and the synthetic key was not echoed.
- `BUILD.json` and `SOURCE.patch` bind the development native to its exact
  preformatter source. `SOURCE-FORMATTED.patch` and `COMMITS.json` describe the
  retained implementation. Required formatting passed; unrelated formatter
  churn was restored. Scoped CLI lint passed without changing owned files.

The native SHA-256 is
`d17b0b27ee9352728480c221509dedccaa3b6380d9a1f691921853b95f66d0c0`.
It still reports mcp.2 and is a development verification binary, not a published
release. Logs retain their original scratch-path references; equivalent retained
files live alongside this README. Checksums cover the retained evidence.

## Scope and remaining gates

This closes the implementation/fixture onboarding focus. Three-platform package
builds, native installed acceptance, Mac signing/notarization, npm publication and
fresh registry installs remain separate release gates. Owner-session Ubuntu
investigation and attended production SSO/ServiceNow acceptance were explicitly
deferred by the owner. No owner credentials were accessed, and no production or
full-workspace-green claim is made. The historical full-workspace result remains
documented in the baseline evidence.
