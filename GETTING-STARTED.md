# Getting started with Prisma AIRS Harness

## SSO to ServiceNow: a complete first session

The outcome is concrete: you sign into the harness as yourself, connect the ServiceNow MCP integration in the same environment, and ask the agent to read an incident. Your company SSO identity is used throughout the human login steps. Inference and MCP still receive separate credentials, and the ServiceNow backend uses a server-side integration account.

**Release: 0.1.1.** The stable release includes guided environment setup, company SSO or workspace API-key inference, the in-session `/mcp` connection manager, and `/doctor`. Desktop MCP sign-in opens the browser and reports credential storage and tool discovery. SSH users can paste the full callback into the hidden sign-in field. Existing environments retain their storage mode when upgraded. Native MCP device authorization remains dependent on gateway support.

The owner confirmed the complete inference → ServiceNow sign-in → read-only query → restart/reuse workflow on Apple Silicon with the preceding mcp.6 release. Automated package checks and real-account acceptance are recorded separately.

The npm package remains `airs-harness`; invoke it as `airs`. **Prisma AIRS CLI 7.1.5** and nine product skills are bundled as `airs cli`, so no separate product CLI installation is required. Supported native packages are Linux x64, Linux ARM64 and Apple Silicon; Windows and Intel Mac packages are outside this release.

Check Node.js and npm in the terminal you will use. The harness requires **22.13.0 or newer in the 22.x line, or 23.5.0 or newer** (`^22.13.0 || >=23.5.0`). Installing npm on Ubuntu does not upgrade a distro-provided Node 18. Install a supported Node version using your organization's method, reopen the terminal, and check again.

```sh
node --version
npm --version
npm install -g airs-harness@0.1.1 --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Replace the example registry with your administrator's registry. Ordinary installs include the matching native package; `--include=optional` is unnecessary unless npm configuration explicitly omits optional dependencies. Upgrades preserve existing environments, credentials and histories. Restart an already running AIRS process after upgrading.

If `airs` reports that its native package is unavailable, reinstall the exact package version shown in the error with `--include=optional`. Use the same registry and installation scope as before: retain `-g` for a global installation, or run the install in the same project for a local installation. This repairs missing optional dependencies without changing releases or your npm configuration.

If an old standalone CLI owns `airs`, upgrade it to `@cdot65/prisma-airs-cli@7.0.1` first; it uses `airs-cli`. Do not force npm to overwrite another package's command. Stable harness 0.1.1 bundles CLI 7.0.0. For `redteam judge`, install [harness preview 0.1.2-alpha.5.mcp.1](RELEASE-TYPESAFE-JUDGE.md), which bundles CLI 7.1.5. Standalone CLI 7.1.5 is also published on public npm under `next`.

Use `type -a airs airs-cli airs-harness` and `airs --migration-check` to investigate a shadowed executable. A temporary `airs-harness` compatibility alias remains in 0.1.1; new commands use `airs`. If an earlier review installation exported `PATH` or `AIRS_HARNESS_HOME`, use a fresh terminal so those exports do not select its isolated state.

### 1. Get the connection details and access

Ask your administrator for these public connection settings. The values below are examples, not a live tenant configuration.

| Setting | Example | Used for |
| --- | --- | --- |
| Inference API URL | `https://gateway.example.com/v1` | Model requests through AI Gateway |
| Company OIDC issuer | `https://sso.example.com/realms/company` | Your inference browser sign-in |
| Public native client ID | `harness-native` | The installed harness; no client secret |
| Inference audience | `airs-inference` | The resource expected in the inference token |
| ServiceNow gateway MCP URL | `https://gateway-mcp.example.com/mcp-service-now-dev/mcp` | Your gateway-mediated ServiceNow connection |

Your account needs inference access, membership in the gateway workspace that exposes ServiceNow, and a ServiceNow MCP subject binding with the appropriate incident permissions. Being able to sign into SSO does not grant those permissions automatically. The administrator provisions the gateway integration and its upstream OAuth client before you add it locally. The example integration targets a ServiceNow development instance.

Have Git, ripgrep and your project's own build tools available. Linux also requires a usable Bubblewrap sandbox. Use a desktop browser and an available OS credential store. On macOS, sign in from the desktop session and allow Keychain access. On Linux, make sure the Secret Service/keyring session is available and unlocked. Passwords belong in the company browser page, never in a command or configuration file.

For an Ubuntu SSH test host, the optional [Ubuntu preparation guide](UBUNTU-TEST-HOST.md) installs prerequisites and checks the sandbox and credential service before attended sign-in.

### 2. Create or select a local environment

For a new profile, start guided creation:

```sh
airs env create work
```

Enter `https://gateway.example.com/v1`, choose **Create environment and sign in**, and follow one of the authentication paths below. This shell command finishes at the shell after sign-in. New environments created by 0.1.1 need no manual MCP storage setting. Cancelling before creation saves nothing; cancelling after creation preserves the environment so you can resume login.

If `work` already exists, reuse it:

```sh
airs env use work
airs --environment work login
```

Alternatively, supply the gateway URL explicitly, then sign in separately:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
```

Creation with `--gateway-url` saves and selects the environment without opening sign-in. Bare `airs` also offers **Connect an environment** on a fresh installation. In the welcome screen, **Choose another environment** selects a destination for that session; `airs env use NAME` changes the saved default. Do not recreate an existing profile to repair a cancelled or denied login.

### Environments and gateway workspaces are independent

An **environment** is a local profile containing a gateway URL, credential binding, model settings, MCP connections and conversation history. Create one when you need separate credentials, destinations or histories—for example, `work-sso` and `workspace-api`.

Native MCP keyring records with the same connection name and URL can be shared by the same OS user across environments. Use distinct MCP connection names when you need separate local MCP credentials; a different environment name alone does not isolate that record.

A **gateway workspace** is the server-side boundary that owns provider access, saved model configs, API keys, budgets and guardrails. `airs env create` only creates the local profile. It does not create a gateway workspace or require matching names.

For example, local environments `work-sso` and `workspace-api` can both use the gateway workspace `agent-team`. An API key is bound to the workspace where it was created; SSO uses the authorized workspace mapping in its token. Renaming an environment does not change either binding.

```sh
airs env list
airs env use workspace-api
airs --environment work-sso doctor --verify-access
airs env remove workspace-api
```

`env use` changes the default for new commands; `--environment` selects one command's environment. `env remove` unregisters the local environment and preserves its history on disk. It does not delete a gateway workspace or revoke a key. Revoke keys in AI Gateway when their access should end.

### 3. Sign into inference with company SSO

Choose **Sign in with company SSO**. Enter the company issuer, public client ID and gateway audience from the table. These are public connection settings, not a client secret. Later attempts offer **Continue with saved settings**.

Choose **Open browser on this machine** on your desktop. Over SSH, choose **Use device authorization** when your company issuer supports it, and follow the displayed verification link and code on a device with a browser. **Show the full browser URL** retains the manual browser flow; its callback must reach the machine running AIRS, so device authorization is usually easier over SSH.

Sign in as the intended company user in the browser, then return to AIRS. The screen shows progress through authorization, native credential storage and a minimal inference access check. The browser success page alone does not prove credential persistence. **You're ready to use AIRS** means the credential was saved and that inference check passed. Continue with step 4; only existing environments or mcp.3 installations may need a manual MCP storage change.

The access check sends one small inference request and can consume gateway quota; it sends no local files or tools. A denied or unavailable gateway produces **Credential saved · gateway access needs attention**, with separate options to retry the check, continue or exit. Fix access before proceeding with this walkthrough. Storage failures remain sign-in failures and offer recovery guidance; inference credentials have no plaintext fallback. Escape cancels an unfinished sign-in and preserves the environment.

To supply public settings explicitly, use the existing command form:

```sh
airs --environment work login \
  --issuer-url https://sso.example.com/realms/company \
  --oidc-client-id harness-native \
  --audience airs-inference
```

If login was cancelled, run `airs --environment work login` and choose company SSO again. To inspect the saved environment or retry an access check without another browser login:

```sh
airs env status work
airs --environment work doctor --verify-access
```

`env status` inspects local configuration; it does not prove fresh authentication or remote access. `doctor --verify-access` sends another inference probe. Neither result tests ServiceNow tools. Adding MCP will not repair an incorrect inference URL or missing inference entitlement.

### Optional: TypeSafe judge key for red-team scoring

Release **0.1.2** makes live skill runs request approval for the
judge command to access your native
credential store and send scan records to TypeSafe. Approve that specific command
inside AIRS; dry runs and explicit replay keep normal sandbox permissions. A saved
key can be readable by `/typesafe` while inaccessible to a sandboxed shell. If a
shell attempt reports an unreadable key, retry through the command approval path
before replacing it. The workspace sandbox remains enabled for other commands.

The JavaScript entrypoint requires release **0.1.2** with CLI
**7.1.5**. See the [release record](RELEASE-TYPESAFE-JUDGE.md) for publication
status. Stable 0.1.1 does not include the judge.

Inside `airs`, enter `/typesafe` and choose **Save or replace API key**. Paste the
key into the hidden field and press Enter. The dialog also shows configuration
status and lets you remove the saved key. Escape cancels without changing an
existing key. Your conversation and unsent draft are preserved.

No external command is required. For terminal automation, the existing
`airs --environment work env typesafe set` remains available. Use `/doctor` and
its access verification action to check the key against the TypeSafe models
endpoint; saving a key alone does not verify remote acceptance.

Inside `airs`, invoke `$prisma-airs-asr-judge attacks.json`. Its JavaScript
entrypoint automatically retrieves the owning environment's saved TypeSafe key
and invokes the bundled TypeScript CLI. No Python or separate SDK installation
is needed. An inherited `TYPESAFE_API_KEY` takes precedence. The key stays scoped
to the judge child and is not placed in arguments or saved to a tenant file.

Doctor checks the TypeSafe models endpoint without sending judgment questions.
The skill starts with a dry-run and a small recorded probe. An absent shell
variable does not establish that the environment's saved key is missing.
Credential failures report the actual error; replay is used only when requested
and is clearly labeled as reused answers. Standalone `airs-cli redteam judge`
continues to read its tenant's `typesafeApiKey`; `--job` additionally requires
AIRS management credentials in the CLI tenant.

### Browserless sign-in over SSH

AIRS already supports device authorization for inference. On a host without a browser, select **Sign in with company SSO → Use device authorization**, or use `login --device-auth`. Your administrator must enable device authorization for the public harness client; an issuer advertising the endpoint alone does not prove that client is permitted.

To try SSO while keeping a working workspace-key environment, use a separate local profile. In this example, `work` is your existing default; substitute its actual name and your administrator's connection settings. Skip creation if `work-sso` already exists:

```sh
airs env create work-sso --gateway-url https://gateway.example.com/v1
airs env use work
airs --environment work-sso login --device-auth \
  --issuer-url https://sso.example.com/realms/company \
  --oidc-client-id harness-native \
  --audience airs-inference
```

Keep the SSH terminal open. Open your laptop or phone browser yourself; the SSH process cannot open that browser automatically. Open the displayed verification link there, enter the displayed code, and complete company sign-in there. AIRS polls the issuer from the SSH host; this flow needs no inbound callback or SSH port forward. The code is short-lived: if it expires or you cancel with Ctrl+C, rerun the login command to start a new attempt. After the command reports that credentials were stored, run `airs --environment work-sso doctor --verify-access`. This separate check sends one small inference request and can consume gateway quota. Once access is verified, open `airs --environment work-sso`. If credentials were saved but the gateway denies access, fix the reported route, entitlement or policy issue; successful browser approval does not grant a missing gateway entitlement.

`login --no-browser` selects the authorization-code flow and prints its browser URL. It is not device authorization; that inference flow still needs its callback to reach the SSH host.

| Sign-in | Browser on another device | Return to the SSH terminal |
| --- | --- | --- |
| Inference with `login --device-auth` | Open the verification link and enter the user code | Wait for polling, credential storage and the inference result; no callback to paste |
| Gateway MCP through `/mcp` | Open the connection's authorization URL and complete consent | Paste the full callback URL into the manager's hidden input, even if the browser cannot load its localhost page |

Inference and MCP use separate credentials and permissions. A workspace API key or successful device login does not sign in an MCP connection. Paste an MCP callback only into its hidden authorization field, never into the agent conversation. The gateway continues to own upstream ServiceNow authorization.

### Alternative: use a workspace API key for inference

Use this path when your administrator permits workspace-key authentication. You can use the same gateway workspace as SSO; the workspace policy must explicitly support both methods.

1. Open Prisma AIRS AI Gateway in Strata Cloud Manager and select the intended tenant and **gateway workspace**. The gateway is available under **AI Security → AI Gateway**; see the [vendor configuration guide](https://docs.paloaltonetworks.com/ai-runtime-security/administration/configure-ai-gateway).
2. In that workspace's API-key management, create a **user workspace API key** for your user. Give it inference permission (`completions.write`) and the expiration and limits required by your administrator. A provider integration key or AIRS scanner key is a different credential.
3. Attach the administrator-approved **default saved config** to the key. That config selects an authorized provider/model when the harness uses the gateway default. Confirm its provider is provisioned into this workspace. A key grants access; it does not create a model route.
4. Copy the newly issued key into the harness's hidden prompt. Do not put it in a command argument, shell history or chat transcript.

```sh
airs env create workspace-api --gateway-url https://gateway.example.com/v1
airs --environment workspace-api login --with-api-key
airs --environment workspace-api doctor --verify-access
```

Skip `env create` if the environment already exists. Interactive `airs login` also offers the workspace-key choice. The hidden prompt saves the key in the OS credential store. **Credential saved** confirms local storage; **Gateway access verified** confirms one successful inference request. A user key can retain gateway-side user attribution, but it does not create an OIDC sign-in session in the harness.

For recovery, HTTP 401 indicates rejected authentication; HTTP 403 indicates insufficient permission; HTTP 446 indicates a blocking guardrail. Some gateways return a completed response with HTTP 200 for a blocked request. AIRS checks the blocking hook results and reports that as a policy denial too. Give your administrator the trace ID, not the credential. Recreating a local environment will not fix a workspace policy or missing route.

**MCP still needs its own login.** Continue with the in-session ServiceNow steps below, using `workspace-api` wherever the example selects `work`. The gateway-facing MCP connection uses organizational SSO and its own authorization. Successful inference with a workspace key does not grant ServiceNow tool access or replace the gateway's upstream OAuth integration.

### Terminal controls and quiet operation

Use arrow keys or Tab to move, Enter to select, or the displayed number shortcuts. Escape cancels. Public fields accept pasted text without submitting it automatically; workspace API keys use a separate hidden prompt. Long authorization links and recovery messages scroll with arrow keys or Page Up/Page Down.

Set `animations = false` under `[tui]` in the environment configuration, or launch with `airs -c tui.animations=false`, for a static mark. `NO_COLOR=1` removes the accent colors. Small terminals use a compact layout. Plain terminals retain text prompts, and explicit scripted commands retain their existing output and exit behavior.

### 4. Open AIRS and check MCP storage for existing environments

**New environments created by 0.1.1:** native MCP storage is already configured. Open AIRS and continue with `/mcp`; no configuration edit is needed. If the OS credential store is unavailable, sign-in fails without saving a plaintext fallback. Restore the credential service before retrying.

**Existing environments and mcp.3:** upgrading preserves the existing mode and saved tokens. To inspect the mode, run `airs env show work`, locate `state_directory`, and inspect that directory's `config.toml`. The explicit native-only setting is this **top-level** key, before any `[table]` headers:

```toml
mcp_oauth_credentials_store = "keyring"
```

Changing this setting alone does **not** move or delete existing tokens. If no MCP credentials have been saved, set it before the first MCP login, updating an existing value rather than adding a duplicate. Migration is optional and the setting applies to the whole environment: sign out every MCP connection with saved credentials, including expired or sign-in-required connections through `/mcp` while its **original storage mode is still configured**, and confirm every sign-out succeeds. Then exit AIRS, change the setting, reopen the environment, and sign into those connections again. If sign-out or cleanup fails, restore service access and finish cleanup before changing modes. Do not copy token files or their contents.

The new default does not change native credential identity: the same OS user's identical MCP connection name and URL can share a native record across environments. Signing out that record can affect those environments; use distinct connection names when separate credentials are needed. Linux needs an available Secret Service session, and macOS may request Keychain authorization.

```sh
airs --environment work
```

The remaining connection workflow stays inside AIRS. It uses the environment displayed by this session, even if another terminal changes the saved default.

### 5. Add ServiceNow and complete MCP SSO inside AIRS

1. Enter `/mcp` to open **MCP connections**.
2. Choose **Add gateway MCP server**. Enter the local connection name `service-now` and `https://gateway-mcp.example.com/mcp-service-now-dev/mcp` as the gateway URL.
3. Complete **Sign in to gateway MCP**. On a local desktop, AIRS opens the browser automatically. Over SSH or without a desktop browser, use **Ctrl+Y** to copy the full authorization link and open it on your laptop or phone. After SSO and consent, the browser may show a localhost page that cannot load: copy its entire address into AIRS's hidden callback field and press Enter. Use **Ctrl+O** to retry opening a browser here and **Page Up/Page Down** to scroll long links. Never paste the callback into the agent conversation or a support report. AIRS shows progress through authorization, credential storage and tool discovery; a saved credential alone does not establish tool access. If the attempt expires, select the saved connection and choose **Sign in** for a fresh link.
4. In the gateway's company SSO flow, choose the **same company account** used for inference. With workspace-key inference, choose the organizational account that has the MCP workspace grant. An existing browser session may avoid another password prompt; account selection or consent can still appear.
5. Complete any gateway-managed upstream consent. Wait for native credential persistence and MCP initialization/tool discovery to finish. After **MCP connection updated**, choose **Start new conversation**.

The process stays open; your previous conversation remains saved and your unsent draft carries over for review. Nothing is submitted or replayed automatically. This transition is required because a changed MCP identity or tool inventory cannot safely be inserted into the old conversation. An opaque gateway MCP token does not prove continuity with the inference identity.

If a connection was saved but sign-in was cancelled or failed, select `service-now` in `/mcp` and choose **Sign in**. Do not add a duplicate. Use **Reconnect and verify** for a fresh connection check; **Refresh connections** refreshes the manager's view. A cached inventory alone is not proof of current tool access.

The URL must be the gateway's integration URL, including `/mcp`, never the direct ServiceNow instance or upstream MCP server. The harness signs into the gateway; the gateway owns upstream OAuth and its confidential client; the MCP service holds the ServiceNow integration credential. Do not copy inference tokens or ServiceNow passwords into MCP configuration.

Shell commands remain available as an optional fallback:

```sh
airs --environment work mcp add service-now \
  --url https://gateway-mcp.example.com/mcp-service-now-dev/mcp \
  --scopes mcp:servers:read,mcp:tools:list,mcp:tools:call
# Only if the add flow did not finish sign-in:
airs --environment work mcp login service-now --no-browser
```

### 6. Inspect health and verify a read-only ServiceNow call

Enter `/doctor` for the current environment's connection-health dashboard. Opening it or choosing **Refresh diagnostics** does not send an inference request. **Verify gateway access** opens a confirmation; **Send connectivity check** sends a small inference request that can consume quota and appear in gateway logs. It sends no local files, conversation content or tools. This verifies inference, not ServiceNow permissions.

**Pending test release:** this branch adds the following diagnostic-report actions; publication and native acceptance are still in progress.

For a support summary, choose **Diagnostic report**, then **Preview report**. The report contains the AIRS version, platform, authentication method, known check outcomes and recovery steps. It omits environment and connection names, addresses, paths, credentials, raw errors and conversation content. It reuses the last completed check; previewing, copying or saving it does not probe the gateway again. An inference check that has not run is **Not verified**.

Choose **Copy report** to send it to your clipboard, or **Save local report** to create a private `diagnostic-report-*.txt` file in this environment's state directory. AIRS displays the saved location. Over SSH, copying depends on your terminal accepting clipboard requests; use the saved file if it does not. Review the report before sharing it. Raw `airs doctor --json` output is a different, detailed local diagnostic and can contain private addresses and paths.

Return to `/mcp`, select `service-now`, and use **Reconnect and verify** if you need fresh initialization/tool discovery. Choose **Start new conversation** after connection changes, then inspect the available tools. A read-only grant exposes `list_incidents` and `get_incident`; incident-management grants may also expose `create_incident` and `update_incident`. Successful login does not imply all four permissions.

Ask in the new conversation:

> Use the service-now MCP connection to list up to five active incidents. Show their numbers, short descriptions and priorities. Do not create or update any records.

Confirm that the transcript actually called `list_incidents` on `service-now` and returned a tool result. An empty authorized list is valid. A connected label, an inventory, or a model answer without a tool call is not end-to-end evidence. This final live check is for your provisioned account; it was not performed by the documentation update.

### 7. Use the bundled Prisma AIRS product CLI and skills

The ServiceNow session above needs no product management credential. If you also manage Prisma AIRS products, configure a separate CLI tenant using credentials supplied for that tenant:

```sh
airs cli tenant create development
airs cli tenant switch development
airs cli --tenant development doctor
airs cli runtime --help
```

`tenant create` prompts for the tenant service group ID, OAuth client ID and a hidden client secret, and saves a private JSON configuration. It does not select the tenant until `tenant switch`. To register an existing JSON file without copying or changing it, use `airs cli tenant create development --config /secure/development.json` instead. Doctor reports which capabilities the configuration supports; it can perform remote probes and is not a guarantee that every product is licensed or authorized.

Harness environments and CLI tenants are independent. `airs env use work` selects the conversation, inference login and MCP connections. `airs cli tenant switch development` selects product credentials. Prefer `airs cli --tenant development ...` in scripts. Put `cli` immediately after `airs`; `airs --environment work cli ...` does not select a product tenant. Company SSO does not supply Prisma AIRS management API credentials, and the CLI does not read old dotenv credentials as a fallback.

In the harness, ask:

> Use the bundled prisma-airs-cli skill to inspect the development tenant's configuration and identify available read-only Prisma AIRS commands. Do not create, change or delete resources.

The skills invoke the private bundled executable, so a global CLI version cannot silently replace it. They cover runtime scanning, AI Gateway, red teaming, model security and related workflows. Check the proposed tenant and operation before authorizing writes. Standalone `airs-cli` and `airs cli` use the same tenant store, so switching the saved CLI tenant affects both entry points.

### Return, switch and recover

List environments with `airs env list`; switch the saved default with `airs env use work`. A bare `airs` then opens that environment. `--environment NAME` selects an environment for one command without changing the saved default. Each environment has its own history, inference identity binding and MCP configuration.

| Symptom | Next step |
| --- | --- |
| Inference login was cancelled | `airs --environment work login`; reuse the environment |
| Gateway MCP login needs renewal | `/mcp` → `service-now` → **Sign in**, then **Start new conversation** |
| Inference succeeds but ServiceNow is absent | `/mcp` in the selected environment, then the gateway URL and workspace integration grant |
| Browser callback says success but the terminal fails | `/doctor`; check native credential-store persistence and keep the terminal open through completion |
| Workspace key needs replacement | Leave the session, run `airs --environment work login --with-api-key`, then reopen the same environment |
| Credential cleanup is pending | Restore credential-service access, then retry the displayed environment-specific login or logout; unavailable does not necessarily mean locked |
| Gateway returns 404 | Check the exact ServiceNow gateway URL and its final `/mcp` |
| Tools return an authorization error | Have an administrator check the gateway workspace grant and upstream incident roles/scopes/subject binding |

For inference SSO renewal, `/doctor` offers **Restore company sign-in**, or use `/signin`. Workspace-key replacement remains a shell operation. Neither operation grants MCP permissions. Escape cancels an unfinished action; existing work remains saved.

If startup or resume stops because the inference refresh was rejected or its outcome is unknown, restore sign-in from the shell with `airs --environment work login --restore-session`, using the affected environment’s name. Sign in as the same person, then retry the original command. This restores access to the saved conversation; an already-open session continues to use `/signin`.

To retire the environment, use `/mcp` → **Sign out** if desired, then sign out the credentials you intend to remove while it is still selected and unregister it:

```sh
airs --environment work mcp logout service-now
airs --environment work logout
airs env remove work
```

`env remove` preserves local files and history and does not itself revoke credentials. If it was the default, select another environment before starting a new session. Recreating the same name creates a fresh namespace, not a reconnection to the preserved history.

The steps above define what to verify. They do not claim that this documentation edit performed a fresh human SSO login or a live ServiceNow tool call. See [implementation evidence](https://cdot65.github.io/prisma-airs-reference-architecture/learn/evidence/) for the recorded deployment and release limits.
