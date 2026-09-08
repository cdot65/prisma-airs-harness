# Why the restored Mac build still took 42 minutes

The cache worked for third-party dependencies, but Cargo rebuilt the workspace.
Completed f964 run 34265276427 restored the c10 cache (2,137,173,305 bytes) in 67 seconds,
then compiled 131 codex-* workspace packages and reported 42m 16s for the CLI. The
small native-store fixture took another 8.30 seconds. Cache saving took 59 seconds.
The preceding c10 run 34259924957 restored 1,775,315,296 bytes in 71 seconds and also
compiled 131 workspace packages; its CLI took 47m 16s. Both compilations succeeded;
the later acceptance assertions failed. The three inherited warnings were not
compile errors. The original job logs remain private; [structured evidence](report.json)
retains hashes and selected nonsecret timing/cache records.

The local 154 comparison is not equivalent. Its 560-second report records only
CLI library 58.08 seconds, CLI binary unit 501.22 seconds and a 0.01-second build script
as nonzero, among 1399 timing units. The rest were reused/zero-duration entries.
The binary unit includes compilation/code generation/linking; it is not proof
that the linker alone took 501 seconds. Local 154 also reduced CLI optimization to 1;
the old Mac workflow used release optimization 3. Hardware, target and rebuilt
scope differ. No measured speedup can be derived from 42m 16s versus 9m 20s.

## Evidence versus explanation

Cargo uses source/output timestamps and dependency fingerprints when deciding
whether workspace/path units need rebuilding. Registry dependencies deliberately
avoid normal source-mtime checks. Consequently a fresh checkout followed by
restoring older target outputs can leave dependency reuse intact while rebuilding
workspace code. That mechanism is documented; it is a strong explanation consistent
with these logs, **not a confirmed per-unit cause**. No fingerprint reason trace
was retained, and unchanged utility crates also recompiled. Profile, features,
toolchain and dependency changes can invalidate units too. [Cargo fingerprint documentation](https://doc.rust-lang.org/stable/nightly-rustc/cargo/core/compiler/fingerprint/index.html)

The workflow uses CARGO_INCREMENTAL=0, disabling rustc's incremental workspace
state; restoring target artifacts is a different cache layer. Its LTO=false still
permits local thin LTO, while LTO=off disables it. Current private candidates reduce
CLI optimization to 1 while retaining other packages' release settings. These are
compiler settings with runtime/size tradeoffs, not automatic speed guarantees.
[Cargo profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html)

## Safest next steps and validation

1. **Use the already-implemented immutable build/acceptance split.** Fixture-only
   corrections should reuse the exact signed artifact and rerun acceptance, with
   source/hash/run/attempt recorded. This removes the compilation stage from those
   retries without changing runtime behavior. Validate through the no-build job
   and ensure its steps never invoke Rust compilation.
2. **Instrument the next scheduled build, not an extra benchmark now.** Add the
   documented fingerprint info log alongside existing Cargo timings, cache key
   and profile. Record peak memory and CPU/resource observations before adjusting
   jobs. Compare why the 131 workspace units were considered dirty. Cargo recommends
   `CARGO_LOG=cargo::core::compiler::fingerprint=info` for rebuild diagnosis.
   [Cargo FAQ](https://doc.rust-lang.org/cargo/faq.html#why-is-cargo-rebuilding-my-code)
3. **Prefer a persistent Apple Silicon workspace once the owner supplies access.**
   Confirm SSH identity, architecture, disk and toolchain first. Keep unchanged
   sources and build outputs intact between normal Git updates; serialize that
   build directory. Evaluate an unchanged rebuild and one small CLI edit using the
   same source/profile, then compare compile counts, total time, memory and full
   artifact acceptance. No manual timestamp rewriting, new paid runner, Intel
   target, or predicted speedup is justified by this review.
4. **If hosted fresh runners remain necessary, evaluate sccache for workspace
   libraries in a future controlled candidate.** Cargo documents it as a shared
   compiler cache, but sccache cannot cache Rust crates invoking the system linker
   (including binaries) and requires incremental compilation disabled. It therefore
   cannot erase the final CLI binary-unit cost. Check hit/miss and noncacheable
   reasons, total time and matching acceptance outcomes; consider build scripts
   and procedural-macro caveats before adoption. [Cargo cache guide](https://doc.rust-lang.org/cargo/reference/build-cache.html),
   [sccache Rust limitations](https://github.com/mozilla/sccache/blob/main/docs/Rust.md)

The current cache key includes the entire workflow file and tooling Git SHA,
with compatibility and broad ARM64 fallback keys. A YAML-only edit therefore
changes the primary cache identity, but fallback restored useful artifacts in
both measured runs. Narrowing that key to genuine build inputs is a possible
later cleanup, not sufficient evidence that it will prevent source-mtime rebuilds.
Do not remove the build-profile/toolchain/lockfile compatibility boundary.

No active build, Rust process, runtime source, CI file, credential or infrastructure
was changed during this investigation. No OOM cause, hardware bottleneck or
numeric performance improvement is asserted without measurement.
