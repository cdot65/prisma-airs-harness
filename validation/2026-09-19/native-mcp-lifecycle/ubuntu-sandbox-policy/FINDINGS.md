# Ubuntu 26.04 sandbox prerequisite investigation

The installed AIRS mcp.3 sandbox works on `cdot@10.0.44.1` with Ubuntu's user-namespace restriction enabled. No custom AIRS AppArmor profile or security-policy change is currently needed.

## Observed evidence

- Ubuntu 26.04.1 LTS, AppArmor `5.0.2-0ubuntu1~26.04.1`, bubblewrap `0.11.1-1ubuntu0.3`.
- `kernel.apparmor_restrict_unprivileged_userns = 1` throughout the observations.
- The owner's preparation had installed `/usr/bin/bwrap` by the first SSH read. This investigator did not install packages or change security settings.
- The AppArmor package owns `/etc/apparmor.d/bwrap-userns-restrict`. Its attachment is exactly `/usr/bin/bwrap`; child processes stack the `unpriv_bwrap` profile that denies capabilities.
- A child of the system bwrap reported `bwrap//&unpriv_bwrap (enforce)` from `/proc/self/attr/current`. This confirms active enforcement, not merely a policy file on disk.
- The actual installed `airs 0.1.0-alpha.22.mcp.3` read-only sandbox read a synthetic marker, rejected an append with `Read-only file system`, and left the marker unchanged. It used a unique temporary environment with an unset credential-environment reference. No authentication, owner credential access, or inference request occurred. Scratch state was removed.
- The installed npm prefix contained no file named `bwrap`; do not describe this particular published AIRS package as shipping a bundled bwrap.

Receipts: `HOST-PROBE.json`, `PROFILE-PROBE.log`, `PACKAGE-LAYOUT.json`.

## Source behavior and interpretation

`codex-rs/linux-sandbox/src/launcher.rs:126` prefers a supported system bwrap before trying a bundled resource. The capability probe requires `--as-pid-1` and `--perms`; Ubuntu's installed binary has both, plus `--argv0` and `--ro-bind-fd`. `codex-rs/sandboxing/src/bwrap.rs:168` canonicalizes the PATH candidate and excludes executables inside the current working directory.

The upstream-compatible fallback in `codex-rs/linux-sandbox/src/bundled_bwrap.rs` resolves `codex-resources/bwrap` or legacy adjacent locations, verifies an embedded digest when present, then executes the opened file through `/proc/self/fd`. Ubuntu's exact `/usr/bin/bwrap` attachment does not automatically grant an arbitrary npm resource path the same policy. That is a policy/source inference, not a tested failure of a bundled binary: this package has no bundled binary to test.

Ubuntu's official documentation says AppArmor uses path-based policies and requires explicit allowance for applications needing unprivileged user namespaces:
https://documentation.ubuntu.com/security/security-features/privilege-restriction/apparmor/

Ubuntu's security team specifically recommends a purpose-built bwrap profile that can be reused by applications relying on bwrap; it also explains why broad unconfined profiles allow more than intended:
https://discourse.ubuntu.com/t/understanding-apparmor-user-namespace-restriction/58007

## Concrete recommendation

Keep the preparation script's Ubuntu `bubblewrap` package prerequisite and actual read-only AIRS canary. On this host, that path is proven to work with stock enforced policy. Do not install an additional AIRS policy or disable any sysctl/AppArmor protection.

If a future host fails the canary, first confirm `/usr/bin/bwrap` is the PATH candidate, its capability flags, the distribution-owned profile, and the concrete denial. Repair the distribution package/profile only after an administrator reviews that evidence. If a future release intentionally requires an npm-bundled helper, treat exact-path policy attachment and child capability stripping as a separate design and acceptance task; avoid a broad user-writable-path allow rule. No such remediation is necessary or applied here.

Scope: sandbox readiness only. Native credential-store readiness and attended SSO/workspace-key/MCP production acceptance are separate.
