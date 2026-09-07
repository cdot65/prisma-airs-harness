# 0.1.0-alpha.4 — Linux workspace-key pilot

This release targets the owner's explicit Linux workspace-key fallback.
VALIDATION.json records the packaged binary's local verification. The separate
`RELEASE-VERIFICATION.json` release asset records publication checks, independent
CI results and the final self-assessment after the archive is published. Keycloak browser/device login, refresh,
individual-user authorization and macOS acceptance remain separate milestones.
The complete original team MVP is not claimed complete.

## Acceptance

| Area | Evidence |
| --- | --- |
| Routing | Default omits root `model`; explicit sends exact `@provider/model`. Real default/explicit Responses turns complete. Null, empty and legacy alias inputs are rejected. |
| Policy | Both routes have synchronous input/output scans. Injection, synthetic DLP and malicious-URL fixtures block. An invalid mandatory profile fails closed; the valid profile was restored. |
| Authorization | Wrong key 401; MCP-only key cannot infer (403), inference-only key cannot initialize MCP (403); unauthorized config/provider-key overrides rejected. Provider/guardrail headers cannot bypass the denial fixture. |
| Local agent | Actual sandboxed read/edit, skill instructions, passing Python tests, remote scanner result and model continuation in both routing modes. |
| Environments | Independent UUID state, endpoint-bound credentials, history pinned to identity/catalog/context/MCP, foreign-session rejection, process-stable environment selection. |
| Recovery | Live cancellation stops a pending write; resume does not replay it. Live `/compact` persists compaction and retains a session marker on resume. Automatic compaction is also covered by a bounded fixture. |
| UI | AIRS branding, gateway/environment/workspace credential/permissions, model selection, startup trust and logout after atomic binary replacement. |
| Diagnostics | Local credential/config checks, actual Linux namespace probe and unauthenticated API-root health request. `doctor` distinguishes these from live inference/MCP authorization. |

Focused Rust suite: **5,151 passed**, seven existing skips. Additional CLI/MCP
suite: **972 passed**, seven skips, overlapping the CLI count. Later boundary
and MCP-branding checks also passed. The final health correction passed **695
CLI tests** after the dependency patches. The final installed binary passes **17
executable/PTY fixtures** on the current host and fresh Alpine and Debian containers.
Its normal interactive startup works without an exported key, and both live agent
routes, resume and an alpha.3/alpha.4 upgrade/rollback rehearsal pass. Counts from overlapping runs are
not added together. See VALIDATION.json for exact final-binary evidence.

The full upstream workspace suite was attempted but cannot compile V8 150.4.0 on
this musl target because its prebuilt archive returns HTTP 404. A broad core run
had 4,046 passes and 104 failures before follow-ups fixed a hook-fixture race and
installed the missing Perl prerequisite; most failures require the unavailable
host. Code mode and Node REPL are disabled in this pilot. The complete upstream
suite is not green and must be revalidated on a supported V8 build platform.

## Live deployment

- Inference: `https://airs.cdot.io/v1`; gateway image 2.20.0, two replicas.
- Workspace: `2f2ba0ed-7797-441e-9add-eec2c60b9723` (`ws-develo-71f8d8`).
- Routing config: `d68e3bf0-e1c7-4825-bea9-a3351d2d9e20` (`pc-prisma-7d87b5`).
  The terminal key is bound to this config with override disabled.
- Guardrail: `5a4c6cb0-bdd6-4784-81d4-95920b82330c` (`pg-prisma-ff3021`).
- Blocking profile: `Prisma AIRS Terminal`, `11d6d866-a363-44a3-84ec-d161634cc017`.
  The original `Claude Code` profile remains unchanged.
- Client MCP: `https://mcp-airs.cdot.io/ws-develo-71f8d8/airs-terminal-security/mcp`.
  Independent MCP-only credential and server-side `pan_inline_scan` permission.
- Scanner backend: `https://mcp-airs.cdot.io/terminal-scanner/mcp`, two replicas.
  Harbor image `registry.cdot.io/airs-terminal/scanner@sha256:1cdaca2ff3134e81ad0ac9661e41557eb49805cab4178352d10b1f46de5f97a3`.

The gateway chooses its default provider/model internally. No terminal/proxy alias
is rewritten to GPT-4.1. Explicit routes remain subject to mandatory policy. The
local budget defaults to 1,000,000 tokens; provider limits remain authoritative.

The vendor's hosted scanner MCP service intermittently returned HTTP 500, including
three of eight direct initializations in one diagnostic series. Disabling gateway
pooling alone did not fix it. The new stateless adapter uses the MCP TypeScript SDK
and the owner's Prisma AIRS SDK to call the official REST scanner, avoiding the
hosted MCP session dependency. It exposes one fixed-profile tool, bounds inputs
and timeouts, uses separate backend authentication and returns errors without an
allow verdict when scans are unavailable. It does not silently retry scan calls.

The final deployment passes 20 raw routing/policy/authorization checks and repeated,
concurrent and idle MCP checks; the exact receipts are in `validation/2026-09-07/`.
Real assembled-agent default/explicit work and resume also passed on the final
minimal runtime image. Earlier failed probes are documented as historical evidence,
not counted as successful tests. No replica-affinity change was needed.

Infrastructure is committed to `cdot65/talos-cluster`: DNS `533b30a`, pooling
isolation `31fadca`, failure clarification `f150717`, and scanner manifests plus
exact-host trust `5df8d39`. Helm revision 32 preserves the existing configuration
and adds only `mcp-airs.cdot.io` to the SSRF hostname allowlist. See
[administration/README.md](administration/README.md) for management-plane details.

## Security and supply chain

Rust audit follow-ups update quinn-proto 0.11.15, event-listener 5.4.2 and memmap2
0.9.11; chacha20 0.10.2 and spin 0.9.9 replace yanked versions. The final Cargo audit
reports zero vulnerability entries, with one remaining `scc` 2.4.0 unsoundness
warning outside the Linux CLI normal/build dependency graph. That warning remains
relevant to broader upstream workspace builds. Cargo and Bazel lockfiles agree.
References: [event-listener advisory](https://rustsec.org/advisories/RUSTSEC-2026-0221.html),
[memmap2 advisory](https://rustsec.org/advisories/RUSTSEC-2026-0186.html),
[scc advisory](https://rustsec.org/advisories/RUSTSEC-2026-0205.html).

The scanner's production npm audit reports zero findings. Its final distroless
image scan reports zero critical/high findings and 20 unfixed medium/low findings
(13/7). The initial Alpine runtime image had critical/high findings and was
replaced before final acceptance. Track base updates; this is not a claim of zero
vulnerabilities. The temporary build robot was revoked; deployment uses a separate
project-scoped pull-only robot. Scanner and management secrets are absent from
terminal configuration and release artifacts.

The private source repository is `https://github.com/cdot65/prisma-airs-terminal`.
Inherited OpenAI Actions are disabled. The owned manual release-check workflow
downloads the published archive, verifies checksums/provenance, runs executable
fixtures on a fresh Ubuntu runner, and tests/audits the scanner. Its final run
status is recorded separately from local build evidence. The Linux binary is locally built with
pinned Rust 1.95.0 and locked dependencies; no signed CI attestation is claimed.
The archive contains LICENSE/NOTICE, dependency and Rust runtime notices,
BUILD-INFO.json, VALIDATION.json and SHA-256 checksums. Internal Codex crate names
and version 0.153.4 retain the upstream update boundary.

## Reproduce acceptance

```sh
AIRS_TERMINAL_BIN=/absolute/path/to/airs-terminal \
  python3 -m unittest discover -s scripts -p 'test_airs_terminal*.py' -v

python3 scripts/validate_live_agent.py \
  --binary /absolute/path/to/airs-terminal \
  --gateway-url https://airs.cdot.io/v1 \
  --credential-file /absolute/path/to/inference-key \
  --mcp-url https://mcp-airs.cdot.io/ws-develo-71f8d8/airs-terminal-security/mcp \
  --mcp-credential-file /absolute/path/to/mcp-key \
  --output-directory /absolute/path/to/new-acceptance-directory
```

The agent probe creates a disposable workspace, skill and failing addition test,
then independently checks file effects, actual test results, MCP completion and
resume. Raw probes are `validate_live_gateway.py` and `validate_mcp_sessions.py`;
`--help` describes explicit credential-file inputs and redacted receipts. They
report transient errors without silently retrying them into a pass.

## Self-review

The earlier **8/10** assessment identified incomplete artifact delivery and unstable
hosted MCP. Improvements replaced the failing backend session dependency, hardened
the scanner image, patched audited Rust dependencies, fixed the actual health
endpoint and added clean-container and executable checks. The final assessment is **9/10 for the delivered Linux workspace-key pilot**,
provisional pending owner review. [Independent CI](https://github.com/cdot65/prisma-airs-terminal/actions/runs/34100831631) passed against the exact
published archive (17 executable fixtures, four scanner tests, zero npm audit
findings). The release's `RELEASE-VERIFICATION.json` asset and its copy in
`validation/2026-09-07/release-verification.json` record final evidence. The
immutable release source is `381266c87f9072d19a5fa85d18556b13aef74017`; this
post-publication documentation does not change the tagged archive.

A final score applies only to the authorized Linux workspace-key pilot. Keycloak,
individual-user audit, macOS, native CI builds/signing and complete upstream workspace
validation remain explicit limitations. Owner hands-on review is the final product
acceptance event, independent of this self-assessment.
