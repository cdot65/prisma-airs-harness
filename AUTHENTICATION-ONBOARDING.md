---
title: Prisma AIRS Harness candidate setup and sign-in
description: First launch, secure sign-in, resume, logout, and support diagnostics for the private authentication candidate.
status: private-candidate
created: 2026-09-08
updated: 2026-09-08
source_commit: c10e3f993
tags: [prisma-airs-harness, authentication, onboarding, macos, linux, windows]
---

# Try the private authentication candidate

This guide describes an **unpublished private candidate**. Use the candidate
executable or package supplied by your project administrator. Existing alpha.9
registry installation instructions describe the historical release and do not
install these changes. A version label alone does not identify a candidate;
retain the administrator's artifact or build reference.

Apple Silicon Mac, Linux desktop, and Windows are the candidate validation
targets. Native acceptance is still in progress. The owner's reported Mac
storage failure has **not been proven fixed**. Developer ID/notarization,
Windows signing, headless onboarding, and an organization-profile wizard are
not delivered by this guide.

Open Terminal on your Mac, your Linux desktop terminal, or PowerShell in Windows
Terminal, using your normal signed-in account. After the candidate is installed:

```text
airs-harness
```

On a fresh installation, enter an environment name (Enter accepts `work`) and
the AI Gateway URL supplied by your administrator. For this project's gateway,
use `https://airs.cdot.io/v1`. Setup then offers sign-in. It saves the environment
before authentication, so cancelling sign-in does not require repeating setup.
The local context budget defaults to 1,000,000 tokens; the selected backend's
actual limits still apply.

Choose **1, Company sign-in**, or **2, Workspace API key**:

- Company sign-in asks for the public issuer URL, client ID, and gateway
  audience supplied by your administrator. It opens your browser; if opening
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
Successful sign-in stores credentials; it **does not verify gateway access or
MCP authorization**. Start with a harmless chat request, such as “Reply with
ready,” to exercise inference. Optional MCP access needs its own tool test.

If you need to sign in again, run:

```text
airs-harness login
```

To add another environment without replacing an existing one, run
`airs-harness setup` and choose a new name. Changing the user, workspace key, or
gateway must respect the existing environment's history boundary.

## Return in a new terminal

```text
airs-harness env list
airs-harness resume
```

The asterisk in the environment list marks the default. Resume uses its saved
settings and credentials. To choose another listed environment explicitly:

```text
airs-harness --environment work resume
```

A signed-out environment offers guided login in an interactive terminal. If a
saved credential is missing or the OS refuses access, follow the reported
recovery action or run `airs-harness login`. Do not delete configuration or
credential records to work around an error.

## Sign out and collect diagnostics

```text
airs-harness logout
```

Logout invalidates that environment's running AIRS clients and attempts local
credential cleanup. It cannot undo a request already accepted by a server.
Read the result: issuer revocation or native cleanup may remain incomplete.
Retry logout after resolving a reported storage error. Workspace-key revocation
is administered in AIRS; signing out locally does not revoke the shared key.

For support, run these application commands:

```text
airs-harness --version
airs-harness doctor
airs-harness doctor --json
```

Doctor checks local configuration, tools, credential availability, and gateway
health. Its health probe does not authenticate inference, and its MCP count
does not prove tool permissions. This candidate has no diagnostic-bundle wizard.

Share the candidate build reference, OS and terminal versions, failing command,
and the storage error's **operation, category, and OS status**. An unavailable
status means the backend supplied no recognized native code; it does not prove
that the store is locked. Review diagnostic output for organizational URLs or
local paths before sharing. Never include a workspace key, JWT, company password,
credential file, or browser callback URL.

The remaining release requirements are tracked in
[the authentication plan](AUTHENTICATION-PLAN.md).
