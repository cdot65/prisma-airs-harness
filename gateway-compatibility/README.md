# AIRS Terminal request compatibility

This optional gateway webhook removes root `reasoning` only for the resolved
`openai` provider, model `gpt-4.1`, and Responses API. Other providers, models,
APIs, prompt text, and nested tool-schema fields pass through unchanged. The
terminal binary and PAH are not dependencies of this service.

AIRS 2.20.0 normalizes `@provider/model` before conditional routing. The terminal
routing config therefore selects a candidate branch using `params.model ==
"gpt-4.1"`; this webhook checks the **resolved provider** before changing JSON.
It is attached with the mandatory scanner guardrail. The default GPT-4.1 target
uses native `drop_params: ["reasoning"]` inside the gateway. Client default
requests still omit `model`; explicit client requests still use `@provider/model`.

The hook requires its own `x-airs-compatibility-key`, supplied server-side by AIRS.
It accepts uncompressed JSON, limits request bodies to 16 MiB, concurrent bodies
to two, connections to 64, and request time to ten seconds. It makes no external
requests and logs no prompt, credential, or response bodies. The hook result
records only the rule version and removed field names. Invalid credentials,
malformed contexts and overload fail closed through the gateway hook policy.
AIRS imposes a three-second hook timeout. Requests exceeding these limits are
rejected; the terminal context budget is not a guarantee of every model's limit.

Run `node --test gateway-compatibility/server.test.mjs` from the repository root.
The service has no npm dependencies. `Dockerfile` pins the distroless base. This
host assembled the equivalent OCI file layer with `crane mutate`; image/source
hashes and the Harbor scan are in `validation/2026-09-07/compatibility/`.

Infrastructure is in `cdot65/talos-cluster/airs-terminal-compatibility`:
two digest-pinned non-root replicas, read-only filesystem, no service-account
token, resource bounds, TLS ingress, and a NetworkPolicy denying all egress.
The existing scanner deployment is independent. See
[request compatibility administration](../administration/request-compatibility.md)
for bindings and rollback.
