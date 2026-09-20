# Stable upgrade and rollback implementation

The upgrade validator now accepts exact stable versions and historical immutable alpha versions. Tags/ranges and a stable version without its previous native/source pins fail before output directories or npm mutations. Legacy `setup` and `airs-harness` command mapping applies only to the early 0.1.0 alpha series; subsequent alpha numbering resets use `airs env create`.

A specification with a stable `previous_version` must declare `previous_release.source_commit` and exactly three target/native-hash pins. Each native acceptance invocation receives its selected pin. A previous package must match that independent baseline before observation, and rollback must restore the same exact bytes. Candidate identity must match the enclosing source-bound release contract.

The maintained lifecycle fixture signs in two independent synthetic identities through the native store, completes a real inference/MCP turn on the previous executable, upgrades in the same npm prefix, resumes that exact conversation and completes another turn, then reinstalls the pinned previous release and completes a third turn. Environment registry, configuration, native binding and epoch remain unchanged. The same native records remain present and authenticated requests reuse their unchanged issuer generations. Both npm command links remain unchanged. No force or uninstall is used. Native records are deleted before success; an owned copy of the initial executable remains available solely for cleanup if a failed install removes the prefix binary.

Linux uses an isolated private D-Bus session; macOS uses namespaced exact synthetic Keychain accounts. No owner records are read or enumerated. Worker execution is bounded to 720 seconds with 60 seconds for graceful cleanup, within the existing 900-second release stage. Failure output retains only a phase and error class, never raw PTY output.

## Evidence

- `roundtrip-unit.log`: 6 tests passed, covering historical/current version selection, invalid input before mutation, canonical all-target baseline pins and invocation, missing/malformed pins, forged/incomplete preservation receipts, and legacy alpha compatibility.
- `all-release-tests.log`: 65 release/spec/publication/receipt tests passed.
- `development-roundtrip-final.json`: bounded actual-runtime driver observation; retained stable Linux x64 native SHA256 `57b58b278ad83d984e024c9c0add6c8c87b9a0cc995f0d9225b3353885884ee9`, current development native `da54e47800ddfe78a0d233508fe0f6b92776f93a804472470056948b20a1c79e`, then exact stable restoration. Three actual resumed MCP turns, preserved identity/history and native cleanup. The fixture copy and full tooling inventory remain in `/tmp/airs-post011-20260919/frozen-roundtrip-development-final`.

The development driver replaces owned native files and uses two binaries which still report 0.1.1. It explicitly sets `npm_upgrade_acceptance: false`; it is not evidence for the new version's npm install/upgrade/rollback or any other platform. Exact candidate and anonymous registry gates must execute the complete npm driver independently on all three supported native targets before publication. Historical alpha upgrade-only receipts cannot satisfy the new stable-baseline roundtrip contract. No production sign-in or upstream ServiceNow acceptance is claimed.
