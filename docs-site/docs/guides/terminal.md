---
title: Terminal workflow and routing
---

This page has two parts. The first explains how the terminal session behaves: what
the agent can do on its own, how model routing is decided, and what the
verification checks send. The second is the practical part: commands, keys and
settings for everyday work.

## How the terminal session works

The agent can read files and execute local tools under its sandbox and approval
policy. Approval is where you decide what runs, so review the requested command
and its access before approving it. Linux requires a working Bubblewrap sandbox;
missing namespace support is an error, not a fallback to running unsandboxed.

**Routing has two settings.** A saved gateway config selects a route, and a model
selects what runs on it. Changing the config resets the model to follow that
route. An explicit model override pins the model instead, and choosing the default
follows the selected config again. The gateway decides authorization and the
effective model, so a choice the menu offers is not a guarantee that the request
will be allowed.

**Where a routing choice applies.** Routing is scoped to the conversation. Saved
conversations restore their routing, while new conversations use the environment
defaults. That is why `/doctor` checks the environment defaults and not a
conversation's override. To verify an override, reselect the current route.

**What verification sends.** Opening either menu sends no request. **Verify and
use** sends a fixed, bounded inference check without conversation text, files or
tools, though quota and gateway logging can apply. A failed or cancelled check
keeps the previous routing pair, so a bad selection does not replace a working one.

**Inline or fullscreen.** Inline is the default display. Fullscreen is opt-in per
environment and takes effect after AIRS restarts.

## Use the terminal

### Start, resume and script

Run `airs` in your project. Use `airs resume` to select a saved conversation or
`airs resume --last` to continue the most recent one. For a scripted task, use
`airs exec 'Inspect this project and run its tests.'`.

### Everyday controls

Use arrows or Tab to move, Enter to select and Escape to cancel or interrupt.

| Command | What it does |
| --- | --- |
| `/compact` | Summarizes a long thread |
| `/doctor` | Inspects connection health |
| `/signin` | Restores inference login |
| `/mcp` | Manages remote tools |
| `/typesafe` | Manages the optional judge key |
| `/config` | Selects a saved gateway config for this conversation |
| `/model` | Selects a model for this conversation |
| `/tui` | Chooses fullscreen per environment |

Set `tui.animations=false` for quiet operation; `NO_COLOR=1` removes accent colors.

### Skills

Local skills live in `.agents/skills/<name>/SKILL.md` or the selected environment's
skills directory. Invoke a skill with `$name`. The managed product CLI and embedded
skills are described in [Provision a workspace with the bundled CLI](../generated/bundled-cli.md).

### Select a config and model

`/config pc-example-123456` selects a saved gateway config; `/config default`
returns to gateway defaults. `/model @integration/model` chooses an explicit model
override; `/model default` follows the selected config. Choose **Verify and use** in
the menu to run the bounded check before switching.

### Use fullscreen and search

Enter `/tui` to choose fullscreen for the current environment, then restart AIRS.
In fullscreen, F3 opens search and F4 controls activity detail.

See [release channels](releases.md) for how to check which release you are
running and how to install a different one.
