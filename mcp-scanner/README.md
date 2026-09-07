# AIRS Harness stateless scanner MCP

An optional remote backend exposing only `pan_inline_scan`, using the official MCP
TypeScript SDK and the owner-maintained `@cdot65/prisma-airs-sdk`. It calls the
official Prisma AIRS REST scanner API. It has no PAH dependency and is deployed
separately from the local Rust terminal.

The hosted vendor MCP endpoint intermittently returned HTTP 500 during direct
initialization. Gateway connection isolation did not fully resolve it. This
adapter uses stateless streamable HTTP and JSON responses, avoiding that upstream
MCP session service. It never retries an uncertain scan result automatically.

## Build and checks

```sh
cd mcp-scanner
npm ci --ignore-scripts
npm test
npm audit --omit=dev --registry=https://registry.npmjs.org
```

Node 22 is the runtime. The public npm lockfile pins the MCP SDK 1.30.0, Prisma AIRS
SDK 0.21.0, Express 5.2.1 and Zod 3.25.76. Four test groups exercise the real MCP
HTTP client/server contract: discovery and scanning, authentication, invalid and
failed scan outcomes, and concurrent stateless clients. Failure tests check that
errors cannot become an allow decision or reflect credentials.

From the repository root, `docker build -f mcp-scanner/Dockerfile .` builds the
service. Both build and final base images are digest-pinned. The production
runtime is distroless Node 22 Debian 13, UID/GID 10001, with no package manager or
shell. The pilot image was assembled using `crane append`/`crane mutate` from the
same pinned final base and locked production install after TypeScript compilation;
this was a local build, not signed CI provenance. The published layer also includes
a CycloneDX npm inventory and Apache notices. The Dockerfile is the maintained
rebuild recipe, and a rebuild is not claimed to be byte-for-byte identical.

## Authentication and limits

Inject `AIRS_SCANNER_API_KEY`, `AIRS_MCP_GATEWAY_KEY` and `AIRS_SCANNER_PROFILE`
through the server's secret store. Never put them in an image, client config or
source. The fixed REST base is
`https://service.api.aisecurity.paloaltonetworks.com`.

`POST /terminal-scanner/mcp` requires an exact, constant-time checked
`x-airs-terminal-mcp-key`. The gateway injects that independent backend key after
authorizing the caller's MCP-only workspace credential. No caller headers are
passed through. The profile cannot be selected by tool arguments. `/health` is an
unauthenticated readiness endpoint, not proof of scanner authorization.

JSON input is capped at 1 MiB; prompt/response fields at 200,000 characters each.
Only strict known fields are accepted. The scanner timeout is 10 seconds, with
zero SDK retries and cancellation propagation. HTTP request and graceful shutdown
timeouts are bounded. Missing scan IDs, scan errors/timeouts and unknown decisions
return an MCP error, never an allow result. The SDK error is replaced with a
bounded generic message to avoid reflecting request or credential data.

## Deployed pilot

Harbor artifact:
`registry.cdot.io/airs-terminal/scanner@sha256:1cdaca2ff3134e81ad0ac9661e41557eb49805cab4178352d10b1f46de5f97a3`.
Version tag: `0.1.0-alpha.4`. Dockerfile source is in this repository; Harbor stores
the built OCI image. The terminal executable is a separate GitHub release artifact.

Kubernetes manifests and runbook are in `cdot65/talos-cluster/airs-terminal-scanner`.
Two replicas use read-only roots, dropped capabilities, RuntimeDefault seccomp,
resource limits, readiness/liveness probes and no service-account token. A
NetworkPolicy permits Traefik ingress and DNS/HTTPS egress. HTTPS egress is not
restricted to a domain list. Secrets and a project-scoped pull-only robot are
provisioned out of band. The gateway trusts only the exact added backend hostname
in its existing SSRF allowlist.

The production npm audit reported zero findings. Trivy reported zero critical/high
and 20 unfixed medium/low findings (13/7). Retain those findings for base-image
maintenance; do not describe this image as vulnerability-free.
