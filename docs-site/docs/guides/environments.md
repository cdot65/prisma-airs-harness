---
title: Environments and credentials
---

An environment is a local gateway profile with configuration, credential bindings,
MCP connections and conversation history. A gateway workspace is a server-side
access and routing boundary. Their names do not need to match.

```sh
airs env create work --gateway-url https://gateway.example.com/v1
airs --environment work login
airs env list
airs env show work
airs env status work
airs --environment work doctor --verify-access
airs env use work
```

Creation with `--gateway-url` saves and selects the environment without starting
sign-in. Omit the flag for guided creation and sign-in. `env use` changes the
default for new processes; `--environment` selects one invocation. Running
conversations retain their existing environment.

`env status` reports local state. `doctor --verify-access` sends a small inference
probe and can consume gateway quota. Neither proves a remote tool works. After
MCP sign-in, verify an actual read-only tool result.

## Storage and identity

Inference SSO uses the native OS credential store: Keychain on macOS and an
unlocked Secret Service session on Linux. New environments also use native MCP
storage. Existing storage modes remain unchanged on upgrade. Follow the
[migration steps](../generated/getting-started.md#4-open-airs-and-check-mcp-storage-for-existing-environments)
before changing an existing MCP storage mode.

Inference and MCP credentials are separate. Native MCP keyring records with the
same server name and URL can be shared by the same OS user across environments;
use different connection names when separate MCP credentials are needed.

State defaults to `~/.airs-harness`; `AIRS_HARNESS_HOME` selects an absolute
alternative. Legacy `~/.airs-terminal` state is reused when it is the existing
home. Project configuration uses `.airs-harness/config.toml`, with a legacy
`.airs-terminal` fallback. Gateway, credential, capability and MCP bindings
belong to the selected environment and cannot be replaced by project overrides.

## Rename, remove and recover

`airs env rename work team` retains the environment UUID, credentials and history.
`airs env remove team` unregisters the name and preserves files. It does not revoke
credentials; recreating the name creates a fresh namespace. Choose a different
default when retiring the current one.

Use `airs --environment work login --restore-session` if a rejected or uncertain
inference refresh prevents startup or resume. Sign in as the same person. Within
an open conversation, use `/signin`. Use `/mcp` separately for MCP recovery.

See the generated [environment commands](../generated/reference/env.md) and
[login options](../generated/reference/login.md).
