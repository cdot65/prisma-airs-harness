# Connected application network policy review

Source: `63920760fc7a5e6434e2dc8741194530b6a6f926`. This is a self-review against counterexamples and retained test evidence, not an independent certification. The bounded transport feature now passes its review gate; subsequent features and signed delivery remain incomplete.

| Invariant | Counterexample checked | Evidence |
| --- | --- | --- |
| Admission precedes network access | Denied DNS/connect, HTTP draft, standalone MCP endpoint, CAS discovery/refresh and gateway diagnostic | Policy primitive/request tests; stage-10 identity/CLI tests |
| Permits cover response lifetime | Revocation while body/frame/SSE/WebSocket/gRPC data is pending | Stages 5–7 transport and caller tests |
| Redirects cannot broaden authority | Destination changes, URL credentials, issuer endpoint mismatch | HTTP redirect tests; CAS redirect refusal; exact bootstrap endpoint tests |
| Retained clients observe revocation | Account switch, local load failure, overlapping stale publication, config reload | Policy/account ownership tests, stages 7–9 |
| Embedded startup shares policy | Pre-start clients and later config/session/permission rebuilds | Stage-9 activation and rebuild tests |
| CAS rotating-token state remains safe | Pre-admission denial versus cancellation after a request could start | Stage-10 byte-preservation, no-connect and pending-state tests |
| Denial does not cause auth fallback | Local denial marker, 401 recovery dispatch and user draft | Core credential-preservation integration; model-provider fatal classification; AIRS recovery snapshot |
| AIRS boundaries stay intact | Gateway route, issuer/resource checks, redirects, Jev child execution | Existing AIRS package tests plus scoped source audit |

The CAS transaction is intentionally stricter than ordinary retryable network operations. Admission failure preserves the original credential. Once the durable pending record is written, an unknown outcome never restores a rotating predecessor. This remains true when policy changes while the request is in flight.

Short-lived CLI helpers load a local requirements snapshot for each invocation. They do not share the parent's live controller. The managed parent transport still enforces its own effective policy. Approved Jev/CLI child processes and user-directed browsers retain their existing separate execution boundaries; this feature is not a claim of a process-wide network firewall. The existing MCP capability error interface remains generic, with no-send enforcement tested underneath.

The final native affected-package run passed 5,800 tests with six skips. One twelve-session recovery test exceeded its 60-second timeout before passing the configured retry in 29.503 seconds. Earlier isolated rechecks passed. Keep this known timing flake visible; do not call the run flake-free. An earlier ENOSPC-aborted run remains retained and is not counted as passing.

Scoped local lint and native lint pass. Native lint made no source changes, and all 98 changed source files across the final ownership/embedded/CAS stages match the final local source. Tests preceding style-only formatting/lint edits are identified as such; no signed artifact or installed-package equivalence is inferred.

The stage-5 GNU run had three failures whose complete assertions match the retained stable baseline. The current source requires its own comparison: run 3850 must finish and any new failure must be investigated before feature scores are assigned. Full integration, later feature gates, and signed Mac delivery remain separate requirements even after this bounded feature passes.


## Final bounded feature gate

GNU run 3850 completed against the exact source/tooling commit: 18,535 passed, three failed, 34 skipped, one flaky. Every assertion from both attempts of each persistent failure matches the retained stable baseline after normalizing only timestamps, ANSI/indentation, process IDs and temporary fixture paths. The comparison also checks that the failure count equals the complete identified failure set. These are existing remote exec-server fixture failures; they are retained rather than removed or called passing. AIRS does not enable the remote exec-server product path.

The one GNU subagent deadline flake passed its configured retry and ten consecutive focused repetitions with retries disabled, in the same container and compiled source. This does not prove the flake is fixed. Together with the native results and source/lint checks above, no new persistent regression was found in the required scope.

Scores: implementation **9**, code quality **9**, design **9**, bounded feature completeness **9**. Implementation covers admission and in-flight revocation across the connected consumers, with credential-state regressions. Quality has scoped source changes, separate tests, matching native source and clean scoped lint. Design preserves upstream transport ownership while making the AIRS CAS rotating-token and child-process boundaries explicit. Completeness covers embedded startup/rebuilds and standalone AIRS helpers, with the existing generic MCP error interface documented. The known baseline failures, timing flakes, per-process snapshot boundary and pending signed-artifact acceptance prevent a 10/10 claim. These scores do not certify the unfinished session, terminal, fullscreen or release phases.
