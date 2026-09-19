# Reliability and onboarding: mcp.3 test release

Release version: `0.1.0-alpha.22.mcp.3`, registry `https://npm.cdot.io`, tag `mcp`.
All four packages are published and fresh anonymous registry acceptance passed
on all three native targets. The package and deployed guide are ready for owner testing. Final independent
review and the evidence index accompany this record.

## User-visible changes

- Authentication recovery preserves the initiating credential failure when cleanup
  also fails, with actionable native-store guidance and no plaintext fallback.
- Recovery commands retain the selected environment, including names beginning
  with a hyphen, and malformed saved names are rejected without terminal injection.
- The npm launcher checks supported Node versions before native, bundled CLI or
  completion dispatch. Installing npm alone does not upgrade an old Node runtime.
- Current packaged and public guides describe environment lifecycle, SSO and
  workspace-key inference, in-session `/mcp` and `/doctor`, and their identity boundaries.
- Committed release tools replace scratch-only publication scripts with explicit
  source/hash bindings, safe resumption, native-first publication and fresh installs.

## Immutable identities

| Role | Commit |
| --- | --- |
| Native runtime | `7eba6bf33c13b669a6c28b53ec658b3c9c3be5c2` |
| Candidate validation | `a4f85a3de4a3337407fecd1e2eeeb3182573e8cf` |
| Launcher/packaging and corrected Mac prerequisite workflow | `94b45c17bd1182e789772f3f88685c5486cda7d8` |
| Repaired registry verification | `0f625cd047a19e559f0500dd0d9f0b131c791e78` |

`release/SPEC.json` remains unchanged after publication. The registry verifier's
separate revision is explicit in its tooling manifest and every new stage receipt.
The bundled product CLI remains 7.0.0 / SDK 0.33.0; standalone CLI 7.0.1 is unchanged.

| Native target | Executable SHA-256 |
| --- | --- |
| Linux x64 | `774c47315cacb3836ad6461d061974f3ed32e7a7762dba293b1dd9e3de154222` |
| Native Linux ARM64 | `23a6642fb140775e27014fd696150a38389525a780e92c3312939b9bd9c9c87a` |
| Signed Apple Silicon | `b0376aee39dbbcec138008647f1b8298aff906829c580180df24808b40704b8d` |

## Evidence and failures retained

Candidate acceptance passed on all three native targets; `release/candidate-acceptance`
contains portable stage receipts, structured results and logs. Staging rechecked
all runtime file bytes and modes, allowing only defined publication metadata changes.
Original candidate archives remain in private build storage; registry archives are
bound by the staged manifest and published immutable integrity.

Linux builds used owned workflow runs 228 (ARM64) and 229 (x64). Mac run 227 stopped at
its existing disk-space guard. Run 230 compiled the intended runtime, then failed
acceptance because the runner lacked `pyte`. Run 231 reused that exact intake,
provisioned an isolated Python environment and passed signing, notarization,
Keychain, raw native-store and installed fixture checks. The earlier failures are
preserved and are not counted as passing runs.

The first anonymous registry attempt failed on all targets because npm changes a
bundled dependency's declared command from 0644 to 0755. Its bytes were unchanged.
The repaired verifier first checks every byte, then derives the permitted command
permission normalization only from those verified package manifests. A real npm
regression fixture and adversarial byte/path/mode tests passed. Failed attempt
logs remain under `release/registry-acceptance`; final successful runs use a fresh,
separately named `release/registry-acceptance-revised` evidence root. All three
passed with the explicit repaired tooling commit. No package was rewritten or republished for this
validator repair.

## Acceptance boundaries

This is an owner-authorized test release, not stable promotion. Existing non-mcp
tags, including latest/alpha/onboarding, remain unchanged. Native metadata retains
`release_ready:false` for the broader production acceptance contract.

Attended real-account SSO, workspace-key and ServiceNow acceptance and the owner's
Ubuntu credential-store investigation were explicitly deferred until tomorrow.
The synthetic gateway/native-store fixtures do not establish those live scenarios.
There is no Windows or Intel Mac distribution claim. The historical full Rust
workspace report remains 18,097 passed/150 failed/34 skipped; it is not a green
workspace result. This work's affected-package results are recorded separately in
sibling authentication-recovery and onboarding-reliability evidence directories.

Current native MCP profiles still require the documented keyring setting. Existing
MCP native records may be shared by connection name/URL within an OS user. Neither
an inference workspace key nor a local environment name authorizes an upstream
MCP service on its own.

## Additional upgrade coverage

The main candidate/registry matrices use mcp.2 as the immutable upgrade baseline.
A separate normal-Python supplement also upgraded onboarding.4 on all three native
targets. Each passed npm-managed installation, legacy migration and front-of-PATH
command cases, preserving all four checked configuration files. See
`release/STABLE-UPGRADE.json` and `release/stable-upgrade-portable`. No live
credentials or owner profiles were used.

## Actual unsupported-Node supplement

Official SHA-verified Node 18.20.8 on native ARM64 Debian exercised the published
launcher in 16 plain/instrumented cases. Every case rejected the unsupported
runtime with actionable guidance before child dispatch or state creation. No npm
installation under Node 18 was performed. The original Ubuntu x64 Node 18.19.1 /
npm 9.2.0 session is not claimed as reproduced. The official x64 archive could not
execute on the local musl host; its failed attempt is retained alongside the
successful ARM64 result. See `evidence/focus5-review/node18`.

## Public documentation and cleanup

[Getting started](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/)
is deployed from docs commit `9d328da14b0abf3e8b3823cebb4f92a4bd4a8641`.
[Pages run 35420045886](https://github.com/cdot65/prisma-airs-reference-architecture/actions/runs/35420045886)
passed build/deployment after 22 local browser checks. Live verification checked
five HTTP 200 routes, new release/identity boundaries and the ServiceNow anchor,
with zero browser errors or failed requests. Original response hashes and the
visible guide screenshot are under `evidence/documentation/pages-publication`.

Owned temporary npm credentials and Forgejo cookies were removed. The task's
three completed GUI launch labels were unloaded and its disposable ARM64 container
was removed after all portable evidence was collected. Owner credentials, the
owner VM and its services were not changed. Native fixture credential cleanup
receipts passed separately. Build caches and the verified off-host recovery
backups remain available. No plaintext account credentials belong in this evidence.

## Final independent review

All five planned focuses passed independent agent review: baseline 9.5,
authentication 9.5, onboarding 9.3, repeatable delivery/documentation 9.6, and
publication/handoff 9.5 out of 10. `FINAL-FOCUS5-REVIEW` records the final gate.
The review bound the then-complete 391-entry evidence inventory and 392-file
committed tree at `ec7c4698b5223e7c21e7b94057983fbdb4d1bc7d`. This subsequent
record adds that review and final cleanup proof; it does not change runtime,
package, acceptance or deployment evidence. Owner review remains pending.
