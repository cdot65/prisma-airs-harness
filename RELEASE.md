# 0.1.0-alpha.5 — interactive model-switch repair

Owner E2E testing found a real alpha.4 integration failure: selecting GPT-4.1
through `/model` fabricated a default reasoning effort from an empty capability
list. GPT-4.1 rejected the resulting `reasoning.effort`, and the picker could remain
open. The earlier provisional 9/10 assessment is withdrawn. Startup-route tests
had missed this interactive path; their passing receipts remain historical evidence.

## Changed behavior

- Gateway model selection clears reasoning effort when no levels are advertised
  and dismisses the picker. Switching back to the gateway default also works.
- The inference request boundary omits effort values unsupported by the selected
  gateway capability entry, including stale persisted settings. Explicitly
  advertised reasoning levels remain available. Upstream non-gateway behavior is
  preserved; no provider-specific model names are hard-coded in the terminal.
- Runtime instructions distinguish remote inference from local tools, requested
  routes from verified backend identity, MCP tools from resources, and skills
  from executable integrations. A deterministic, bounded summary of enabled MCP
  server names and tool allowlists contains no URLs or credentials. It is labeled
  as configuration, not a live health check.
- Existing environments receive this runtime context without rewriting their
  pinned model catalogs, credentials or session history.

## Acceptance

The release requires the actual PTY path default → explicit → default, not only
separate process launches with `-m`. Deterministic tests inspect request payloads
for routing and unsupported effort, and preserve advertised effort in a separate
positive test. The live script asks the owner's model/codebase/MCP questions,
changes calculator code, independently checks tests and arithmetic, and invokes
MCP scans across model changes. Natural-language self-descriptions are retained
for human review and are not authoritative backend identity metadata.

The exact optimized binary passed 20 executable fixtures; the live seven-turn
interactive replay completed both model switches, five calculator tests and three
real scans. A separate two-route agent probe passed local skill/edit/test/MCP work
and resume. Alpha.5 also resumed an isolated alpha.4 session after reproducing its
live unsupported-effort failure. Installation preserved all ten fingerprinted
owner configuration/history files and retained alpha.4 for rollback.

Scoped source checks passed: 23 core tests, 4 home-directory tests and 4,090 TUI
tests (6 skips; one existing timing test passed on retry). Scoped Clippy passed.
The first final replay exposed raw-keyboard paste timing in the test driver; it
left a prompt in the composer. Explicit bracketed paste fixed the driver, and all
20 fixtures plus the full live replay passed again with the same binary.

Runtime descriptions improved on the reported model/MCP errors. Generated prose
still includes generalizations and is not authoritative routing, repository or
connection-health metadata; use configuration, source files and `/mcp` for those.

Final artifact evidence is recorded in `VALIDATION.json` and the release's
`RELEASE-VERIFICATION.json`. The earlier alpha.4 notes and receipts are retained
under `validation/2026-09-07/`; they do not certify this binary.

```sh
python3 scripts/validate_live_model_switch.py \
  --binary "$HOME/.local/bin/airs-terminal" \
  --gateway-url https://airs.cdot.io/v1 \
  --credential-file "$HOME/workspace.txt" \
  --mcp-url https://mcp-airs.cdot.io/ws-develo-71f8d8/airs-terminal-security/mcp \
  --mcp-credential-file "$HOME/.airs-terminal/credentials/mcp-workspace.key" \
  --output-directory /tmp/airs-terminal-new-interactive-check
```

The output directory must not exist. The probe uses isolated state and leaves the
owner's active environment and review project alone. After an installed upgrade,
exit the old running process and start the new binary; use `resume` to reopen the
existing session. A process already running retains the old executable code.

## Gateway compatibility transforms

The deployed AIRS 2.20.0 implementation was inspected in response to the owner's
suggestion to remove unsupported fields centrally. Its `default.regexReplace`
guardrail changes message text, not root JSON parameters. Its
`default.requestParametersCheck` validates/rejects parameters. The
`default.webhook` guardrail accepts transformed request JSON. Further inspection
also verified native target `drop_params` for simple key removal, before guardrail
context creation; four local transform checks passed. Prefer that mechanism for
model-specific removal and reserve a webhook for more complex transformations.

A future compatibility policy should match resolved provider/model capabilities,
remove only known unsupported fields, preserve supported reasoning, and retain
mandatory input/output security scans. Never use regex against arbitrary JSON or
silently strip reasoning from every model. This terminal repair itself does not
change the live gateway routing configuration or the scanner deployment.

## Remaining release boundaries

This remains the authorized standalone Linux workspace-key pilot. Keycloak login
and refresh, individual-user authorization/audit, macOS, native CI builds/signing,
and the complete upstream V8-dependent suite remain unfinished. The unavailable
V8 musl archive prevents full workspace validation on this host. Existing scanner
image findings (20 unfixed medium/low, no critical/high) and the broader workspace
scc warning outside the Linux CLI graph remain documented in the earlier audit.
Owner hands-on acceptance is required; a self-assigned score does not replace it.

## Publication verification

[Independent Ubuntu release CI](https://github.com/cdot65/prisma-airs-terminal/actions/runs/34111335276)
passed against immutable tag `airs-terminal-v0.1.0-alpha.5`: 20 packaged executable
fixtures, four scanner tests and zero production npm findings. The downloaded
archive's binary and source hashes match the installed binary and release
provenance. `RELEASE-VERIFICATION.json` and its checksum were uploaded, downloaded
again and verified. Owner hands-on revalidation remains pending.
