# Linux pilot administration

The local terminal has no management-plane SDK or PAH runtime dependency. These
records describe the owner's September 7 pilot; they are not an automatic
migration. Provision equivalent resources in another workspace, then validate both
routing branches and independent MCP authorization before distributing a key.

## Runtime contract

Inference is `https://airs.cdot.io/v1/responses`. An environment-backed key uses
`x-portkey-api-key`; file/OS-store credential helpers use the gateway's supported
`Authorization: Bearer` transport. The client never sends both. Remote MCP uses
its separately bound `x-portkey-api-key` credential. Neither transport is an
individual-user identity in workspace-key mode.

The configured workspace is `2f2ba0ed-7797-441e-9add-eec2c60b9723`, slug
`ws-develo-71f8d8`. The terminal inference key is bound to routing configuration
`d68e3bf0-e1c7-4825-bea9-a3351d2d9e20` (`pc-prisma-7d87b5`) with
`allow_config_override=false`. Only its needed inference/agent/logging scopes are
present. Do not give ordinary terminal users management credentials or provider keys.

`pilot-routing.json` is the live, non-secret routing body. The gateway-default
target selects provider `@openai` and model `gpt-4.1` inside the gateway. The client
omits root `model`. Qualified explicit routes use the passthrough branch under
the same synchronous input/output guardrails. The conditional expression accounts
for this gateway's query engine coercing an absent property to the string
`undefined`; null/empty values are directed to the explicit branch and rejected,
not silently replaced. The terminal itself emits neither null nor empty models.

## Mandatory scanner policy

Use the separate profile `Prisma AIRS Terminal`, ID
`11d6d866-a363-44a3-84ec-d161634cc017`. It was cloned from the existing policy so
that other applications keep their original behavior. Prompt injection, the
selected DLP detectors and malicious URLs block; scanner timeout is configured to
block. The original `Claude Code` profile was preserved.

Guardrail `5a4c6cb0-bdd6-4784-81d4-95920b82330c` (`pg-prisma-ff3021`) invokes
`panw-prisma-airs.intercept` with this profile, `scan_scope=last_message`,
`strip_scaffolding=false` and **`failOnError=true` inside the check parameters**.
The guardrail has synchronous, sequential checks and deny-on-failure enabled.
Both routing branches reference it for input and output. The latest restored
version is `42f72de7-50a0-4326-bf9e-9a1f0dffb2bf`.

Acceptance includes benign input/output scans, denied prompt injection, synthetic
DLP and malicious URLs, and an invalid required profile that fails closed. The
invalid-profile probe was temporary and the valid profile was restored. Config,
provider-key, provider-header and guardrail-header bypass attempts are retained
in the release's redacted evidence. A successful `/v1/health` response alone does
not establish policy enforcement.

## Independent MCP configuration

Endpoint:
`https://mcp-airs.cdot.io/ws-develo-71f8d8/airs-terminal-security/mcp`.
Integration `a4efedaa-4137-4d49-85a9-6fc8776318a1` points to
`https://mcp-airs.cdot.io/terminal-scanner/mcp` using HTTP transport and a dedicated
server-side `x-airs-terminal-mcp-key`. No caller headers pass through. The stateless
[scanner adapter](../mcp-scanner/README.md) calls the official REST scanner API
with a fixed server-side profile. Its scanner key stays in a Kubernetes Secret.
The vendor's hosted MCP endpoint returned intermittent HTTP 500 even when called
directly; disabling pooling alone did not resolve that failure.

Server binding `71f3efa5-9a44-4159-9b40-89135ecb9f8e` permits only
`pan_inline_scan`. `pan_batch_scan` and `pan_get_scan_results` are disabled on the
server. The separately provisioned workspace credential has only `mcp.invoke`;
it cannot call inference. User configuration additionally limits the visible tool
catalog. Do not substitute client-side filtering for server authorization.

The installed AIRS gateway image is 2.20.0, with two replicas. Infrastructure lives
in `cdot65/talos-cluster`: DNS correction `533b30a`, supported MCP pooling isolation
`31fadca`, and failure-evidence clarification `f150717`. Helm revision 32 retains
`MCP_POOL_ENABLED=false`, giving each session its own upstream connection. This
has connection overhead and does not guarantee upstream scanner availability.
The infrastructure README records application and rollback commands. Revision 32 also adds the exact backend hostname `mcp-airs.cdot.io` to
`TRUSTED_CUSTOM_HOSTS`, preserving the existing allowlist and SSRF enforcement.
The scanner runs two replicas of the digest-pinned Harbor image documented in
RELEASE.md. Inference routing is preserved.

## Management and validation

The operator's Prisma AIRS CLI 3.3.0 reads its existing private configuration and
`airs doctor` verifies management/scanner/gateway reachability. Its current
`airs aigateway workspace` commands expose workspace administration; catalog,
config, key and MCP changes for this pilot used the owner-maintained
`@cdot65/prisma-airs-sdk` 0.21.0 management client. Do not assume CLI subcommands exist
for every SDK resource or save complete credential-bearing API responses in Git.

Run the source's `scripts/validate_live_gateway.py` and
`scripts/validate_live_agent.py` with explicit private key-file references. The
former checks raw routing/policy/authorization; the latter verifies actual local
file effects, a skill, tests, remote scanning and continuation. Retain failures
alongside successes. Required MCP startup fails after bounded initialization
retries; do not blindly replay an uncertain remote mutation.

Keycloak public-client login, refresh and distinct-user audit evidence remain
unfinished under the owner's explicit workspace-key fallback. Do not label this
pilot a completed team identity implementation.

Private pre-cutover MCP and Helm configurations are preserved by the operator in
`~/.local/share/airs-terminal/operator-recovery/2026-09-07/` with directory mode
0700 and file mode 0600. These contain credentials and must never be attached to
issues or copied into this repository. The rejected untagged scanner image
candidates were removed from Harbor; the final digest remains available.
