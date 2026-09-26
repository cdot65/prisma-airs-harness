# MCP descriptor isolation review

Adapt upstream 47094 to AIRS's existing launchers. Linux and macOS Tokio fallback invoke child-local cleanup; macOS native relative-path spawning applies POSIX_SPAWN_CLOEXEC_DEFAULT with explicit stdio dup2 actions. Windows remains unchanged. Scope does not touch OAuth, gateway routing, remote credentials or tool grants.

Review discovered the macOS native path bypasses pre_exec. Corrected this separately rather than importing the entire upstream Command abstraction. Updated the native unit assertion from the old intentional inheritance contract to closure, retaining an ordinary Command positive control. New integration tests independently qualify inheritable pipe/socket/file sentinels, then check servers AND descendants, stdio round trip, arguments/env/cwd, and unchanged parent flags. Interpreter resolution avoids symlinking Apple's location-dependent python launcher. Shebang scripts run on Linux and Mac; only Mac promises the existing no-shebang shell fallback, which is separately covered. Linux ENOEXEC was a fixture mismatch, not a new product behavior.

Linux: all 352 tests pass (eight preexisting ignored tests). Native Mac: all 357 supported tests pass, eight ignored. Three remote-executor tests explicitly excluded on Mac because the published AIRS binary disables that upstream service; do not weaken the product gate to make those tests pass. They pass on Linux with the existing ordinary codex fixture. Final workspace verification must use newly built binaries and report inherited failures distinctly. These are not live-account OAuth results.

Scoped Linux Clippy completed with one unchanged baseline expect_used warning in rmcp-client/src/utils.rs (static product version header), no new warnings. Native lint pending. Existing cleanup is best effort if the host rejects all mechanisms; descriptor filtering complements, not replaces, the sandbox. New public exposure is limited to the existing helper through the existing pty module.

Scores pending native lint and final source/evidence comparison.

Final gate: native lint completed with the same unchanged static-header warning and no new warnings. All six production/test source hashes match the native checkout. Implementation 9, code quality 9, design 9, bounded feature completeness 9. Signed release and final workspace verification remain pending.
