# Upstream and fork boundary

- Product: Prisma AIRS Terminal; executable: `airs-terminal`.
- Upstream: https://github.com/openai/codex.git (git remote `upstream`).
- Baseline tag: `rust-v0.153.4`.
- Baseline commit: `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`.
- Fork started: 2026-09-07.
- Rust toolchain: 1.95.0, as pinned by upstream.
- Runtime dependency on PAH: none.

Keep the upstream commit ancestry and internal crate names. Separate behavior
changes from mechanical branding. Before updating upstream, review reintroduced
service endpoints, login behavior, application-home resolution, request model
serialization, auth handling and platform packaging. Run the affected integration
checks against the installed executable, including real gateway/tool traffic.

## Initial delta ledger

1. Reconcile the upstream release tag's Cargo.lock workspace versions from
   0.0.0 to 0.153.4; external dependency versions and checksums remain unchanged.
2. Add independent project documentation and preserve the upstream README.
3. Optional gateway routing at the Responses wire boundary, with local model
   metadata preserved and explicit qualified routes validated.
4. Independent product entry point, private application home, setup and startup
   guards. Product and upstream binaries share source without an external runtime.
5. Executable protocol tests and UI snapshots. Remaining release gates and test
   limitations are recorded in IMPLEMENTATION.md.

Do not remove copyright or license attribution to achieve branding consistency.
Do not add PAH libraries, RPC transports, services or proxies to this project.
