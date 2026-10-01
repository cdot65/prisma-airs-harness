# AIRS 0.1.6 private test release

Published to **https://npm.cdot.io** for Linux x64, native-validated Linux ARM64
and Developer ID signed/notarized Apple Silicon. The three native packages and
`@cdot65/prisma-airs-harness` launcher are exactly **0.1.6**, under
`stable-candidate`. `latest` remains **0.1.5** on all four packages; public npm is
unchanged. CLI **7.2.0** and SDK **0.34.0** remain bundled.

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.6 --registry=https://npm.cdot.io
airs --version
```

The command should report `airs 0.1.6`. Instant steering is opt-in:

```sh
airs --enable instant_interrupt
```

The default waits for the current response. To roll back, install the same
launcher at `0.1.5` from the same registry. Keep the AIRS home and native
credentials; no forced install or reset is needed.

For Ubuntu preparation, make a task-owned copy of the packaged
`scripts/prepare_airs_ubuntu.sh` example and configure both `gateway_url` and
`registry=https://npm.cdot.io` before running it. Its public-registry default
cannot discover this private-only version. The configured example passed the
isolated encrypted-keyring, installed-version, sandbox and connectivity checks.
The initial public-default failure and corrected private configuration are
retained in the evidence; the published package bytes were not replaced.

## Feature readiness

| Feature | Score | Observed evidence |
| --- | --- | --- |
| Sandbox boundaries and macOS TLS trust | 9/10 | Applied Linux sandbox policy; native Mac aliases/trust-service probes; explicit-denial and credential/TLS acceptance. |
| Command completion and launch errors | 9/10 | Complete bounded head/tail transcript, cancellation-safe launch lifecycle, UTF-8/delta regressions and native early/final output exactly once. |
| Markdown copy and blank-task drafts | 9/10 | Markdown/table snapshots, protected clipboard/composer cases, actual remote app-server blank-session restoration and native PTY copy/draft checks. |
| Opt-in instant steering | 9/10 | Default/enabled native behavior, observable no-replay append, retry/compaction/queued-input checks, durable cell and WebSocket continuation regressions. |

Scores assess implementation and automated private-test readiness: behavior,
safety, regressions and native delivery contribute two points each; documented
controls contribute one. The final point is reserved for owner attended
acceptance. Gateway and TypeSafe semantic verdicts use controlled fixtures;
these checks do not claim owner production SSO or paid Jev service acceptance.
Deterministic code continues to own authorization and execution.

## Validation and provenance

- Exact candidate **and fresh anonymous registry installations** passed on all
  three native targets, including native stores, managed CLI, Gateway/TypeSafe
  agent workflows, command reporting and 0.1.5 upgrade/rollback.
- Every installation ran 65 behavioral tests. All four new PTY priority tests
  ran and passed; platform-specific skips were three on Mac and one per Linux
  target.
- Scoped source suites passed 5,307 Linux and 5,315 Apple Silicon tests.
- Full GNU: **19,097 passed, three failed, 35 skipped**, one successful retry.
  The three remote-shell assertion failures exactly match 0.1.5, including their
  normalized assertion output. Their tests and CLI gate are unchanged; AIRS
  rejects the external exec-server command before opening a listener. They are
  retained as failures, not reported as passing tests.
- Runtime source: `f3d55fc1a1316fb8e191e87a5267d616db7bdef9`.
  Acceptance tooling: `30f9686cefdac98b7faa4840a2eedd8209b90bfd`.
  Packaging: `f0135d41909dabf440606070e68b1bca23b08219`.
  The full GNU tooling-source revision has the identical Rust runtime tree.
- macOS notarization submission: `dd51dfe2-770f-49b6-b2a6-228b127f2720`, accepted.
  Installed signature/notarization checks passed on the exact registry bytes.

[RELEASE-ACCEPTANCE.json](RELEASE-ACCEPTANCE.json) binds scores, native hashes,
receipt chains, publication and the complete 506-file evidence bundle.
The bundle retains initial failures and corrected acceptance evidence.

Run the offline integrity audit from the repository root:

```sh
python3 validation/2026-10-01/private-0.1.6/audit.py
```

This release is for owner testing. Stable/public promotion is a separate action.
