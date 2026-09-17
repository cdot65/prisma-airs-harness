# Alpha.21 environment lifecycle release

Runtime source: `c2d555d3547d1d9ca79c45c99f97c40b02c1a66f`.
Published to `https://npm.cdot.io` with `latest`, `alpha` and `gateway-validation` pointing to `0.1.0-alpha.21` for the launcher and all three native packages. Publication was explicitly authorized by the owner.

## Changes

- Create, list, inspect, switch, rename and remove environments through `env`.
- Remove top-level `setup` and `status`; use `env create` and `env status`.
- Preserve UUID-based environment state on rename and unregister without deleting files on remove.
- Keep `--environment` as a one-command override and `env use` as the saved default.
- Document inference SSO followed by gateway ServiceNow MCP authorization with the same company identity.

## Verification

The environment implementation passed 780 scoped Rust CLI tests and 11 focused Python onboarding/status checks. Packaging/registry/workflow contracts passed 18 checks and launcher tests passed 16 checks. Native release fixtures ran 47 tests on each target, passing 45 with two expected skips before npm installation. Separate exact-executable lifecycle checks passed on all three targets.

Apple Silicon was built on Jadzia, Developer ID signed and accepted by Apple notarization. Keychain acceptance passed. Linux ARM64 was cross-compiled and then executed natively inside Jadzia's ARM64 Docker Linux VM; its fixture/lifecycle evidence is not a QEMU-only result. Linux x64 was built and exercised natively on the Linux workstation.

Fresh anonymous registry installations ran 47 tests per platform, passing 46 with one platform-specific skip. All exact installed executables passed environment lifecycle checks. Upgrades from alpha.20 preserved configuration on all three targets; upgrades from alpha.14 also passed on Linux x64 and Apple Silicon. Alpha.14 did not provide a Linux ARM64 package. Installed Mac Keychain acceptance passed in the GUI session; SSH does not establish desktop Keychain availability.

Unversioned anonymous npm installs also resolved to alpha.21 with the same native hashes on all three platforms.

Install:

```sh
npm install -g airs-harness@0.1.0-alpha.21 --registry=https://npm.cdot.io
airs-harness --version
```

## Acceptance fixture corrections

Early Mac acceptance jobs failed because fixtures retained a removed `setup` invocation and expected a doctor JSON response after named-environment binding rejection. The corrected tests assert the early rejection, secret redaction, no network requests and unchanged credentials. The original signing-job failure is preserved; the same signed executable subsequently passed the corrected suite.

Initial Linux ARM64 shell tests stored fixture state under the OS temporary directory. Release binaries intentionally refuse executable helper aliases there. Fixtures now use private temporary directories under the user's cache, matching installed state placement. The original failure is retained; the same native bytes passed afterward. Runtime safety checks were not bypassed or modified.

## Limits

This release does not claim a fresh production human SSO login, ServiceNow incident call, frontend expiry/renewal acceptance, full workspace test run or independent release review. The gateway routing requirement and 30-minute SSO idle policy remain unchanged.
