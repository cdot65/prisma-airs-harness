# Hosted macOS 26 bundled npm 12 acceptance

[Run 34284911571](https://github.com/cdot65/airs-harness/actions/runs/34284911571) passed on Apple Silicon macOS 26.7.9. Acceptance tooling `71902b53362833125cf4f0b3d10d257d40c8f7c7` reused the unchanged source154 executables from artifact `10076349913`; no Rust compilation occurred. The original fixture failure remains recorded in [the preflight evidence](../../bundled-mac-cebe-preflight/README.md).

A fresh ordinary scoped-name install with npm 12.0.2 and Node 22.23.2 used four staged loopback registry requests, zero unexpected requests, and zero public dependency redirects. The installed bundle verified 68 exact-version packages, 3,080 files and 66 licenses. Its two Sharp native payload packages are Apple Silicon only. Four inventoried optional source files omitted by npm explain the 3,084 staged versus 3,080 installed count.

The preserved native suite passed 37 of 39 methods with two explicit skips; the installed suite passed 37 of 38 with one Linux-only skip. Both native and installed CLI Keychain lifecycles passed, as did seven separate-process native-store phases. Managed CLI 5.2.0 passed the configuration/doctor contract, 20 capability help checks and actual PDF/PNG/JPEG/SVG/DOCX generation. Native dependencies were exercised by generation; live document scanning was not performed.

The original acceptance evidence ZIP is retained and independently matches its GitHub digest. All 13 packaging-source file hashes were independently compared with the recorded packaging commit. The installed binary hash matches the independently retained original native artifact. The larger private candidate ZIP is referenced by GitHub metadata and was not independently downloaded in this review.

This is private hosted acceptance with ad-hoc signatures and deterministic loopback servers. It does not establish owner-device behavior, Developer ID signing/notarization, live gateway access, GitHub package authentication, teammate permissions, publication or full release readiness.
