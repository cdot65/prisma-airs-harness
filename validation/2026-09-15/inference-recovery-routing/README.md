# Inference sign-in recovery routing

After roughly 30 minutes idle, the owner’s published alpha.15 session displayed
the generic bound-credential fatal error instead of opening sign-in guidance.
Manual `/signin` restored the same verified identity, preserved the conversation,
and allowed the next inference reply. This verifies manual idle recovery only;
it does not establish active renewal or a subsequent MCP tool call.

`ModelProvider::api_auth_for_scope` sent custom providers directly to `api_auth`.
AIRS uses command-backed credentials, so its normal inference path skipped the
recovery classification in `resolve_provider_auth_for_scope`. Existing tests
invoked that classifier directly and missed this routing decision.

AIRS gateway command credentials now take the recovery-aware path. Other custom
providers retain their existing dispatch. Token renewal, credential persistence,
the idle policy and conversation binding behavior are unchanged.

The new integration test constructs the real configured provider and executes a
local helper subprocess through the inference dispatch entry point. It failed
before the fix and passes afterward for all four recovery classifications.
Another case verifies successful credentials still produce the expected bearer
header. The full provider suite passes 78 tests. No new release binary is claimed
by this source-level evidence.
