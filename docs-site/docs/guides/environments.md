---
title: Environments and credentials
---

This page has two parts. The first explains what an environment is and where its
credentials and state live. The second is the procedure: create, select, inspect,
rename, remove and recover environments.

## How environments work

An environment is a local gateway profile with configuration, credential bindings,
MCP connections and conversation history. A gateway workspace is a server-side
access and routing boundary. The environment records where to connect and with
which credential; the workspace decides what that credential is allowed to do.
Because they live in different places, their names do not need to match.

**Two credentials, kept apart.** Inference and MCP credentials are separate. The
inference login is stored in the native OS credential store: Keychain on macOS and
an unlocked Secret Service session on Linux. New environments also use native MCP
storage. Existing storage modes remain unchanged on upgrade.

**Shared MCP records.** Native MCP keyring records with the same server name and
URL can be shared by the same OS user across environments. Signing out of one
can therefore affect another. Use different connection names when you need
separate MCP credentials, because a different environment name alone does not
isolate the record.

**Where state lives.** State defaults to `~/.airs-harness`; `AIRS_HARNESS_HOME`
selects an absolute alternative. Legacy `~/.airs-terminal` state is reused when
it is the existing home. Project configuration uses `.airs-harness/config.toml`,
with a legacy `.airs-terminal` fallback.

**What a project cannot change.** Gateway, credential, capability and MCP bindings
belong to the selected environment and cannot be replaced by project overrides,
so a repository you open cannot point your credentials at a different gateway.

**What selects an environment.** Most people never need to choose. The first time you
run `airs`, its welcome screen creates an environment for you and signs you in, and
after that `airs` opens it. `env use` changes the default for new processes when you
keep several, and `--environment` selects one invocation without changing it. A
running conversation keeps the environment it started in, so changing the default does
not move a session that is already open.

## Work with environments

### Create, sign in and inspect

To create your first environment, run `airs` and answer the welcome screen: an
environment name, the gateway URL, and then company SSO or a workspace API key. The
commands below do the same thing without the screen, for scripts and for adding more
environments:

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs login
airs env list
airs env show work
airs env status work
airs doctor --verify-access
airs env use work
```

Creation with `--gateway-url` saves and selects the environment without starting
sign-in. Omit the flag for guided creation and sign-in. Commands act on the default
environment. Add `--environment NAME` to run one command against another.

`env status` reports local state. `doctor --verify-access` sends a small inference
probe and can consume gateway quota. Neither proves a remote tool works. After
MCP sign-in, verify an actual read-only tool result.

Follow the
[migration steps](../generated/getting-started-mcp.md#environments-created-before-native-mcp-storage)
before changing an existing environment's MCP storage mode.

### Rename and remove

`airs env rename work team` retains the environment UUID, credentials and history.
`airs env remove team` unregisters the name and preserves files. It does not revoke
credentials; recreating the name creates a fresh namespace. Choose a different
default when retiring the current one.

### Change an environment's credential

Use this when a workspace API key is rotated, when you move from company SSO to a
workspace key (or back), or when a different company user should own the
environment. The environment keeps its name, gateway, MCP connections and saved
conversations; only the inference credential changes.

```sh
airs env auth work
```

The guided flow shows how the environment signs in today and offers the matching
choices: replace the workspace API key, switch to company SSO, sign in again as the
same SSO user, change the SSO user or settings, switch to a workspace key, or test
the current credential. Running `airs --environment work login` on an environment
that is already signed in opens the same menu. Before anything is replaced it asks
for confirmation. It then saves the new credential, removes the previous one from
this device's secure storage, and sends one minimal inference request to confirm
gateway access.

Replacing a credential has these effects:

- Open AIRS sessions in the environment stop; restart them with `airs resume`.
- Saved conversations stay in the environment and continue with the new credential.
- The old workspace key still works at the gateway until you revoke it there.
- A replaced company sign-in is revoked at the issuer when it can be reached.
- MCP servers set up with company sign-in (`setup-mcp --issuer-url`) need a fresh
  sign-in as the new user: repeat `airs setup-mcp` with the same options.

For automation, pass `--replace` with an explicit credential source. Without
`--replace`, a different credential is refused and nothing changes:

```sh
# New workspace key from a file or pipe (stored in the OS credential store).
airs --environment work login --replace --with-api-key < new-key.txt

# Or reference an owner-only file or an environment variable.
airs --environment work login --replace --credential-file /secure/new-key
airs --environment work login --replace --credential-env AIRS_WORK_KEY

# Switch to company SSO.
airs --environment work login --replace \
  --issuer-url https://id.example.com/realms/company \
  --oidc-client-id airs-harness --audience airs-gateway

# Confirm access again at any time.
airs --environment work doctor --verify-access
```

With `--replace`, `login` runs the same access test and exits nonzero if the
gateway does not accept the new credential. The credential is still saved, so fix
gateway access and rerun `doctor --verify-access`. Within an open conversation,
`/signin` → **Change key or sign-in method** shows these commands for the current
environment.

### Recover a sign-in

Use `airs login --restore-session` if a rejected or uncertain
inference refresh prevents startup or resume. Sign in as the same person. Within
an open conversation, use `/signin`. Use `/mcp` separately for MCP recovery.

See the generated [environment commands](../generated/reference/env.md) and
[login options](../generated/reference/login.md).
