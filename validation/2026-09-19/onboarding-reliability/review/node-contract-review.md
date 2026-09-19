# Focus 3 Node runtime guard — independent review

Read-only code review covers runtime.js, launcher.js, prisma-cli.js, both public executable shims, direct POSIX/Windows managed wrappers, completion module and new launcher tests. No application mutations by this reviewer. Final focus score remains pending the separate onboarding/recovery work and its checks.

## Findings

No blocking correctness finding for the declared `^22.13.0 || >=23.5.0` policy. The evaluator admits stable 22.13+ within major 22 and stable 23.5+, excludes unsupported gaps and prereleases, and accepts valid build suffixes. It rejects leading-zero, partial, trailing-newline, malformed and unsafe numeric version strings. Unknown future engine grammar fails closed rather than silently diverging. Invalid declaration now names Prisma AIRS Harness.

An independent differential check compared 729 canonical stable, build-metadata and rc versions against npm's already-installed semver 7.8.5. All results matched. This check introduced no dependencies or downloads and is explicitly simulation on Node 22.23.2; it does not claim actual Node 18 execution. Evidence is `node-semver-differential.json`.

The guard runs before native/product resolution, completion child invocation, migration inspection, or legacy warning. `runPrismaCli` is independently guarded, including the private `managed-cli/airs-cli` wrapper and its Windows command shim. Import-time modules use built-in Node APIs and ESM/optional-chaining syntax compatible with Node 18 on source inspection; they do not import the bundled CLI application before the guard. The published package's `lib/` files list automatically includes the new guard.

Unsupported-runtime output is bounded and JSON-escapes the observed version; the validated range cannot contain control characters. Rejection sets exitCode 1 and returns before child execution. Supported branches preserve argument boundaries and exit status. The duplicate guard on `airs cli` is negligible deterministic work and necessary to keep the direct managed wrapper independently protected.

## Test review and remaining limits

The new entrypoint matrix explicitly proves no child marker for native, managed CLI, completion, direct managed wrapper and legacy launcher on unsupported versions. Supported native/managed paths prove argument and exit forwarding. Existing completion tests retain supported completion behavior.

All newly added runtime tests were initially Windows-skipped because their fixture writes a POSIX executable. The guard itself is platform-neutral; recommended improvement is a portable subprocess test importing `checkNodeRuntime` directly for version/manifest boundaries, while keeping real POSIX child execution tests platform-specific. This observation was sent to root and author. A POSIX result must not be reported as Windows execution.

Actual Node 18 is unavailable in the current environment. Simulating `process.versions.node` verifies policy and guard ordering; source inspection supports compatibility but cannot replace runtime execution. Keep this explicit in receipts and the focus review. No production version override is introduced.
