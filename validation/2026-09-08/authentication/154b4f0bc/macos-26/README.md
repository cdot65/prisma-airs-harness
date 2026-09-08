# Preserved-artifact macOS 26 acceptance

[Run 34278615096](https://github.com/cdot65/airs-harness/actions/runs/34278615096) passed on hosted macOS 26.6.2 ARM64. It downloaded immutable executable artifact `10076349913` from compilation run `34272843528` and verified the archive, source, CLI and native fixture hashes before execution. No Rust compilation was performed.

The runtime remains `154b4f0bcef7528844e85177d7e4dce611982044`; validation tooling is separately recorded as `67976dfafdbe017918c8ab46db34a8b50eefc63f`. The original compilation attempt and builder SHA remain preserved in `artifact-selection.json`.

- Native suite: 37 passed, 2 explicit skips out of 39 methods. The npm-managed test ran in the installed suite; the Linux recovery case is inapplicable on this runner.
- Installed npm suite: 37 passed, 1 Linux-only skip out of 38 methods.
- Native and installed CLI Keychain lifecycles passed, including private new-process readback, the authenticated local tool loop, plaintext absence and logout rejection.
- The native credential fixture passed all 7 separate-process phases, including legacy/v2 migration and the 16 KiB workspace-key case. Pinned Prisma AIRS CLI 5.2.0 offline contracts and corpus generation passed.

The retained logs and fixture JSON files are unchanged downloaded evidence. The separately assembled run receipt records API provenance, digest-verified evidence artifact identity and exact pass/skip counts. Original downloaded evidence remains at `/var/tmp/airs-macos-full-34278615096/acceptance-evidence` on the operator host and in GitHub artifact `10076957421`.

This establishes cross-version hosted acceptance using the same executable bytes without another compile. It does not establish owner-Mac acceptance, Developer ID signing, notarization, live gateway use from macOS, or publication. The private alpha.9 candidate is distinguished by its source and binary hash from the earlier published alpha.9 version.
