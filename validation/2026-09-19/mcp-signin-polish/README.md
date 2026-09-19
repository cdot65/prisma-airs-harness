# MCP sign-in polish — mcp.6 test release

Runtime, acceptance tooling and packaging source: `a0a1b5c8d4306e23e75816139ba8538d95da0b92`.
Published `0.1.0-alpha.22.mcp.6` under `mcp` at `https://npm.cdot.io`.
The other registry tags are unchanged. This is owner-authorized test-channel
acceptance, not production SSO or ServiceNow acceptance.

The change opens a browser for desktop MCP login, retains an explicit SSH/manual
callback path, makes long links scrollable and reports authorization, credential
storage and discovery separately. Existing conversation and environment behavior
is preserved; core and the app-server protocol are unchanged.

## Evidence

- `SPEC.json` pins all three native hashes and the source/tooling/packaging commits.
- `CANDIDATE-VERIFIED.json` and `REGISTRY-VERIFIED.json` bind the retained per-platform
  receipts to exact installed bytes. Mac signature checks include notarization.
- Candidate and anonymous registry checks cover install, onboarding, terminals,
  installed regressions, MCP manager, diagnostics, bundled CLI, upgrades and command
  output. Required native MCP persistence, reuse and logout executed on each host.
- `extra-candidate` retains two renewal cycles for inference and MCP and three
  synthetic read-only turns per platform, plus upgrades from onboarding.4.
  The ordinary upgrade gate checks mcp.5.
- `PUBLICATION.json` records all four registry integrity checks and tag preservation.
- `LOCAL-CHECKS.json` records the Rust, snapshot, packaging and development fixture
  checks, including the unchanged temporary-filename assertion and its passing rerun.

The five-minute expiry test passed during development, before the final version
stamp and UI wrapping; it was not repeated against each packaged binary. The
Linux browser-launch fixture executed; the real Apple Silicon desktop browser/SSO
check is the owner's next attended step. Synthetic tool calls do not establish a
ServiceNow grant. No production identity was used for these release checks.

Builds: Forgejo UI runs 240 (x64), 241 (ARM64), 242 (Mac build/acceptance),
243 (signed/notarized Mac package). Artifact downloads use the corresponding
internal run IDs 3671–3674, not those UI indices.
