---
id: troubleshooting
title: "Troubleshooting by trust boundary"
sidebar_label: "Troubleshooting by trust boundary"
---

## Locate the failing step

Something failed. Before changing anything, decide which boundary it failed at. Follow the request from the harness to the intended gateway listener, then to the model or mcp server 1, and ask at each hop whether the problem is authentication (who are you), authorization (what may you do), argument validation (is the input acceptable) or tool execution (did the work succeed). Those are four different questions, and the fixes for them do not overlap: a new login does not repair a bad argument, and a corrected argument does not add a missing grant.

```mermaid
flowchart LR
    accTitle: Diagnose inference and local MCP utility failures
    accDescr: Identify the failing listener, check network reachability and credentials, inspect gateway and server grants, and then validate utility arguments and local execution. No downstream management API appears in the utility path.
    failure["Request failed"] --> destination{"Which gateway listener?"}
    destination --> inference["Inference: credential, route and content policy"]
    destination --> mcp["MCP: gateway login and integration access"]
    mcp --> upstream["Gateway upstream OAuth grant"]
    upstream --> grant["mcp server 1 token, roles and subject binding"]
    grant --> input["Tool name and argument schema"]
    input --> execution["Local utility execution and result"]
```

## Diagnose inside the selected environment

In stable 0.1.1 and the newer test releases, enter `/doctor` for the connection-health dashboard. **Refresh diagnostics** refreshes local diagnostics without making an inference request. A native-store availability check does not prove that every saved credential is readable. **Verify gateway access** asks for confirmation; only **Send connectivity check** sends the small inference probe. It may consume quota and appear in gateway logs, but sends no local files, conversation or tools. The equivalent shell commands are `airs --environment work doctor` and `airs --environment work doctor --verify-access`.

**Restore company sign-in** renews inference SSO for the same verified identity. Workspace-key replacement remains a shell action: leave the session, run `airs --environment work login --with-api-key`, and reopen the same environment. Use the name displayed by the session; local names do not need to match gateway workspace names.

**Optional test release `0.1.2-alpha.1.mcp.1`:** when startup or `resume` encounters a rejected refresh grant or an uncertain refresh that may already have been consumed, AIRS exits before opening the session and recommends `airs --environment NAME login --restore-session`. Use the environment named by the error, sign in as the same person, then retry the original command. Existing conversations are preserved; no operation is replayed automatically. `/signin` remains the recovery action in an already-open session. The published package passed installed recovery fixtures; an attended restoration test remains separate.

For MCP, select the affected connection through `/doctor` or `/mcp`. Choose **Sign in** for expired or absent credentials, or **Reconnect and verify** for fresh initialization and tool discovery. After connection changes choose **Start new conversation**; history and the unsent draft remain saved and nothing is replayed. A tool inventory is still weaker evidence than a completed authorized tool call.

A workspace key being saved is not evidence of an allowed model route. Check the key's workspace, inference permission and default saved config with your administrator. Recreating a local environment does not fix a gateway route or policy denial. An unavailable native credential service does not necessarily mean a locked store: check its availability and any OS authorization prompt in the same user session. If cleanup is pending, restore service access and retry the displayed environment-specific login or logout. A report can identify the failed check; exact local error categories may need separate review with your administrator.

## Symptoms and useful checks

Most of the checks below test a boundary rather than a component, so the first question is usually whether the request reached the place you think it reached.

| Symptom | First useful check |
| --- | --- |
| Inference fails after about 30 minutes idle | Whether the inference grant is still renewable; use the company sign-in prompt |
| Alpha.15 shows the generic bound-credential fatal error after idle | Enter `/signin`; alpha.16 includes the provider-routing correction for automatic guidance |
| Login completes but credential save fails | Native Keychain or Secret Service availability and the reported storage category |
| Browser cannot reach localhost | Use the current MCP flow's hidden callback input over SSH, or verify callback routing for browser-only inference flows |
| Gateway denies workspace access after CAS login | CIE group-to-workspace mapping and the workspace Members view |
| Gateway MCP login succeeds but upstream needs consent | The separate gateway-held upstream OAuth grant |
| mcp server 1 rejects the token | Issuer, signature, resource audience, allowed client and time limits |
| mcp server 1 rejects authorization | `invoke`, `utilities.use` scope and role, and the subject's policy binding |
| Resources list is empty | Inspect `/mcp` for tools; resources and tools are separate capabilities |
| The model never selects a visible tool | Actual inference tool declarations and model-returned function calls |
| `calculate` fails for division by zero | Correct the input; a new login does not repair arithmetic |
| `decode_base64` rejects text | Canonical padding and valid decoded UTF-8 |
| `current_time` differs from the workstation | The tool reports the MCP server's clock |
| `airs` reports that its native package is unavailable | Reinstall the same exact version with `--include=optional`, preserving the original registry and global/local installation scope; follow the repair example in the login lesson |
| An upgrade still shows an older version | Resolved executable, npm prefix and `PATH` ordering |

To return from the optional test release to stable, reinstall exact `airs-harness@0.1.1` using the same registry and installation scope, then restart AIRS. Keep existing environments and credentials. The login lesson provides both exact installation commands and the tested rollback scope.

A missing native dependency and an unsupported platform are different failures. Do not switch releases or change npm configuration merely to repair an omitted package. Normal installs include optional dependencies; the explicit flag is a repair for a missing dependency. See [the login lesson](./login.md) for the exact 0.1.1 repair example.

Tool execution has no downstream management API to diagnose, which removes a whole class of failure from the utility path. The server's authentication layer can still depend on trusted signing-key retrieval, so DNS or issuer-key failures are real, but they belong to the identity boundary rather than to the tool.

## The alpha.15 idle-return incident

The maintainer returned after about 30 minutes idle and received a generic fatal credential-helper error. Manual `/signin` completed, preserved the same verified identity and conversation, and allowed the next inference reply. The credential path was fine; what failed was the guidance. The normal AIRS provider dispatch had bypassed the typed recovery handler, so the terminal never received the classification it uses to show sign-in guidance. Alpha.16 corrects that route so an expired or otherwise non-renewable sign-in reaches the terminal's guidance. A real helper integration test failed before the change and passes afterward; all 78 provider tests pass. Be precise about what that evidence covers: it establishes the source correction. It does not prove every production token-renewal case.

## Ubuntu SSH credential readiness

An SSH connection and a usable credential store are separate prerequisites. Public-key SSH authentication does not automatically unlock an encrypted login keyring. Check readiness in the same OS user session that will run AIRS:

1. Confirm a supported Node version and npm. The harness requires Node 22.13.0 or newer in the 22.x line, or 23.5.0 or newer; installing npm does not upgrade Node 18.
2. Confirm Bubblewrap is installed and the host permits the harness's sandbox. Have the administrator diagnose the applicable namespace or AppArmor policy if the sandbox cannot start; do not disable those protections globally.
3. Confirm a per-user D-Bus session and a Secret Service provider are available. A running provider and an unlocked encrypted collection are separate conditions.
4. Unlock the existing collection through the OS credential service in that user session, and handle any OS authorization prompt. A missing service, a locked collection and a denied authorization need different repairs.

```sh
node --version
npm --version
command -v bwrap
command airs --version
airs --environment work doctor
```

The version and prerequisite checks need no production credential. Run `doctor` after selecting an existing environment; it reports local readiness without an inference request. `doctor --verify-access` is a separate authenticated probe that can consume gateway quota. A readiness report does not prove every saved credential is readable or that a model route is authorized.

New environments created by mcp.4 and later require native MCP storage and fail sign-in without a plaintext fallback when the service is unavailable. Older environments keep their existing mode; use the login lesson's cleanup-and-sign-in procedure if migrating them. Preserve the existing keyring and credentials while repairing access: do not reset the store, copy token files or choose plaintext storage as a workaround. Enter an unlock password only through a trusted hidden prompt or the credential service's protected input, never in command arguments, logs or chat.

The Ubuntu helper bundled in mcp.4 predates a keyring-daemon correction. Use the verified mcp.5 helper or a corrected administrator-provided copy rather than extracting that older helper for SSH unlock. The corrected helper checks the actual collection state, preserves encrypted data after a wrong password, and reuses an unlocked service. The bundled mcp.5 helper passed its isolated encrypted-keyring regression with the default mcp.5 version. That package proof is separate from the user's own successful unlock; the implementation-status lesson records both boundaries.

The isolated fresh-Ubuntu checks establish fixture readiness. A subsequent read-only observation found an unlocked user collection and passing readiness reports; that observation does not imply the checks unlocked it or establish company SSO.

## Browser callbacks on a remote machine

A callback to `127.0.0.1` reaches the browser's computer, which may differ from the harness host. For MCP, the in-session sign-in dialog supports opening the authorization URL on another computer and pasting the complete callback URL into its hidden callback input. The shell fallback is `airs --environment work mcp login service-now --no-browser`. Keep that attempt open; never paste a callback into the agent conversation or a support report. A correctly routed HTTP callback can also finish the same attempt.

For inference, use **Use device authorization** or `airs --environment NAME login --device-auth` over SSH when your issuer enables the grant for the harness client. Open the verification link and enter the user code on your laptop or phone; the SSH host polls over HTTPS and needs no inbound callback or tunnel. `login --no-browser` selects the browser callback flow, not device authorization. See [the login lesson’s browserless walkthrough](./login.md) for a separate SSO profile that preserves an existing workspace-key profile. If using the browser callback flow instead, the callback must reach the harness host, which can require a correctly bound tunnel. These are different flows; MCP manual callback support does not imply inference device authorization support at every issuer.

The callback window is five minutes in the tested MCP flow. An expired tab cannot complete a later attempt because its state and verifier belong to the earlier one. The PKCE verifier and native credential store remain on the harness host. Verify native completion and an actual request after browser consent; the success page alone is insufficient.

## Capture a useful report

**Stable 0.1.1:** record the package version, platform, approximate idle time and failed step. Describe whether a tool completed, failed or has an uncertain outcome. Review any additional identifiers before sharing them through your organization's support channel. Environment names, tool names and request IDs may be private; do not add them to a public report automatically. Never share credentials, device codes, authorization/callback URLs or private tool input.

**Published optional test version `0.1.2-alpha.1.mcp.1`:** the new `/doctor` → **Diagnostic report** action offers **Preview report**, **Copy report** and **Save local report**. These controls are not in stable 0.1.1. Preview shows the exact text that will be copied or saved; **Esc** returns to the actions. Saving creates a new owner-only `diagnostic-report-*.txt` file in the current environment's state directory and displays its location. If your terminal declines clipboard access, use preview or the saved file. Nothing is uploaded automatically.

The report allowlists version/platform, authentication method, known check outcomes and recovery steps. It excludes local environment/connection names, addresses, paths, credentials, raw errors, conversation content and tool inputs/results. It does not include individual MCP status or Node/npm versions. Report actions reuse the last completed snapshot and make no additional health, credential-store, inference or MCP request. Running `/doctor` itself still performs its existing diagnostics; **Verify gateway access** remains a separate, explicit inference action.

A passed native-service check does not prove access to the saved credential. A failed service check does not establish that the store is locked. Gateway health does not verify inference, and MCP discovery does not verify a completed tool call. An absent inference check remains **Not verified**. Inspect the report before sharing it; raw `airs doctor --json` contains more detailed local information and is not the shareable report.

Continue with [Labs and answer keys](./labs.md).
