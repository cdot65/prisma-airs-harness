# Compiler-cache action startup failure

Both retained identity runs ended in `startup_failure` before jobs/check runs
were created. GitHub returned no log archive (404), and its REST run/check-suite
responses did not expose a detailed startup annotation.

The repository permits GitHub-owned actions plus exactly two external action
pins: the existing Rust toolchain and installation actions. The new exact
Mozilla sccache action pin is absent. This policy mismatch explains the
pre-execution rejection; retries without correcting it cannot establish CI
acceptance. The repository policy and run identities are retained in
[FAILURE.json](FAILURE.json).

Checksum-verified actionlint 1.7.12 accepts all three modified workflows. YAML
parsing and semantic linting cannot establish that a repository allows every
referenced external action. Future CI review must check both. The initial local
validation did not inspect this external repository policy and missed it.

At this checkpoint no policy was changed, no replacement native build was
started, and the already running release build was untouched. The minimal
repair is to add only the reviewed immutable Mozilla action pin to the existing
allowlist while preserving all other settings, then observe a fresh identity
run. The alternative is to redesign setup around already approved actions;
that does not make an unverified backend or speedup a passing result.

## Authorized policy repair

The owner-authorized repair appended only the immutable Mozilla action pin.
Readback matched the complete expected before/after policy; GitHub-owned and
verified-publisher settings and both pre-existing pins were preserved. The
first explicit identity dispatch after the change created both Linux and Apple
Silicon jobs, which confirms the startup blocker was removed. Completion and
compiler-cache performance are separate, pending gates at this checkpoint.
The earlier preflight-only startup failure is also retained in
[REPAIR.json](REPAIR.json). Its replacement dispatch is owned by the parent
agent; no additional release build was started here.

The npm lane now runs pinned actionlint through the already approved install
action. The three modified workflows pass it locally. Source review covered
Mozilla's setup and statistics entrypoints at the permitted commit, including
release SHA256 comparison and existing Actions cache credential export; this
was not an exhaustive audit of its bundled dependencies.
