# Explicit fullscreen preference

Adopted the applicable configuration portion of a2de8fedcc. The persisted
`tui.fullscreen_transcript` boolean defaults to false and flows through resolved
Config into client LocalSettings. Generated the JSON schema with the actual
`just write-config-schema` command. Existing Tui/Config constructors were updated.
The deprecated features.transcript_v2 key is ignored with a notice pointing to
the new preference, matching upstream. It cannot migrate into or override the
preference. The generic CLI feature-enable fixture now uses the supported
shell_tool flag rather than the deprecated transcript flag.

361 focused tests passed: the complete selected config/features packages,
LocalSettings default/explicit loading, every combination of missing/false/true
preference and deprecated false/true flag, AIRS recovery and existing terminal
alternate-screen checks (including Terminal.app over SSH). No retries or executed
test skips. The removed feature test asserted the superseded feature-toggle
contract; its replacement exercises the real ConfigBuilder/preference boundary.

This is configuration plumbing only. Launch-time ownership restrictions, reload
and session-transition mode preservation require the following renderer/input
integration. No daemon-startup module exists in this fork, and none was imported
or activated. The preference is not advertised as a working standalone feature.

Adversarial review: only one explicit default-false boolean enters the schema;
no auth/provider defaults, environment paths, dependencies or server protocol
fields change. Local configuration ownership is retained. Core and CLI source
changes require the final workspace/native gates; the focused suite alone does
not establish those gates or a completed feature score. No release is claimed.

Scoped config/core/TUI/features lint passed in 240.04 seconds, zero warnings or
fixes. Formatting passed in 17.55 seconds. The actual schema command passed in
18.79 seconds. Focused tests took 225.09 seconds including the config dependency
rebuild (1.144 seconds execution). No tests were repeated solely after lint/fmt.
