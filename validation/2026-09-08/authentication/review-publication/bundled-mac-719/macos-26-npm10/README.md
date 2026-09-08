# Hosted macOS 26 bundled npm 10 acceptance

[Run 34285996453](https://github.com/cdot65/airs-harness/actions/runs/34285996453) passed on Apple Silicon macOS 26.7.9. Acceptance tooling `5644244f9b2b5a09af64600843b61a78cfcb0543` reused the unchanged source154 executables from artifact `10076349913`; no Rust compilation occurred. The original fixture failure remains recorded in [the preflight evidence](../../bundled-mac-cebe-preflight/README.md).

A fresh ordinary scoped-name install with npm 10.0.2 and Node 22.23.2 used four staged loopback registry requests, zero unexpected requests, and zero public dependency redirects. The installed bundle verified 68 exact-version packages, 3,080 files and 66 licenses. Its two Sharp native payload packages are Apple Silicon only. Four inventoried optional source files omitted by npm explain the 3,084 staged versus 3,080 installed count.

The preserved native suite passed 37 of 39 methods with two explicit skips; the installed suite passed 37 of 38 with one Linux-only skip. Both native and installed CLI Keychain lifecycles passed, as did seven separate-process native-store phases. Managed CLI 5.2.0 passed the configuration/doctor contract, 20 capability help checks and actual PDF/PNG/JPEG/SVG/DOCX generation. Native dependencies were exercised by generation; live document scanning was not performed.

The original acceptance evidence ZIP is retained and independently matches its GitHub digest. All 13 packaging-source file hashes were independently compared with the recorded packaging commit. The installed binary hash matches the independently retained original native artifact. The larger private candidate ZIP is referenced by GitHub metadata and was not independently downloaded in this review.

This is private hosted acceptance with ad-hoc signatures and deterministic loopback servers. It does not establish owner-device behavior, Developer ID signing/notarization, live gateway access, GitHub package authentication, teammate permissions, publication or full release readiness.

The newer workflow also retained named npm install diagnostics in the evidence ZIP. Its INSTALL-NETWORK and INSTALL-VERIFICATION values were independently reconciled with npm-install.json. This confirms collection on success; no failure was manufactured for this run.
