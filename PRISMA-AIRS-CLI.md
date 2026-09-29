# Provision a workspace with the bundled CLI

This guide is for the gateway administrator. It takes you from an empty Prisma AIRS
tenant to a workspace that users can sign into: a service account, a CLI tenant, a
workspace, a provider integration, and a saved config that routes to a model. It
ends by handing the workspace to a user, who follows
[Getting started](GETTING-STARTED.md), and by checking their traffic from the
terminal.

It assumes a sandbox tenant. Every step here creates real resources, and the
service account in step 1 is deliberately broad. Do not use these settings as they
are in a production tenant.

## How the bundled CLI and provisioning work

**Two commands, two jobs.** One npm installation of the harness includes the Prisma
AIRS CLI and its embedded product skills. `airs` starts the harness, and
`airs cli ...` runs the bundled CLI. A separately installed product CLI uses
`airs-cli ...`, and the harness never substitutes it for the bundled one. Put `cli`
immediately after `airs`, because a harness `--environment` flag does not reach the
CLI.

**A tenant is not an environment.** A harness environment is a local profile for
inference: a gateway URL, an inference credential and a conversation history. A CLI
tenant holds the product credentials, meaning a service group ID, an OAuth client ID
and its secret. `airs env use` and `airs cli tenant switch` are independent
selections, and company SSO or a workspace key does not stand in for tenant
credentials. That is why the harness works before any tenant exists, and why an
administrator needs both.

**Three separate grants.** Being able to provision a workspace is not the same as
being able to use it. The management grant lets your service account create
resources. A user's identity role lets them call the gateway, and workspace
membership decides which workspace they land in. Granting one does not grant the
others, so when a user is denied, ask which of the three is missing.

**A workspace is a chain, not a record.** Each link is created by a different
command and identified differently, and a later step usually needs an identifier
that an earlier one printed.

| Resource | Made by | What later steps need |
| --- | --- | --- |
| IAM scope, then workspace | `aigateway workspaces create` | Workspace UUID for bindings and configs, workspace slug for telemetry, and the scope name |
| Provider integration | `aigateway integrations create` | Integration UUID and its slug |
| Workspace binding and models | `integrations workspaces set`, `integrations models` | Model slug |
| Saved config | `aigateway configs create` | The config's slug, which users and tokens reference |
| User workspace key | `aigateway api-keys user create` or the console | Handed to the user once |

Do not substitute one identifier for another. The workspace UUID, the workspace
slug and the scope name are different values, and commands are picky about which
they take. Relationship commands take the UUID, and telemetry takes the slug.

**Why the order matters.** A user's key gives access, but it does not create a route
to a model. The route is the saved config, and it needs a provider integration that
is bound to the workspace, which in turn needs the workspace to exist. Provisioning
in the order below means each step has what it depends on.

**Skills.** The embedded skills cover diagnosis, Runtime Security, Guardrail
Generation, Red Teaming, AI Gateway, Model Security, DLP Testing and DLP Management.
They run the bundled CLI by its absolute path, so a change to your shell's `PATH`
cannot select another version. A skill changes what the agent tries to do, not what
it is allowed to do, so the credentials and approvals in this guide still decide the
outcome.

## Set it up

### 1. Create a service account in Strata Cloud Manager

The CLI authenticates as a service account. In Strata Cloud Manager:

1. Open **System Settings**, then **Identity & Access Management**.
2. Choose **Add Identity** and select **Service Account** from the dropdown.
3. Enter a client ID as the username.
4. Download the OAuth client credentials when they are shown. You cannot retrieve
   the secret again later.
5. Set the permissions to **All apps and services** with the role **Superuser**, and
   leave the scope blank.

That role is broad on purpose, so that you can focus on the workflow. In a real
tenant, give the service account only what these commands need. A narrower role also
needs the workspace's scope granted to it, and step 3 shows how to find that name.

Keep the downloaded credentials somewhere private, never in a repository or a chat.

### 2. Create a tenant and check it

`tenant create` prompts for the tenant service group (TSG) ID, the OAuth client ID
and the client secret, which is hidden as you type. It saves a private configuration
but does not select it, so `tenant switch` is a separate step:

```sh
airs cli tenant create sandbox
airs cli tenant switch sandbox
airs cli --tenant sandbox doctor
```

For automation, `--tsg-id`, `--client-id` and `--client-secret-stdin` supply the same
values without a prompt, and `--config <path>` registers an existing JSON file
without copying it. Never paste a secret into a command argument or a chat.

Doctor reports pass, warn, fail or skip for the tenant's configuration, credentials
and API connectivity. It is a preflight and not proof that every operation is
permitted, so a pass here means the credentials work, not that you can create a
workspace. Prefer an explicit `--tenant` in scripts, because the saved selection can
change under you.

### 3. Create the workspace

This step creates resources. `workspaces create` does three things in order: it
creates an IAM scope, creates the workspace with that scope name, and binds the scope
so the workspace gets data-plane access.

```sh
airs cli --tenant sandbox aigateway workspaces create \
  --name agent-sandbox \
  --description 'Sandbox agent inference' --output json
```

Record the returned workspace UUID, slug and scope name, and set them as variables
for the steps that follow:

```sh
AIRS_DOCS_WORKSPACE_ID='11111111-1111-4111-8111-111111111111'
AIRS_DOCS_WORKSPACE_SLUG='replace-with-returned-workspace-slug'
```

Confirm the workspace and its binding:

```sh
airs cli --tenant sandbox aigateway workspaces list --plane admin --output json
airs cli --tenant sandbox aigateway workspaces list --output json
```

The first list reads the admin plane, and the second reads the data plane, which
only shows workspaces your service account can see. If the workspace appears in the
first but not the second, the service account lacks the role for that workspace's
scope name. Add that role scope in Strata Cloud Manager under Access Management. With
the Superuser role from step 1 you should not need to.

### 4. Add a provider integration

List the providers your tenant offers, then create an OpenAI integration. The
provider credential is requested in a hidden prompt, so it stays out of your shell
history:

```sh
AIRS_DOCS_ORG_ID='1234567890'

airs cli --tenant sandbox aigateway integrations providers

airs cli --tenant sandbox aigateway integrations create \
  --organisation-id "$AIRS_DOCS_ORG_ID" --ai-provider open-ai \
  --name agent-models --slug agent-models

airs cli --tenant sandbox aigateway integrations list --output json
```

`--key-file` and `--key-stdin` also supply the credential for automation. Avoid
`--key`, which puts it in shell history. Record the integration UUID that the list
returns, then bind only this workspace to it:

```sh
AIRS_DOCS_INTEGRATION_ID='22222222-2222-4222-8222-222222222222'

airs cli --tenant sandbox aigateway integrations workspaces set \
  "$AIRS_DOCS_INTEGRATION_ID" \
  --workspace-binding "$AIRS_DOCS_WORKSPACE_ID=true" --preserve-existing

airs cli --tenant sandbox aigateway integrations workspaces list \
  "$AIRS_DOCS_INTEGRATION_ID" --output json
```

`--preserve-existing` matters. Without it, `workspaces set` replaces the
integration's entire set of workspace bindings, and every workspace you did not name
loses access. With it, the command changes only the workspace you name.

Then see which models the integration exposes:

```sh
airs cli --tenant sandbox aigateway integrations models list \
  "$AIRS_DOCS_INTEGRATION_ID" --output json
```

Choose one model slug. `integrations models set` states the complete set of enabled
models, so name every model that should stay enabled, not just the one you are adding.

### 5. Create the saved config

The saved config is the route from a request to a model. It points a target at the
integration by its slug, prefixed with `@`, and pins the model for that target:

```sh
airs cli --tenant sandbox aigateway configs create \
  --name agent-default \
  --workspace "$AIRS_DOCS_WORKSPACE_ID" \
  --set-string 'config.targets[0].provider=@agent-models' \
  --set-string 'config.targets[0].override_params.model=<model-slug>' \
  --output json

airs cli --tenant sandbox aigateway configs list \
  --workspace "$AIRS_DOCS_WORKSPACE_ID" --output json
```

Record the config's slug, since users and identity tokens reference it. Provider
model names and policy schemas depend on the deployed gateway, so confirm the config
looks the way you expect in the list output before you continue.

### 6. Hand the workspace to a user

The user creates their own workspace API key and follows
[Getting started](GETTING-STARTED.md). Give them the gateway inference URL, the name
of the workspace, and the saved config that the key should carry. Do not send them
your tenant credentials, and keep your provider credential out of the handoff.

If you would rather create the key for them, `aigateway api-keys user create` takes
`--workspace`, `--user-id`, `--name`, `--expires-at` and
`--scopes completions.write`. Add `--secret-output <path>` to write the one-time
secret to a new file that only you can read, instead of printing it. Attach the saved
config to the key in the console, as Getting started describes.

### 7. Check their traffic

After the user's first request, look at it from the CLI. Relationship commands use the
workspace UUID, but telemetry uses the slug:

```sh
airs cli --tenant sandbox aigateway telemetry requests \
  --workspace "$AIRS_DOCS_WORKSPACE_SLUG" --days 1
airs cli --tenant sandbox aigateway telemetry logs list \
  --workspace "$AIRS_DOCS_WORKSPACE_SLUG" --page-size 50
```

Creating a resource does not establish a successful model request. The request in
the logs, from the user's own key, is what tells you the whole chain works.

## Add guardrails

Guardrails exist at two levels, and the command tells you which. `aigateway
guardrails` manages guardrails for one workspace. `aigateway admin-guardrails`
manages organization-wide ones on the admin plane, and it has no workspace fallback.
Start by listing what is available, because the checks you can enable come from the
catalog and not from a fixed list:

```sh
airs cli --tenant sandbox aigateway guardrails catalog
airs cli --tenant sandbox aigateway guardrails list \
  --workspace "$AIRS_DOCS_WORKSPACE_ID"
```

A guardrail is a set of checks and the actions to take when they trigger. Take the
check identifiers from the catalog output, and set each field explicitly:

```sh
airs cli --tenant sandbox aigateway guardrails create \
  --name deny-risk \
  --workspace "$AIRS_DOCS_WORKSPACE_ID" \
  --set 'checks[0].id=<check-id-from-catalog>' \
  --set actions.deny=true
```

Test a guardrail with input that should be blocked and input that should not, before
users depend on it. A block shows up as HTTP 446, or as a blocking hook inside an
HTTP 200, so read the policy result and not only the status.

## Run a skill from the harness

Once a tenant is selected, ask the harness to use a skill against it. Start with a
read-only request:

> Use $prisma-airs-cli to check the bundled version and selected tenant, then
> diagnose missing settings without revealing credentials or modifying configuration.

Skills require complete scan evidence before they treat an Allow summary as success.
Guardrail writes and rollback can partially fail, so preserve your baseline policy
and verify the actual state afterward.

## If an old CLI owns the command

If a previously installed standalone product CLI already owns `airs`, upgrade it
first, since it moves to `airs-cli`. Do not force npm to overwrite another package's
command. Then check what your shell resolves:

```sh
type -a airs airs-cli airs-harness
airs --migration-check
```

`airs --migration-check` reports the entries on `PATH` and the packages that own them,
without executing those commands or changing anything. Inspect shell aliases and
functions separately, since it cannot see them. Open a fresh shell after any change.
A standalone `airs-cli` and `airs cli` share the same tenant store, so switching the
saved CLI tenant in one changes the other.
