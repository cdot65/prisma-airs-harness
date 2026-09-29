# Getting started with Prisma AIRS Harness

By the end of this page you will have installed the harness, signed it into
Prisma AIRS AI Gateway with a workspace API key, confirmed the connection with
`/doctor`, sent your first request, and found that request in the gateway's
observability console. That is the smallest setup that shows the whole path
working, from your terminal to the gateway and back.

It uses a workspace API key because that is the fewest moving parts. There is no
identity provider to configure and no browser sign-in. You create one credential
in the gateway, paste it into the harness, and the workspace it belongs to decides
what you are allowed to do. Company SSO and remote tools come later, in the
[SSO and ServiceNow walkthrough](SSO-SERVICENOW.md).

## Before you start

You need three things.

- **A supported machine.** Apple Silicon, Linux x64 or Linux ARM64, with Node.js
  `^22.13.0 || >=23.5.0`. Windows and Intel Mac are not supported. Linux also
  needs a usable Bubblewrap sandbox and an unlocked Secret Service session, and
  macOS needs access to your Keychain. Both are where the harness stores your key.
- **Access to a gateway workspace.** Your administrator should have provisioned a
  model provider and a default saved config in it. A key gives you access, but it
  does not create a route to a model, so without that config your first request
  has nowhere to go.
- **The gateway inference URL.** Your administrator provides it. The examples below
  use `https://gateway.example.com/v1`, which is not a live gateway.

Have Git and ripgrep installed as well, since the agent uses them when it works
with a project.

## 1. Install the harness

Check Node first, in the terminal you will use:

```sh
node --version
npm --version
```

If Node is older than the range above, install a supported version using your
organization's method, then reopen the terminal and check again. Installing npm on
Ubuntu does not upgrade a distro-provided Node 18.

Then install the harness:

```sh
npm install -g airs-harness
airs --version
airs cli --version
```

The npm package is `airs-harness` and the command is `airs`. The Prisma AIRS CLI is
bundled, and `airs cli --version` shows which one you have, so you do not install a
separate product CLI. If your organization distributes through its own registry,
add `--registry=<registry URL>` to the install command.

If `airs` reports that its native package is unavailable, reinstall with
`--include=optional`. That happens only when your npm configuration omits optional
dependencies, which is where the native package for your machine lives.

## 2. Create a workspace API key

You create this key in the gateway, not in the harness.

1. Open Prisma AIRS AI Gateway in Strata Cloud Manager and select your tenant and
   the gateway workspace you were given. The gateway is under **AI Security → AI
   Gateway**. The [vendor configuration guide](https://docs.paloaltonetworks.com/ai-runtime-security/administration/configure-ai-gateway)
   describes the console in more detail.
2. In that workspace's API-key management, create a **user workspace API key** for
   yourself. Give it inference permission (`completions.write`) and the expiration
   and limits your administrator requires. A provider integration key or an AIRS
   scanner key is a different credential and will not work here.
3. Attach the administrator-approved **default saved config** to the key. The
   config selects the provider and model your requests use.
4. Copy the key when it is issued. You will paste it in the next step, and you
   should not put it in a command argument, your shell history or a chat message.

## 3. Start AIRS and connect

Run `airs` and answer the prompts. On a fresh installation it opens a welcome
screen and walks you through connecting, so there are no setup flags to remember:

```sh
airs
```

1. **Welcome to Prisma AIRS.** Choose **Connect an environment**.
2. **Environment name.** Accept `work`, or type another short name. An environment
   is only a local profile: where to connect, which credential to use and where your
   conversation history lives.
3. **AI Gateway URL.** Enter the inference API root your administrator gave you, such
   as `https://gateway.example.com/v1`.
4. **Create this environment?** Choose **Create environment and sign in**.
5. **Sign in to continue.** Choose **Use a workspace API key**, then paste the key
   into the hidden field and press Enter.

The screen never asks you to pick a gateway workspace, because the key you created in
step 2 already belongs to one. The workspace decides what that key may do, which is why
its name does not need to match your environment's. The harness stores the key in your
operating system's credential store, and **Credential saved** means that storage
worked. It does not yet mean the gateway accepted the key. The next step checks that.

From now on, running `airs` opens this environment. You only need the
`--environment` flag when you keep more than one, which
[Environments](https://cdot65.github.io/prisma-airs-harness/guides/environments/) covers.

## 4. Validate the connection with /doctor

Once AIRS opens, enter `/doctor`. It shows the health of this environment's
connection. Opening the view and refreshing it send nothing to the gateway. To test
the connection itself, choose **Verify gateway access**, then **Send connectivity
check**.

That check sends one small inference request and reports whether the gateway
accepted it. It sends no local files, conversation content or tools. It can use a
little gateway quota and it will appear in the gateway logs, which is useful in the
next step. **Gateway access verified** means your key, your workspace and your
route all worked for one real request.

You can run the same check from the shell:

```sh
airs doctor --verify-access
```

If the check fails, the status tells you which stage broke:

| You see | Look at |
| --- | --- |
| Credential storage fails | The operating system credential service, such as an unlocked keyring on Linux |
| HTTP 401 | Whether the key is valid and unexpired |
| HTTP 403 | The key's permission, and whether the workspace grants you access to the route |
| HTTP 446, or a blocked result inside HTTP 200 | A gateway security policy blocked the request |
| Access is granted but the model request fails | The default saved config, its provider binding, the model and your quota |

Give your administrator the trace ID from the failure, and never the key itself.

## 5. Send your first request and find it in the gateway logs

In the same session, ask for something short and harmless. If you closed AIRS, run
`airs` again to reopen it:

> Reply with one short sentence confirming you received this.

The answer should stream back. If it does, the whole path worked: your prompt went
through the harness to the gateway, the gateway applied its policy and forwarded
the request to the model provider using the provider credential it holds, and the
response came back the same way.

Now look at the other side of that request. In Strata Cloud Manager, return to
Prisma AIRS AI Gateway, select the same tenant and workspace, and open the
observability view for the workspace. The exact menu names can differ between
console versions, so look for the gateway's logs or observability area. Find your
request by its time. You should see the connectivity check from step 4 and the
prompt you just sent as separate entries.

Seeing the request there is what confirms the gateway is recording your traffic.
The harness only knows what came back to your terminal. The gateway logs show what
it received, what policy decided and what it forwarded.

## What to do next

You now have the working baseline, and everything more advanced builds on it.

- **Sign in with company SSO and connect remote tools** such as ServiceNow, in the
  [SSO and ServiceNow walkthrough](SSO-SERVICENOW.md).
- **Keep separate accounts, destinations and histories** with more than one
  environment, in [Environments](https://cdot65.github.io/prisma-airs-harness/guides/environments/).
- **Look up a command** in the [command cheat sheet](https://cdot65.github.io/prisma-airs-harness/operations/cheat-sheet/).
- **Understand how a request is authorized and routed** in
  [Workspace, models and API keys](https://cdot65.github.io/prisma-airs-harness/configuration/gateway/).
