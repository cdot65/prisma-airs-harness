# Windows command-wrapper source-stage acceptance

Source commits `1117eed3f` and `43ff3f2e8`; 2026-09-08. This bounded packaging
stage passes independent source review. Full authentication release readiness
remains FAIL; no Windows runtime or publication is credited here.

The verifier accepts complete byte-exact wrapper triplets from retained cmd-shim
7.0.0 and 9.0.2 output, preserving source hashes and ISC attribution. Only the
plain Node shebang and a constrained target path are supported. It rejects mixed
families, altered commands, missing members, nonregular/reparse entries,
oversized wrappers and casefold/Unicode-normalized command collisions. Receipts
record template identities and file hashes, not an inferred installed npm version.
The installed npm validator requires every declared Windows command's triplet
before native execution; pre-install staging intentionally does not.

[Independent generator comparison](generator-comparison.json) used both actual
npm implementations on Linux for all five real pinned CLI commands, plus a nested
scoped layout, with installation roots containing spaces. Both full-bundle cases
passed with `require_windows_wrappers=True`. Root independently rehashed all
recorded source files against the final checkout. These fixtures were generated,
not executed as Windows commands. Required Windows installation and PowerShell/cmd
execution are still open.

[Thirty focused tests](focused-tests.txt) passed in 5.875 seconds, covering the
bundle/archive/registry boundary. Five existing packaging tests also passed earlier
in the change, before the final completeness guard. Python formatting and repository
`just fmt` completed; no Rust file changed or executable was rebuilt.

Independent review found a real declaration collision: a Tool/tool alias was
[incorrectly accepted](collision-before.json). The new guard
[rejects that same counterexample](collision-after.json). The first added regression
expected failure after build, but staging already rejected the conflict; the
[original fixture-placement errors](fixture-placement-failure.txt) remain retained.
Moving its assertion to staging produced the final passing suite without weakening
the guard. Actual npm source comparisons were rerun after that correction and the
installed-only completeness requirement.

No CI job was dispatched, credential changed, version bumped or package published.
Windows relaunch awaits clarification of the earlier account-attributed cancellation.
Signing/notarization, actual owner-Mac behavior, authenticated teammate package
installation, migration and all other mandatory release gates retain their prior
status. This source-stage pass does not establish the full release's >=9 score.
