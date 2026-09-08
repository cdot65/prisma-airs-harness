# Apple Silicon macOS 15 candidate acceptance

[Run 34272843528](https://github.com/cdot65/airs-harness/actions/runs/34272843528) passed on hosted macOS 15.7.9. The compiled runtime is `154b4f0bcef7528844e85177d7e4dce611982044`; its build and validation tooling is `5b070fb349b5cbdecc42d49e842dcf030444625d`.

- Native executable suite: 37 passed, 2 explicit skips out of 39 methods. The skipped managed CLI test runs in the installed package suite; Linux session-bus recovery does not apply to this Mac runner.
- Installed npm suite: 37 passed, 1 Linux-only skip out of 38 methods.
- Native and installed CLI Keychain lifecycles passed, including private helper readback from a separate process, an authenticated local tool loop, logout rejection, and plaintext-state absence.
- Native storage fixture: all 7 separate-process phases passed, including legacy/v2 credentials and a 16 KiB workspace key. The pinned Prisma AIRS CLI 5.2.0 and its offline capability/corpus checks passed.

The original build evidence ZIP is retained unchanged and its GitHub artifact digest is recorded in `RUN-RECEIPT.json`. It contains the actual Cargo timing HTML, compiler output, build manifest and cache-save outcomes. Cargo reported 47m 31.3s for the CLI; the combined build/fixture/metadata step took 47m 42s. The old f964 cache was restored successfully in 48 seconds. These measurements do not establish a speed improvement: the source/profile differs from earlier builds. The recorded `lto=false` text represents the environment override; effective per-crate LTO was not independently established.

The executable ZIP, inner archive, source commit, CLI and fixture checksums were independently verified locally before artifact-only macOS 26 dispatch. No Mac executable was run on Linux. Exact immutable artifact IDs and checksums remain in the receipts; binaries are retained in GitHub artifacts and the operator's local evidence directory rather than this repository.

The displayed alpha.9 version is an unpublished candidate built from the runtime commit above. It is not the previously published alpha.9 package. This run used ad-hoc signatures and deterministic loopback gateway fixtures. It does not establish Developer ID signing, notarization, owner-Mac acceptance, or live gateway success from macOS.
