# Connection health npm test publication — September 18, 2026

Version: `0.1.0-alpha.22.mcp.2`. Registry: `https://npm.cdot.io`. Channel: `mcp`.
The owner requested published npm packages for feature handoffs instead of
running development binaries. Stable promotion is separate: existing `latest`,
`alpha` and `onboarding` tags remain unchanged.

This version adds `/doctor` inside `airs`: active-environment health, bounded
native credential-service diagnostics, explicit minimal inference verification,
and recovery through the existing MCP manager. Opening the dashboard does not
send inference. Cancellation preserves the conversation and draft. Workspace
API key replacement remains `airs --environment NAME login`.

Runtime source: `19e5bcee52f6b21e22441a870db0a610fa8b3d4b`.
Owned Forgejo runs: Linux x64 225, Linux ARM64 224, Apple Silicon build 223.
Signing run 226 and exact native hashes are recorded in `ACCEPTANCE.json`.

The first upgrade-check invocation rejected the mcp.1 version before running
any upgrade: the validator only recognized numeric alpha and onboarding
versions. Tooling commit `45eff84ced` adds the immutable mcp version form.
Only the failed upgrade/final-check stage was resumed; runtime artifacts were
unchanged. This is a validation-tool correction, not a release binary rebuild.

Release acceptance uses installed candidate npm packages on Linux x64, native
Linux ARM64 in Jadzia's Docker VM, and Apple Silicon in its GUI session. It
covers native credential storage, onboarding, terminal restoration, the bundled
CLI 7.0.0, upgrades from mcp.1, command output, MCP management and the new doctor.
Linux-only native-service fixtures are explicitly skipped on macOS. Developer
ID signing/notarization is verified against the exact installed Mac executable.

`stage.py` binds publication to those accepted bytes. Only publication/build/
validation metadata may differ from candidate packages; executable, launcher,
CLI and other runtime payloads remain identical. `publish.py` uploads native
packages before the launcher and checks every immutable registry integrity.
Fresh anonymous registry installations are verified separately on all platforms.

This is an owner-authorized test release, not a claim of attended production
ServiceNow OAuth/tool acceptance. That acceptance and the owner-specific Linux
credential-store failure remain open. The CLI/TUI suite passed 5,284 tests with
six skips; eight home-directory tests passed for the version change. No new
full-workspace success is claimed; previous 150 failures are retained as limits.

```sh
npm install -g airs-harness@0.1.0-alpha.22.mcp.2 --registry=https://npm.cdot.io
airs --version
airs
```

Enter `/doctor` for connection health and `/mcp` for gateway MCP management.

Publication completed: all four registry integrities match staging. Fresh
anonymous installs passed on all three platforms with accepted native hashes,
doctor/MCP fixtures, environment lifecycle and command-output checks. All
pre-existing non-mcp dist-tags were preserved.
