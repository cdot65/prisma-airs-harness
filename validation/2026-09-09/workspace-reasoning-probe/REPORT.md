# Workspace reasoning-model connectivity probe

The owner reported alpha10 workspace-key login saving credentials successfully,
followed by repeated doctor gateway_access failures. Their Chat Completions
request succeeded. No credential value is retained here.

## Reproduction

The operator workspace credential from its existing protected file passed the
exact 16-token Responses probe, returning a GPT-4.1 message. This ruled out a
universal Responses endpoint outage, not a problem in another workspace.

The independently retrieved service credential named Prisma AIRS Harness in
workspace harness (ws-coding-8f2e8a) reproduced the reported failure shape:

| Request | HTTP | Responses status | Output item types |
| --- | --- | --- | --- |
| Default route, 16 output tokens | 200 | completed | reasoning |
| Same key/route, 128 output tokens | 200 | completed | reasoning,message |

The 16-token request correlation is c2fd8720-34f7-4bfa-a768-c41c236ee2ba;
the 128-token control is f95f2e62-de9e-465d-b47a-09023ee95feb.
The configured provider is @vllm2 and override model is
unsloth/Qwen3.8-27B-GGUF:UD-Q4_K_XL, matching the model in the owner's response.
Config pc-prisma-56f8b0 was inspected but not changed. The owner's exact pasted
key was not copied into commands or logs. The operator test used the protected
management API retrieval; key material stayed in subprocess memory/environment.

## Cause and change

The connectivity probe required a message output item even when the gateway
reported successful inference with reasoning output. 16 tokens can be consumed
before the model produces final text. This was a false access failure, not
invalid key material. The patch accepts message or reasoning items in completed
or incomplete Responses results. Failed status, non-null API errors, missing
output, invalid shape, and authentication denial still fail. The fixed request
retains its disclosed 16-token limit, does not add a model for default routing,
and does not expose reasoning content or credentials in diagnostics.

## Existing released application

The exact alpha10 npm-installed Linux executable authenticated using the named
workspace key in isolated temporary state and completed a normal agent turn:
Workspace inference works.

A read-only execution correctly denied a file write. Repeating with explicitly
selected workspace-write permissions completed two local shell tool calls and
created/read proof.txt containing exactly workspace-tool-ok followed by newline.
No user files were touched. This validates the existing agent inference/tool
path separately from the defective doctor probe. It does not claim owner Mac
execution or newly tested MCP/Keychain behavior.

## Delivery boundary

The patch is source-only until a new signed release is built and published.
Existing installed alpha10 binaries still contain the defective probe. Users
with successfully stored credentials can launch their environment normally;
this investigation does not require changing gateway routes or increasing the
probe budget. The exposed owner key should be rotated through the normal
credential-management process; no shared key was revoked during diagnosis.


## Regression validation

`CARGO_TARGET_DIR=/var/tmp/airs-auth-target CARGO_BUILD_JOBS=4 just test -p codex-cli airs_access`
passed all20 selected tests (10 probe tests in each CLI binary target).
The new test covers completed and incomplete reasoning-only responses, verifies
one request and the unchanged16-token budget. Negative cases still reject empty
output, failed reasoning responses, non-null API errors, auth denial, redirects,
unsafe destinations, oversize responses, and credential-helper failures.
Repository formatting and diff checks passed. The remaining849 unrelated tests
were not selected. No optimized or signed release was produced.

The patched local Linux binary also passed a live doctor verification on the
Qwen workspace route. PATCHED-LIVE-DOCTOR.json retains the safe receipt and
VALIDATION.json binds it to the source commit and local executable hash. This
local debug build retains the alpha10 version string; it is not the published
alpha10 artifact and has not been shipped to macOS.
