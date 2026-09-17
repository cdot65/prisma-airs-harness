---
title: Prisma AIRS Harness candidate setup and sign-in
description: First launch, secure sign-in, resume, logout, and support diagnostics for the private authentication candidate.
status: authentication-review
created: 2026-09-08
updated: 2026-09-09
source_commit: b012ba55e1404b498ad513cd15efa52a9a60530f
tags: [prisma-airs-harness, authentication, onboarding, macos, linux, windows]
---

# Historical authentication review guide

This alpha.10 record is retained for provenance. Current gateway inference and
native MCP onboarding is in [README.md](README.md) and [MACOS.md](MACOS.md).
Its package versions and acceptance status below are historical.

This guide applies to **0.1.0-alpha.10 authentication review builds**. Use the
exact executable or package approved by your project administrator. Existing alpha.9
registry installation instructions describe the historical release and do not
install these changes. A version label alone does not identify a candidate;
retain the administrator's artifact or build reference.

Apple Silicon Mac and Linux desktop are the current candidate validation
targets; native Windows was deferred by the owner in plan v0.2. Native acceptance
is still in progress. The owner's reported Mac storage failure has **not been
proven fixed** on the owner's device. The owner has returned the Developer ID
signed alpha10 binary and an Accepted notarization result; signed-package
acceptance and publication are tracked in [the signing record](https://github.com/cdot65/airs-harness/blob/main/MACOS-SIGNING.md).
Headless onboarding and an organization-profile wizard are not delivered by
this guide.

Open Terminal on your Mac or your Linux desktop terminal, using your normal
signed-in account. After the candidate is installed:

```text
airs
```

On a fresh installation, enter an environment name (Enter accepts `work`) and
the AI Gateway URL supplied by your administrator. For this project's gateway,
use `https://airs.cdot.io/v1`. Setup then offers sign-in. It saves the environment
before authentication, so cancelling sign-in does not require repeating setup.
The local context budget defaults to 1,000,000 tokens; the selected backend's
actual limits still apply.

Choose **1, Company sign-in**, or **2, Workspace API key**:

- Company sign-in initially asks for the public issuer URL, client ID, and
  gateway audience supplied by your administrator. Later attempts show the
  saved settings: press **Enter** to reuse them or **2** to enter changes.
  It opens your browser; if opening
  fails, open the displayed sign-in URL yourself. Enter your company password
  only on the identity provider's browser page. Public connection settings are
  remembered for subsequent login attempts.
- Workspace sign-in provides a hidden input prompt. Paste only the key there,
  never into a command or chat message. Press **Enter on Mac/Linux**, or
  **Ctrl+Enter on Windows**. Windows Enter alone cancels this input attempt;
  rerun login to retry. Esc cancels without saving the entered key. Windows
  terminal delivery of Ctrl+Enter remains a native acceptance gate.

Credentials use macOS Keychain, the Linux credential service, or Windows
Credential Manager. Follow any operating-system authorization prompt. A storage
failure stops sign-in; the application does not silently save plaintext.

Linux desktop testing uses Secret Service on your existing desktop session.
The candidate does not install or start the service. A missing service or an
unlock failure is a limitation to report to your administrator; the complete
in-app recovery flow is still pending. SSH/headless onboarding is outside this
guide.

After guided sign-in saves your credential, the harness announces one small
connectivity request. It sends a fixed message, allows up to 16 output tokens,
and sends no local files or tools. The request asks the provider not to store
the response; your gateway's logging policy still applies.

**Gateway access verified** means that this inference check succeeded. It does
not verify MCP authorization. If the check fails, you see **Credential saved;
gateway access not yet verified** with a safe reason and retry instructions.
The check does not delete your saved credential; it does not guarantee that the
OS store remains accessible. A client correlation ID identifies
the check; it is not a provider-issued response ID.

To retry without signing in again:

```text
airs doctor --verify-access
```

Select the same environment when retrying, for example
`airs --environment work doctor --verify-access`. The check uses the
configured model selection: the gateway default sends no model name, while an
explicit route must appear in that environment's local capability catalog.
Optional MCP access still needs its own harmless tool test.

If you need to sign in again, run:

```text
airs login
```

To add another environment without replacing an existing one, run
`airs env create` and choose a new name. For a different company identity,
workspace key, or gateway, create a new environment so histories stay separated.

## Return in a new terminal

```text
airs env list
airs resume
```

The asterisk in the environment list marks the default. Resume uses its saved
settings and credentials. To choose another listed environment explicitly:

```text
airs --environment work resume
```

A signed-out environment offers guided login in an interactive terminal. If a
saved credential is missing or the OS refuses access, follow the reported
recovery action or run `airs login`. Do not delete configuration or
credential records to work around an error.

## Sign out and collect diagnostics

```text
airs logout
```

Logout invalidates that environment's running AIRS clients and attempts local
credential cleanup. It cannot undo a request already accepted by a server.
Read the result: issuer revocation or native cleanup may remain incomplete.
Retry logout after resolving a reported storage error. Workspace-key revocation
is administered in AIRS; signing out locally does not revoke the shared key.

For support, run these application commands:

```text
airs --version
airs env status
airs doctor
airs doctor --json
```

`status` shows the selected environment and saved authentication configuration.
For OS-stored credentials, it does not open the native credential store or refresh
tokens, and it reports **availability and gateway access not checked**. Saved OIDC
identity information is not a fresh authentication result. Status makes no network
requests. Explicit file/environment references are still checked locally.

Plain `doctor` checks local configuration, tools, credential configuration, and
unauthenticated gateway health. It also avoids native credential reads. Its JSON
`credential_configuration` result describes configuration, not verified access.
Add `--verify-access` for the disclosed inference
check; add `--json` if support needs a machine-readable report. The access check
allows 10 seconds to connect and 30 seconds overall, including credential-helper
resolution. Its response is limited to 64 KiB, and credentials never follow a
redirect. Explicit login commands used by automation do not trigger this probe.
The MCP count does not prove tool permissions. This candidate has no
diagnostic-bundle wizard.

Share the candidate build reference, OS and terminal versions, failing command,
and the storage error's **operation, category, and OS status**. An unavailable
status means the backend supplied no recognized native code; it does not prove
that the store is locked. Review diagnostic output for organizational URLs or
local paths before sharing. Never include a workspace key, JWT, company password,
credential file, or browser callback URL.

The remaining release requirements are tracked in
[the authentication plan](https://github.com/cdot65/airs-harness/blob/main/AUTHENTICATION-PLAN.md).
