# Policy loading and framed-response prerequisites

Adapted the HTTP/config subset of upstream `22a3f6d5d8` (PR 47407). Capture and validate local application requirements before cloud requests; preserve normal cloud fragment precedence without interpreting unrelated requirements during bootstrap. Refresh macOS managed preferences before using cached requirements. Add endpoint narrowing, explicit TLS and framed response support for later account ownership and gRPC conversion.

Review findings and corrections:

- Kept the existing raw reqwest factory method public until the code-mode caller is migrated. Removing it in this prerequisite alone would break the workspace. This is not a claim that the legacy caller is policy protected.
- Adapted cloud layer extraction to the existing loader rather than importing an absent managed-requirements refactor.
- Endpoint narrowing intersects existing restrictions and cannot grant an otherwise denied host.
- Framed response wrapping preserves gRPC trailers and terminates queued output on revocation. A follow-up test explicitly queues bytes before revocation and confirms they are not returned.
- Captured local policy remains stable if its backing file changes; malformed local policy is rejected before any cloud composition. Malformed cloud application policy also fails rather than being ignored.

Linux affected suite: 416 passed, zero skipped. Final strengthened frame test: one passed, 415 excluded by focused filter. Native Mac affected suite: 419 passed, zero skipped; final frame test one passed, 418 excluded. Native scoped lint passed without warnings. Local scoped lint and formatting pass; 18 incidental formatter paths restored. Native Bazel regeneration and strict check pass, lockfile bytes unchanged.

No feature score: application/account ownership, private CAS integration and release gates remain pending. This commit provides prerequisites, not a completed security feature.
