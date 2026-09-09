# First real-registry acceptance attempt

Run34339428446 tested exact already-published alpha10 archives. All four jobs
failed and remain recorded as failures. npm10 installed on both platforms and
verified native/archive integrity and Mac online notarization. Its deterministic
suites failed because this new workflow omitted established runner prerequisites:
Linux bubblewrap/ripgrep and disposable-host user-namespace configuration, and
Mac ripgrep. npm12 failed before installation because its npm view JSON wraps
one metadata object in a singleton array. The validator assumed npm10's object.

Commitf1c1c88d0 supports both strict shapes, rejects empty/ambiguous results, and
provisions the same prerequisites as existing acceptance workflows. Seven
publisher and four registry-validator tests passed; a fresh four-job acceptance
was dispatched. No package archive was modified or republished for these fixes.
