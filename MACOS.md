---
title: Install Prisma AIRS Harness on your Mac
description: Install the app, sign in, and start your first conversation.
release: 0.1.0-alpha.10
updated: 2026-09-09
audience: end-users
platform: macos-arm64
---

# Prisma AIRS Harness on your Mac

This guide is for **Apple Silicon Macs** (M1, M2, M3, M4, or newer).
Check **Apple menu → About This Mac** if you are unsure. Intel Macs are unsupported.

You will need access to the private GitHub package and either your company
sign-in or a workspace API key from your administrator.

## 1. Open Terminal and prepare your Mac

Press **Command + Space**, type **Terminal**, and open it.
Copy each command from its box into Terminal and press **Return**.

If you already have Node.js, Git, and ripgrep installed, skip this step.
Otherwise, install [Homebrew](https://brew.sh/), follow its setup instructions,
and then run:

```sh
brew install node git ripgrep
```

## 2. Download the harness

GitHub requires a **personal access token (classic)** to download this private
package. If you do not have one:

1. Open [GitHub token settings](https://github.com/settings/tokens).
2. Choose **Generate new token → Generate new token (classic)**.
3. Give it a name and expiration, select **read:packages**, and generate it.
4. Keep the token available for the next prompt. Do not share it in chat.

Run this on **each Mac** where you install the harness:

```sh
npm login --auth-type=legacy --registry=https://npm.pkg.github.com
```

Enter your **GitHub username**. At the **Password** prompt, paste the token—not
your GitHub password. It is normal for nothing to appear while you paste.

After login succeeds, install the signed review release:

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.0-alpha.10 --include=optional --registry=https://npm.pkg.github.com
```

Check the installation:

```sh
airs-harness --version
```

You should see `airs-harness 0.1.0-alpha.10`.

## 3. Create your environment and sign in

Create a saved connection named **work**:

```sh
airs-harness setup --environment work --gateway-url https://airs.cdot.io/v1
```

This creates and selects the environment. Run it once, including when starting
over after removing all environments. `--environment` alone selects an existing
environment; it does not create one.

Now sign in:

```sh
airs-harness --environment work login
```

Choose one sign-in method:

### Company sign-in

Choose **1** and enter these values when prompted:

| Prompt | Enter |
| --- | --- |
| Company issuer URL | `https://auth.dev.cdot.io/realms/truffles` |
| Public client ID | `airs-terminal-pilot` |
| Gateway audience | `airs-terminal-inference` |

Complete sign-in in your browser. If it does not open automatically, open the
URL shown in Terminal. These settings are for our deployment; use your
administrator's values if your organization supplied different ones.

### Workspace API key

Choose **2**, paste the workspace key supplied by your administrator, and press
**Return**. The input is hidden. This is a different credential from your GitHub token.

The harness saves your sign-in in macOS Keychain. Allow access if macOS asks.
Look for **Gateway access verified**. Then start the harness:

```sh
airs-harness --environment work
```

Type your first message when the conversation opens.

## 4. Use it again

You do not need to install or sign in every time.

| What you want to do | Command |
| --- | --- |
| Start the harness | `airs-harness` |
| Continue an earlier conversation | `airs-harness resume` |
| See your saved environments | `airs-harness env list` |
| Select the environment named work | `airs-harness env use work` |
| Sign out of the selected environment | `airs-harness logout` |
| Sign back in | `airs-harness login` |

If you used a different environment name, substitute that name for `work`.
To work on files, open Terminal in your project folder before starting the harness.

## Need help?

- **npm reports 401:** repeat npm login on this Mac using an unexpired **classic**
  token with **read:packages**. If it still fails, ask your administrator to check
  package access and send them the error text, without your token.
- **Company sign-in, Keychain, or gateway access fails:** send your administrator
  the error and the output of `airs-harness --version`. Do not send credentials.
- **You want remote security tools:** ask your administrator to help connect MCP
  after basic sign-in works. MCP requires separate authorization.

To uninstall the application:

```sh
npm uninstall -g @cdot65/prisma-airs-harness
```

Uninstalling keeps your saved environments and conversations. Run
`airs-harness logout` before uninstalling if you also want to sign out of the
selected environment.
