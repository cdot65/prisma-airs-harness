---
title: Terminal workflow and routing
---

Run `airs` in your project. Use `airs resume` to select a saved conversation or
`airs resume --last` to continue the most recent one. For a scripted task, use
`airs exec 'Inspect this project and run its tests.'`.

The agent can read files and execute local tools under its sandbox and approval
policy. Review the requested command and its access before approving it. Linux
requires a working Bubblewrap sandbox; missing namespace support is an error.

## Everyday controls

Use arrows or Tab to move, Enter to select and Escape to cancel or interrupt.
`/compact` summarizes a long thread. `/doctor` inspects connection health,
`/signin` restores inference login, `/mcp` manages remote tools and `/typesafe`
manages the optional judge key. Set `tui.animations=false` for quiet operation;
`NO_COLOR=1` removes accent colors.

Local skills live in `.agents/skills/<name>/SKILL.md` or the selected environment's
skills directory. Invoke a skill with `$name`. The managed product CLI and embedded
skills are described in [Bundled CLI and skills](../generated/bundled-cli.md).

## Preview: config and model selection

The Apple Silicon `mac-preview` release includes conversation-scoped `/config`
and `/model` routing. These controls are not part of stable 0.1.2.

`/config pc-example-123456` selects a saved gateway config; `/config default`
returns to gateway defaults. Changing config resets the model to follow its
route. `/model @integration/model` chooses an explicit model override;
`/model default` follows the selected config.

Opening either menu sends no request. **Verify and use** sends a fixed bounded
inference check without conversation text, files or tools; quota and gateway
logging can apply. Failed or cancelled checks retain the prior routing pair.
Saved conversations restore their routing, while new conversations use environment
defaults. `/doctor` checks those environment defaults; reselect the current route
to verify a conversation override. The gateway decides authorization and the
effective model.

## Preview: fullscreen and search

The alpha.6 and later Apple Silicon previews offer `/tui` to choose fullscreen
per environment after restarting AIRS. Inline remains the default. F3 opens
search and F4 controls activity detail. See [release channels](releases.md) for
the install and rollback commands and [alpha.6 evidence](../generated/terminal-preview.md)
for the recorded validation limits.
