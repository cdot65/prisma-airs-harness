# Ubuntu SSH keyring and Codex remediation

Host: `cdot@10.0.44.1` (`ubuntu1`), Ubuntu 26.04.1 LTS x86_64,
GNOME Keyring 50.0-1. Installed AIRS remains `0.1.0-alpha.22.mcp.3`.

The previous preparation script activated Secret Service over D-Bus and then
ran `gnome-keyring-daemon --unlock` without replacement. Four competing daemons
were left running; the original D-Bus owner could not see the newly created
login collection. The script now uses `--replace --unlock --daemonize`, checks
the default collection's actual D-Bus lock state, and reports failed unlocks
without attempting a headless graphical prompt. New keyrings require password
confirmation even when D-Bus returns a dangling login alias.

Native regression command, executed on ubuntu1 using private D-Bus and XDG
directories with disposable passwords and credentials:

```sh
python3 /tmp/test_prepare_airs_ubuntu.py /tmp/prepare_airs_ubuntu.sh
```

Passed: new encrypted keyring creation and credential write/read/delete;
already-unlocked service reuse; read-only locked check; wrong-password failure
without changing encrypted storage; correct-password recovery with a preexisting
fixture credential preserved. Bash syntax and `git diff --check` passed.
`just fmt` passed after adding the installed Cargo and DotSlash directories to
PATH; unrelated formatter changes were reverted. No Rust code changed.

Deployed to `~/prepare-airs-ubuntu.sh`; the old script is preserved as
`~/prepare-airs-ubuntu.sh.before-keyring-fix-20260919`. Stopped only the five
previously identified GNOME Keyring processes, then reactivated one Secret
Service. Encrypted keyring files were byte-for-byte unchanged. The owner's
keyring remains locked pending local password entry; synthetic acceptance is
not evidence of unlocking that keyring or production authentication.

Codex installed successfully as `codex-cli 0.155.1` under `~/.local/bin`.
The user npm global prefix is now `~/.local`; a fresh login shell resolves
`codex` and runs `codex --version`. AIRS retains its dedicated npm prefix.

The host passes installed-package, CLI, sandbox, registry TLS and gateway
connectivity checks. This is test-host readiness, not native build-runner
acceptance: no Cargo/Rust/GCC was on the SSH PATH and the filesystem had about
35 GiB available. Production AIRS and Codex authentication remain untested.
