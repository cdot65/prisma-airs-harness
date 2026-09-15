# Alpha.17 owner testing publication — September 15, 2026

Alpha.17 is published for Linux x64 and Apple Silicon at https://npm.cdot.io under
`gateway-validation`. Regular release tags are unchanged.

```sh
npm install -g airs-harness@0.1.0-alpha.17 --registry=https://npm.cdot.io
```

The owner authorized this increment with “okay, do it.” Runtime source is frozen
at `26bcae7799ae13c592236efa046a99dbe35e4f9b`.

## Behavior

MCP sign-in can now be started in the terminal interface, through `/signin` or the
MCP authentication prompt. The harness runs its native OAuth login, saves the
credential using the configured native store, reloads MCP configuration and
checks a new initialized gateway connection before displaying a tool count.
Cancellation, timeout, duplicate attempts and stale conversation completions are
handled without automatically replaying a request or a completed tool.

After successful MCP consent, the user can explicitly start a new conversation
inside the application and carry over the unsent draft for review. The previous
conversation stays saved. The current gateway metadata does not establish a
trusted frontend account-continuity contract, so same-conversation MCP restoration
remains unsupported. Successful OAuth alone is not treated as identity proof.

A definitive inference refresh rejection is now persisted as a tokenless
sign-in-required state. Later credential reads retain that reason instead of
misreporting it as an unknown renewal. Unknown exchange outcomes still discard
the predecessor and require sign-in. The user's two reported prompts followed
more than 30 minutes idle; the existing 30-minute policy is unchanged.

## Evidence and limits

The attached receipts bind build, signing, notarization, native Keychain,
installation and upgrade results to the exact native bytes. Log hashes cover the
CLI/login and terminal regression suites, snapshot checks and scoped Clippy.
Upgrades are exercised from alpha.16 on both supported platforms.

Distributed lifecycle receipts retain `passed: false` and `release_ready: false`.
No fresh production browser consent, authenticated tool call, concurrent token
renewal or long-running expiry cycle is claimed for these alpha.17 binaries.
This is owner testing publication, not regular release promotion.

Both inference and remote MCP still traverse AI Gateway. The target remains
**mcp server 1**, with eight utility tools executed locally on that server host.
No SCM management API troubleshooting or target-server modification was performed.
Docusaurus edits remain with the owner's concurrent Claude Code voice review;
this release does not overwrite or deploy that checkout.
