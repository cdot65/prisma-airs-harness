# AIRS CAS and helper policy — implementation in progress

Private CAS retains its existing issuer-origin binding, HTTPS, redirect refusal, PKCE S256, JWT signature/audience/subject checks, bounded JSON, and OS-store durability. The new policy read side admits discovery, JWKS, token exchange, device authorization/polling and revocation. Permits cover response parsing and cancel denied in-flight operations.

Refresh has an explicit admission boundary. `prepare_refresh` checks identity configuration and destination before the caller writes `RefreshPending`. The one-shot admitted exchange can execute only after that durable write. Pre-admission failure leaves the original record intact. After the write, cancellation, revocation and ambiguous response errors retain the pending marker; they never restore or replay a rotating predecessor. Only definitive `invalid_grant` becomes sign-in-required.

Short-lived AIRS CLI helpers load local managed application requirements for their invocation. These processes do not share the parent controller and do not claim live cross-process revocation. The parent still governs managed inference/MCP request transports. Gateway and TypeSafe diagnostic probes also require policy admission, with no redirects. User-directed browsers and approved CLI/Jev SDK children retain their existing distinct execution/approval boundaries.

Validation to date: local identity suite initially passed 31 cases; the added device-poll cancellation case also passes in the corrected combined run. The local combined run passed 440 cases and generated one intentional new snapshot; the reviewed snapshot passes separately. The corresponding native run passes all 441 cases. Scoped lint passes after automatic style corrections; final lint is clean and formatting passes. The full five-package native suite stopped on ENOSPC (not a passing test run). Removed only this task's incremental compilation data while retaining compiled artifacts, recovered 23 GiB, and restarted the unchanged suite.

Adversarial findings corrected:

- Standalone MCP add/login/list constructed clients outside embedded startup. They now bind the effective requirements from their loaded config while preserving any existing live controller. A transport test verifies no MCP request reaches a denied endpoint.
- Generic credential-helper failure could suggest an unavailable sign-in service. The bounded helper marker now carries a distinct policy-denial classification; model-provider dispatch treats it as fatal, not unauthorized. CLI startup preserves that guidance. The snapshot demonstrates the message, unchanged draft, and absence of sign-in recovery.
- Post-admission policy revocation remains an unknown rotating-token outcome, not a safe-to-retry denial. Store tests preserve the tokenless pending record, while pre-admission denial preserves the original credential bytes.

MCP's existing HTTP capability adapter still surfaces transport-layer failures through its established error interface; this slice does not add an exec-server wire error schema. Enforcement is tested before sending bytes. External browser and approved child-process policy boundaries remain explicit above. No final feature score or artifact acceptance is claimed yet.
