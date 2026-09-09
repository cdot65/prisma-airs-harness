# Owned CI lanes

This change improves future runs. It does not change an active build, publish an
artifact, or establish a measured speedup. Windows remains deferred by the owner;
macOS jobs require Apple Silicon before installing build tools. Inherited
upstream workflows remain disabled.

| Lane | Trigger and scope | Compilation |
| --- | --- | --- |
| npm launcher | PR/main changes to launcher, npm packaging, bundling and CLI contract files | None; Node/Python tests and the real pinned CLI contract |
| Native identity | Any `codex-rs/**` PR/main change, credential fixture or cache helper change | Debug identity/keyring contract tests and native store fixture only |
| macOS candidate | Explicit dispatch; cheap preflight precedes build | Existing release profile with CLI opt-level 1; dependencies retain their configured release settings |
| Preserved candidate acceptance | Exact immutable artifact selection | No Rust compilation; installed package, native store and application acceptance |

The identity trigger deliberately includes shared Rust dependencies. It is not
a complete reverse-dependency test selector or full application PR acceptance.
Runtime/core changes still require the appropriate scoped and integration tests
from `AGENTS.md` and the authentication plan. Do not make path-filtered lanes the
sole required check for unrelated changes. No redundant full `cargo check` plus
Clippy build is added here.

## Three distinct caches

1. Cargo registry/git downloads restore independently, then `cargo fetch
   --locked` validates/fills them. Successful downloads are saved before native
   compilation so a later failure cannot discard that work.
2. Cargo target outputs retain complete compiler artifacts. Mac v2 cache keys
   include actual compiler, SDK/Xcode, runner image/architecture, Cargo config,
   lockfile and relevant profile/flag values; the suffix identifies runtime
   source, not unrelated packaging tooling. The old path list and a legacy v1
   fallback remain for migration. A legacy restore is explicitly recorded and
   relies on Cargo fingerprint validation; it is not an identity-matched build.
3. Pinned sccache 0.16.0 uses the GitHub Actions backend for reusable compiler
   work, with incremental compilation disabled. The setup action is pinned to
   the peeled v0.0.11 commit
   `fc920bf0ec8de6ee65d409111f7ec508035751ba`.

Compiled release caches are saved only after immutable executable upload.
Cache-save failure is visible but cannot discard a completed candidate. Source
cache entries are content keyed; target entries may grow per source revision,
so storage eviction and restore time must be monitored. Concurrency groups keep
existing runs active and serialize runs in the same lane/ref. They do not
deduplicate queued builds by runtime SHA.

sccache cannot cache Rust outputs that invoke the system linker, including the
final binary and procedural macro libraries. It also documents limitations for
procedural macros that read files themselves. A version change in the shared
`utils/home-dir` crate can invalidate many downstream crates; caching cannot
make changed code free to compile. See the pinned [Rust cache limitations](https://github.com/mozilla/sccache/blob/v0.16.0/docs/Rust.md)
and [GitHub backend behavior](https://github.com/mozilla/sccache/blob/v0.16.0/docs/GHA.md).

## Measurements and rollout

Retain `cache-identity.json`, `sccache-stats.json`, cache outcomes, Cargo timing
HTML and step durations. Compare cold and warm runs with the same runtime,
toolchain, runner image and profile, then a small CLI-only change. Report cache
hits/misses, non-cacheable compilations, download/restore/save time, compilation
and linking time separately. Missing counters are recorded as unavailable,
never as zero misses or successful reuse.

The npm lane runs pinned actionlint 1.7.12 for these three workflows using an
already approved installation action. Semantic linting does not validate a
repository action allowlist: review that policy separately before introducing a
new external action pin. Local cache-key and workflow-boundary tests do not
validate GitHub backend access. Run the existing Mac workflow with `preflight_only=true` after merging
this tooling, without dispatching a native build. The next authorized native
run establishes actual sccache/backend behavior; retain failures and counters
before claiming an improvement. No current run should be cancelled for this
rollout, and no Windows run is authorized by this change.
