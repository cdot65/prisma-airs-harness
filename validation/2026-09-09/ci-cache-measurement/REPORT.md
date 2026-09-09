# Scoped native identity cache comparison

Both authorized debug identity runs passed on Linux and Apple Silicon. This
validates the corrected action policy, Cargo download/output reuse and working
GitHub sccache reads. It does not measure a full optimized release build.

| Hosted job | First run | Warm run | Identity test/build step |
| --- | ---: | ---: | ---: |
| Ubuntu 24.04 | 274 seconds | 94 seconds | 156 → 6 seconds |
| macOS 15 ARM64 | 151 seconds | 93 seconds | 72 → 9 seconds |

Every run passed Linux 25/25 or Mac 29/29 scoped unit tests with no skips, plus
seven separate native credential-store process phases. These are debug identity
contracts, not installed harness, owner-device or release acceptance tests.

The first run was [34300224313](https://github.com/cdot65/airs-harness/actions/runs/34300224313),
source `33c89f403`; the warm run was
[34300606150](https://github.com/cdot65/airs-harness/actions/runs/34300606150),
source `e5b1d529`. The Git objects for `codex-rs`, identity workflow and cache
helper are identical across these revisions. Compiler/SDK/profile receipts are
also byte-identical for each platform. Main advanced only through npm workflow,
documentation and evidence changes. The warm target caches restored by the
compatible identity prefix from the first revision, then saved under the new
revision; this is not an exact primary-key hit.

Most warm compilation was avoided through Cargo target restoration. sccache
recorded two Rust hits, zero misses and zero write/read errors on each warm
platform. The first run populated 179 Mac and 238 Linux cache entries but also
reported 61 Mac and 50 Linux write errors. Logs expose no specific cause, so
these failures cannot be attributed to rate limits or quota. Backend writes
were only partially successful in that run; two warm hits do not prove all
expected entries were retained. No independent sccache-only performance gain is
claimed.

`first.json` and `warm.json` retain exact job durations, test summaries, cache
keys, counters, critical step durations and private original log hashes. The
four original evidence ZIPs were downloaded by immutable artifact ID and their
SHA256 matched GitHub's recorded digest before parsing. They retain original
compiler identity, sccache counters and native store receipts. Raw job logs stay
outside the repository. `comparison.json` binds the unchanged source objects.

The exact-pinned action policy correction and previous startup failures remain
in the sibling `ci-cache-startup` record. No active release build was cancelled,
no Windows build was dispatched, and these successful debug jobs do not close
release, signing or owner-device gates.
