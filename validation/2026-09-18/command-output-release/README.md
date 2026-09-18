# AIRS command-output release — 0.1.0-alpha.22.onboarding.3

Published at **https://npm.cdot.io** under **latest**, **alpha** and **onboarding**.
The launcher and Linux x64, Linux ARM64 and Apple Silicon native packages are
available for remote testing. Fresh anonymous installs passed on all three
platforms; an unpinned install selected this version without `--include=optional`.

```sh
npm install -g airs-harness@latest --registry=https://npm.cdot.io
airs --version
airs cli --version
```

The first reports `airs 0.1.0-alpha.22.onboarding.3`; the bundled product CLI
reports `7.0.0`. The standalone CLI remains `7.0.1` and was not republished.

## Change and provenance

Generated resume commands, help examples and recovery hints now use `airs`.
Selected environments and shell escaping are preserved. A real installed TUI
session exits with `airs --environment work resume <session-id>`. The Prisma
AIRS logo, package names, internal identities, credential bindings and histories
are unchanged.

Runtime source: `a57f67bd72e9e3cf77d8feccdb2e6c3738012ec1`.
Packaging source: `fd21710f3a`; subsequent changes are acceptance tooling and
receipts. [NPM-PACKAGES.json](NPM-PACKAGES.json) records package hashes and exact
native source. [PUBLICATION.json](PUBLICATION.json) records verified registry
integrities and tag promotion. The publication step changes validation/packaging
metadata only; executable, launcher and bundled CLI payloads match the accepted
candidate bytes.

## Native installed acceptance

| Check | Linux x64 | Native Linux ARM64 | Apple Silicon |
| --- | ---: | ---: | ---: |
| Local HTTPS OAuth / native credential store | 18 | 18 | 12 |
| Shell and terminal restoration | 8 | 8 | 7 |
| Executable regressions passed | 46 | 46 | 46 |
| Platform-inapplicable skips | 1 | 1 | 1 |
| Command-output checks, including actual TUI exit | 15 | 15 | 15 |
| Managed CLI check groups | 5 | 5 | 5 |
| Upgrades from onboarding.2 preserving configuration | 3 | 3 | 3 |

[ACCEPTANCE.json](ACCEPTANCE.json) binds these results to exact installed native
hashes. The three `*-installed.tar.gz` archives contain raw logs, onboarding
receipts and terminal galleries. Apple Developer ID team G5QLZ5A8TA and online
notarization were verified again on installed bytes. Linux ARM64 acceptance ran
natively in Jadzia's ARM64 Linux VM before publication.

Fresh registry installations separately passed the executable suite, environment
create/rename/switch/remove/recreate checks, and the 15 command-output checks.
The Mac registry installation also passed native Keychain checks. Captured normal
AIRS gateway requests omitted `access_programs` on every platform. The owner
identified the reported unsupported `access_programs.cyber` error as occurring
in this Codex conversation; no AIRS runtime change was made for that separate
client/backend request error.

## Full workspace and authorization

**18,097 passed, 150 failed, 34 skipped.** All eight changed packages passed,
including 933 CLI, 4,336 TUI and 4,159 core tests. Twenty-one launcher tests and
eight home-directory tests also passed. Formatting, affected-package Clippy and
whitespace checks passed.

The workspace remains **not green**. The failures are in unchanged exec-server
(146), app-server (2), linux-sandbox (1) and skills-extension (1) packages. A
previous-source baseline comparison was not performed. The owner explicitly
requested publication after receiving this result; [AUTHORIZATION.json](AUTHORIZATION.json)
records that scope. See [the complete report](../command-output/README.md).

Native acceptance uses isolated local HTTPS fixtures and native credential
stores. Attended production SSO and ServiceNow acceptance remains with the owner.
Distributed validation metadata retains these limitations and does not claim
full production gateway lifecycle acceptance. This release does not deploy the
Docusaurus documentation PR.

## Cleanup

The owned ARM64 test container was removed and the two owned test LaunchAgents
were unloaded. Jadzia's Docker engine was restored to stopped while preserving
its Desktop UI. The metadata-only Keychain audit found no recent service entries.
The temporary Forgejo session was signed out and temporary credential files
were removed. Production identity and trust configuration were not changed.
