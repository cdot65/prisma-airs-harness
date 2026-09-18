# Workspace API key and SSO remediation

The gateway workspace now accepts its native workspace API keys alongside authorized SSO. Local environment names are independent of gateway workspace names. The owner key was preserved; no new owner workspace is required.

Runtime source: `d58d55b7aafc9a506b463041b5101116808b76ed`.
Version: `0.1.0-alpha.22.onboarding.4`.
Packaging source: `be77a6b317bf23dc76b88f8a327eb80c0fb29fb6`.

The harness distinguishes saved credentials from verified access, includes HTTP status and a searchable gateway trace ID, and recognizes a blocking guardrail verdict even with HTTP 200.

## Validation

- Affected CLI, TUI and home-directory Rust suite: 5,281 passed, 6 skipped.
- Gateway policy: seven test groups passed; live valid workspace key and browser PKCE SSO inference passed.
- Invalid/missing keys, missing inference permission, missing SSO invoke role and tampered JWTs were denied. Both authentication paths retained AIRS prompt blocking.
- Revocation propagated asynchronously; the revoked test key returned 401 after several minutes. Test keys and disposable SSO users were removed.
- Workspace configuration, default model route and AIRS security checks/actions matched their original snapshots. Inference keys did not grant MCP access.
- Docusaurus content checks, build and all 20 browser tests passed.

The existing published onboarding.3 doctor receipt demonstrates that the live gateway fix works without recreating an environment. The new version's native installed acceptance and registry receipts are recorded separately below. These checks do not claim an attended owner SSO session or ServiceNow tool call, nor a new full Rust workspace run.

## Deployment source

[Infrastructure PR 423](https://git.cdot.io/cdot/talos-cluster/pulls/423) contains the private post-authentication policy, deployment, exact prior guardrail rollback and sanitized live evidence. Native gateway authentication remains responsible for API-key validity, workspace membership and scopes. Preserve the policy service hostname in private Helm values during future gateway upgrades.

## Publication

Published at `https://npm.cdot.io` under **latest**, **alpha** and **onboarding** for the launcher, Linux x64, Linux ARM64 and Apple Silicon packages. Fresh anonymous installations passed on every platform. An unpinned install also returned onboarding.4 without `--include=optional`.

```sh
npm install -g airs-harness@latest --registry=https://npm.cdot.io
airs --environment workspace-api doctor --verify-access
airs --environment workspace-api
```

Keep the existing environment and key. The gateway workspace does not need to be renamed or recreated.

| Installed check | Linux x64 | Linux ARM64 | Apple Silicon |
| --- | ---: | ---: | ---: |
| HTTPS OAuth and native credential store | 18 | 18 | 12 |
| Shell and terminal restoration | 8 | 8 | 7 |
| Executable regressions passed / skipped | 46 / 1 | 46 / 1 | 46 / 1 |
| Public command guidance | 15 | 15 | 15 |
| Managed CLI groups | 5 | 5 | 5 |
| Upgrade cases preserving environment state | 3 | 3 | 3 |

[ACCEPTANCE.json](ACCEPTANCE.json) binds acceptance to installed executable hashes. The installed archives retain logs and terminal galleries. Mac Developer ID signing and notarization were verified on the installed bytes. Publication changed only allowed package metadata; runtime payloads match the accepted candidates.

The [getting-started guide](https://cdot65.github.io/prisma-airs-reference-architecture/learn/login/) is deployed from `5374c25c8888663a2ab57ea27a7662ec8e490fd3`; the live page was checked for onboarding.4, workspace key setup, environment lifecycle and ServiceNow guidance.

Temporary test LaunchAgents and the ARM container were removed, Docker was restored to stopped, and the metadata-only Keychain audit found no recent test entries. The temporary Forgejo session was signed out and credential files removed. See [CLEANUP.json](CLEANUP.json).
