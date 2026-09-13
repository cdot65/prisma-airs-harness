# Upstream and fork boundary

- Product: Prisma AIRS Harness; executable: `airs-harness`.
- Upstream: https://github.com/openai/codex.git (git remote `upstream`).
- Baseline tag: `rust-v0.153.4`.
- Baseline commit: `3d2ee51ca2d5db578f328aa75e20aa22c0197c9a`.
- Fork started: 2026-09-07.
- Integrated stable source: `rust-v0.154.0`, commit
  `6b9826e3aa83b1a5947db50f4332cb9c65f1b340` (alpha.12 candidate, not published).
- Rust toolchain: 1.95.0, as pinned by upstream.
- Runtime dependency on PAH: none.

Keep the upstream commit ancestry and internal crate names. Separate behavior
changes from mechanical branding. Before updating upstream, review reintroduced
service endpoints, login behavior, application-home resolution, request model
serialization, auth handling and platform packaging. Run the affected integration
checks against the installed executable, including real gateway/tool traffic.

The 0.154 merge decisions, endpoint inventory and acceptance evidence are in
[`validation/2026-09-13/upstream-0.154/`](validation/2026-09-13/upstream-0.154/).
The fixed evaluation is run with `python3 scripts/evaluate_airs_upstream.py LEDGER.json`.
It requires at least 90/100 plus every mandatory gate, artifact-bound evidence,
independent review and owner acceptance. An absent gate cannot be averaged away.
Worktrees and inline questions remain opt-in; unsupported service features are
pinned off at runtime for existing environments as well as new setup output.

## Delta ledger

1. Reconcile the upstream release tag's Cargo.lock workspace versions from
   0.0.0 to 0.153.4; external dependencies were initially preserved; later security patches are listed below.
2. Add independent project documentation and preserve the upstream README.
3. Optional gateway routing at the Responses wire boundary, with local model
   metadata preserved and explicit qualified routes validated.
4. Independent product entry point, private application home, setup and startup
   guards. Product and upstream binaries share source without an external runtime.
5. Executable protocol tests and UI snapshots. Historical prototype limitations
   are in IMPLEMENTATION.md; current evidence is in RELEASE.md and VALIDATION.json.
6. Linux/musl compatibility: preserve the existing sandbox restrictions while
   recognizing Bubblewrap's /proc diagnostics; pin protected directory inodes
   with O_PATH and use actual helper binaries in musl test fixtures.
7. Named environment UUIDs, typed versioned registry, private atomic writes and
   process-stable home selection. Credentials and history are bound to gateway,
   credential identity and capability revision; no PAH or OpenAI account is needed.
8. Explicit OS-store/file/environment credential sources and separate AIRS MCP
   credential helpers. Fail closed on identity/destination changes and repository
   overrides. The existing HTTP helper/redirect/retry implementations are reused.
9. Product CLI/TUI presentation, setup/status/doctor, environment-aware resume
   hints and logout that survives atomic executable replacement. Hosted product
   features are explicitly disabled in generated pilot configuration.
10. Reproducible executable, live-agent, raw gateway and MCP-session probes;
    package script includes locked dependency/license inventory and provenance.

The alpha.4 work is split into reviewable environment, authentication, boundary,
MCP, command/UI and validation commits. Rust dependency additions use already
locked workspace dependencies; Bazel dependency metadata was refreshed without
lockfile drift for those additions. No ConfigToml schema fields were added.

The new GitHub repository is private. Inherited OpenAI Actions workflows remain
reference source and are disabled. The owned manual `airs-harness-release-check`
workflow verifies a published archive on a fresh Ubuntu runner and tests/audits
the scanner. It does not build or attest the binary. The pilot artifact is a local
optimized build with a pinned Rust toolchain, not signed CI build provenance.

Do not remove copyright or license attribution to achieve branding consistency.
Do not add PAH libraries, RPC transports, services or proxies to this project.

## Release audit follow-ups

The final audit updates quinn-proto to 0.11.15, event-listener to 5.4.2 and memmap2
to 0.9.11 for RustSec advisories; chacha20 0.10.2 and spin 0.9.9 replace yanked
versions. The remaining scc 2.4.0 advisory belongs to the broader workspace and is
absent from the Linux CLI normal/build graph. Keep that warning visible when
reviewing an upstream-wide build. Cargo and Bazel lockfiles are refreshed together.

The optional `mcp-scanner/` service uses the MCP TypeScript SDK and the owner's
Prisma AIRS SDK to expose one stateless scanning tool. It is a separately deployed
remote backend; the terminal can point at other authorized MCP endpoints and has
no PAH dependency. Its Dockerfile and pinned npm lockfile are maintained here.

## Alpha.5 owner acceptance repair

Keep the gateway-only reasoning capability checks at the request boundary and in
the model picker when merging upstream. Empty supported levels must not invent an
effort. Preserve the bounded runtime context and its separation of MCP tools from
resources. `scripts/test_airs_harness.py` now drives the actual interactive model
switch; `scripts/validate_live_model_switch.py` repeats it through AIRS with real
local work and scanner calls. The alpha.4 startup-only coverage missed this bug.

The repository URL and upstream crate names are retained for source provenance.
The npm distribution is `airs-harness`; it wraps the standalone Rust executable
and does not install or invoke an external Codex CLI. See [RENAME.md](RENAME.md).
