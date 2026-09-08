# Cancelled Windows candidate and planned build checkpoint

Run 34259921603, runtime c10e3f99393f7d79dbe48fa228a9a18e548953e1,
ended cancelled. GitHub's check annotation says "The run was canceled by @cdot65."
This attributes an account, not a specific person or UI/API action. The job lasted
156 minutes 32 seconds against a configured 180-minute timeout. Timeout and OOM
were not established. The full private log's SHA256 and sanitized observations
are retained in receipt.json; raw logs are not committed.

The run started cold: matching cache restoration missed. The native contract
step succeeded, but the optimized executable build never reported successful
completion. Its last progress included clap_complete at 20:04:42 UTC and an
unused_mut warning in airs_secret_terminal.rs at 20:07:25 UTC. Cancellation was
reported at 20:31:15 UTC. The warning was not a compiler error. Completed evidence
does not identify the exact rustc/linker operation active at cancellation.

The successful-compilation cache step was skipped. No matching Windows v1 cache
existed at review and no raw executable artifact was listed. Only the small runner
evidence artifact survived. Actual credential/ConPTY/npm acceptance was not reached.
The command already selected codex-cli and airs-harness; the large dependency
build was not an accidental workspace-wide command.

## Bounded planned remedy

The owned Windows workflow now builds the real codex-cli library target first,
with the same CLI optimization override, release profile, lockfile, job count and
features as the following executable build. This completes normal dependencies
before executable-specific work. Only after that command exits successfully does
an optional cache save run under a distinct key ending -lib. No cache is taken
while a compiler command is active. Final executable artifact upload still
precedes the separate final cache save.

The library receipt records source/run/attempt, completion timestamps, log hash,
private nonrelease status and the cache action outcome. It explicitly does not
claim a successful future cache restore. Both library and executable commands
capture Cargo fingerprint info logs and separate timing reports. Concurrency stays
at two compiler jobs and the job timeout remains 180 minutes. The checkpoint is
not proof that a cold build fits that budget; timing remains to be measured.

Local verification: two Windows artifact tests and five package tests passed.
YAML parsed and structural assertions checked command scope, matching compiler
options, cache success condition, optional cache failure behavior, distinct key,
artifact-before-final-cache ordering, and unchanged concurrency/timeout. Actual
PowerShell compilation/cache behavior has not been run for this change. No Rust
build, Windows dispatch, publication or source154 runtime change was performed.
A relaunch remains separate from this reviewable workflow preparation.


## Completed Windows unit stage

Before cancellation, the job passed 26 unit contract tests with zero skips
(Nextest run `96526853-e23f-49a9-ad76-5ef974c0e8f9`). The tests covered token
validation, chunk/transaction failures, namespace separation and typed Windows
diagnostics. Their [names and source identities](native-contracts.json) are
retained with the full private job-log hash. The identity and keyring crate trees
are unchanged between c10 and source154; this comparison does not rename the
historical test run or establish a current Windows executable pass. Storage
contract mocks do not replace real Credential Manager lifecycle acceptance.
