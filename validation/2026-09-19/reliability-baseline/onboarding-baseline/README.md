# Published mcp.2 onboarding baseline

Executable reproduction completed 2026-09-19 against the installed npm mcp.2
launcher and exact native bytes recorded in `RESULT.json`.

Run using the project's already-installed fixture Python and a new private bus:

```sh
dbus-run-session -- /tmp/airs-dual-auth-20260918/venv/bin/python /tmp/airs-autonomous-20260919/evidence/onboarding-baseline/reproduce.py
```

The script imports the existing repository `IdentityFixture` and `Preview` PTY
driver. It creates only temporary harness state, a synthetic HTTPS gateway, and
a native Secret Service on a new private D-Bus session with isolated XDG state.
No owner credentials, owner environment, production gateway, or real SSO session
are involved. Temporary credential state is removed afterward.

Result: with saved default `work`, `airs --environment staging login` saves the
synthetic workspace key only in `staging`; the gateway rejects its probe with
HTTP 403. The onboarding message displays:

`Retry: airs doctor --verify-access (select the same environment).`

Escaping that outcome restores the terminal and displays:

`Credential saved. Start AIRS when ready, or inspect access with airs doctor --verify-access.`

Both omit `--environment staging`. The saved default and original registry are
unchanged, so copying the recovery command targets `work`. Screen captures are
plain rendered text and contain no key. The preserved environment binding and
history checks pass; this is recovery guidance confusion, not credential leakage.

Node18 check: `command -v node` resolves `/usr/local/bin/node`, version v22.23.2.
No `/usr/bin/node`, `/opt/node/bin/node`, nvm Node versions, mise Node install,
or fnm Node versions were present. No alternate toolchain was installed.
Actual Node18 execution was therefore not performed; unsupported-runtime handling
remains a source finding (manifest engines warning but no launcher runtime guard).
