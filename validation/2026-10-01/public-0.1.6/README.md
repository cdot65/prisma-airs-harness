# AIRS 0.1.6 public npm release

Published to **https://registry.npmjs.org** through the owned Forgejo workflows.
The launcher and Linux x64, Linux ARM64 and Apple Silicon packages are **0.1.6**
under both `latest` and `stable-candidate`. CLI **7.2.0** and SDK **0.34.0** remain
bundled. Apple Silicon is Developer ID signed and Apple notarized.

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.6 --registry=https://registry.npmjs.org
airs --version
```

The command reports `airs 0.1.6`. Unversioned public installs also select 0.1.6.
Instant steering remains opt-in with `airs --enable instant_interrupt`.
For rollback, install the same launcher at `0.1.5` from the same registry;
preserve the AIRS home, environment bindings and native credentials.

For Ubuntu preparation, copy the packaged `scripts/prepare_airs_ubuntu.sh`
example and configure its `gateway_url` for the deployed gateway. Its public npm
registry default now resolves this release. Six isolated encrypted-keyring,
installed-version, sandbox and connectivity checks passed with that default.

## Feature readiness

| Feature | Score | Verified behavior |
| --- | --- | --- |
| Sandbox boundaries and macOS TLS trust | 9/10 | Applied sandbox policy, Mac aliases/trust, explicit-denial retention and installed credentials/TLS. |
| Command completion and launch errors | 9/10 | Complete bounded transcript, cancellation-safe launch events and early/final output exactly once. |
| Markdown copy and blank-task drafts | 9/10 | Markdown/table snapshots, clipboard/composer protection, remote blank-task restoration and native PTY checks. |
| Opt-in instant steering | 9/10 | Default/enabled behavior, append without replay and retry, compaction, queued-input and durable continuation checks. |

Scores cover implementation and automated acceptance; the final point remains
owner attended acceptance. Gateway and TypeSafe agent checks use controlled
fixtures. Owner production SSO and paid Jev service acceptance are not claimed;
deterministic code owns authorization and execution.

## Verification and publication

Exact public candidates and fresh anonymous public-registry installations passed
on all three native targets, including native stores, managed CLI, Gateway/MCP
and TypeSafe agent workflows, 65 behavioral tests per installation, all four new
PTY regressions, and 0.1.5 upgrade/rollback. After promotion, default unversioned
public installs passed on all three targets. The published executables are
identical to the [accepted private binaries](../private-0.1.6/README.md).

Scoped source suites passed 5,307 Linux and 5,315 Apple Silicon tests. Full GNU
retains **19,097 passed, three failed and 35 skipped**. The three remote-shell
assertion failures exactly match 0.1.5; their tests and CLI gate are unchanged.
AIRS disables the external exec-server command. These remain recorded failures.

[Publication run 532](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/532)
verified all four immutable uploads and completed publication.
[Promotion run 533](https://git.cdot.io/cdot/prisma-airs-harness/actions/runs/533)
promoted public `latest` after fresh native registry acceptance. The launcher was
published/promoted after the native packages. Twenty-two publication, promotion
and workflow contract tests passed.

Earlier workflow attempts are retained: run 529 rejected the Mac temporary-path
symlink before mutation; runs 530/531 stopped during slow npm metadata
propagation after acknowledged uploads. Recovery verified existing package
integrity before skipping uploads; no immutable upload was blindly repeated.
Conjur supplied a temporary private npm credential, removed after workflow use.
Final anonymous reads also show npm.cdot.io advertising `latest=0.1.6`; this
publication/promotion ran against public npm only.

Runtime source is `f3d55fc1a1316fb8e191e87a5267d616db7bdef9`; frozen native
acceptance tooling is `5d1123bf33d00c234b58f499f098c1d38886d7ca`; publication
transport/workflow tooling is `9cf63cac55fab5192af7eaaaa9c6b6a3654f0dd6`.
Mac notarization submission `dd51dfe2-770f-49b6-b2a6-228b127f2720` was accepted.
The retained signing check is a prepublication snapshot; publication is recorded
separately in the workflow receipts.

[RELEASE-ACCEPTANCE.json](RELEASE-ACCEPTANCE.json) binds source, executable hashes,
scores and the 489-file public evidence bundle. The offline audit also verifies
the original 506-file private bundle. Run from the repository root:

```sh
python3 validation/2026-10-01/public-0.1.6/audit.py
```
