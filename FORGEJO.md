# Repository authority and delivery

The canonical repository is https://git.cdot.io/cdot/prisma-airs-harness.
Open new issues and pull requests there. GitHub is a public source mirror;
historical GitHub pull requests remain available, and their open heads were
preserved as `archive/github-pr-N` branches in Forgejo.

Forgejo owns CI and release authorization. The owned `airs-harness-docs.yml`
GitHub workflow publishes the harness documentation; all inherited upstream and
historical package workflows remain disabled. SDK/CLI workflows are archived
with `.disabled` suffixes.
Harness documentation is at https://cdot65.github.io/prisma-airs-harness/.
Documentation is deployed only from an
`airs-docs-<full commit SHA>` tag approved in Forgejo and copied by the push mirror.
The GitHub Pages job verifies that the tag matches the exact source commit.

Package destinations remain npmjs.org for the SDK/CLI and npm.cdot.io for Harness.
Migration acceptance uses a separate prerelease channel and preserves `latest`.
The npm publishing credential is held at Conjur
`data/airs-release-bot/npmjs/token`; never put its value in source or logs.
Direct granular-token publishing is a temporary bridge: npm removes it in
January 2027. Before expiry, adopt staged releases with maintainer approval or a
supported trusted-publishing runner. See https://docs.npmjs.com/about-access-tokens/.

Clone the canonical source:

```sh
git clone git@git-ssh.cdot.io:cdot/prisma-airs-harness.git
```

Do not force-update a mirror until all destination-only commits and refs are
preserved. Stop mirroring before any emergency GitHub-authority rollback, then
reconcile both histories before resuming. Existing package versions are immutable.

## Existing operational reference

# Source, CI and mirror operations

The canonical Rust repository is https://git.cdot.io/cdot/prisma-airs-harness.
The public GitHub repository `cdot65/prisma-airs-harness` is a one-way push
mirror. Push branches to Forgejo. The local `origin` points to Forgejo and
`github` retains the mirror URL. The SDK and CLI are separate canonical repositories in the same `cdot` namespace.

Forgejo mirrors all branches and tags on push and hourly using a dedicated
GitHub deploy key. Compare branch/tag hashes before repairing a failed mirror;
never push divergent development directly to GitHub. GitHub Actions enables only
the owned documentation workflow. Other `.github/workflows` files are historical
migration inputs; `.forgejo/workflows` owns application CI. Inherited upstream automation must
remain inactive. Git mirrors do not synchronize future issue/PR discussions or
release assets; use Forgejo for new reviews and release records.

Verdaccio remains `https://npm.cdot.io`, with unscoped `airs-harness` packages.
The repository migration does not publish a release or change executable names.

## Linux

`airs-harness-check.yml` runs package contracts on Forgejo's node runner.
`airs-harness-linux.yml` validates the integration branch on GNU Ubuntu 24.04,
including the complete Rust suite. It also supports manual source selection.
The job uses two CPUs, an 8 GiB memory limit, a regular test user and job-local
seccomp/AppArmor relaxation needed to exercise nested bubblewrap namespaces.
It does not enable privileged containers or change Talos host sysctls. Failure
of the namespace probe blocks the test run instead of counting sandbox skips
as acceptance. Validation logs are stored as Forgejo artifacts.

## Apple Silicon: Jadzia

Host: `10.0.1.121` (`jadzia.local` on the owner's LAN). The runner makes outbound
HTTPS connections to `https://git.cdot.io` to poll and execute jobs. No inbound
connection from Forgejo to the Mac is needed. SSH is only for setup/maintenance.

The account is `cdot`. Repository-scoped runner **74**, `jadzia-airs-arm64`,
is registered with label `airs-macos-arm64:host` and capacity one. Its state is
`/Users/cdot/.local/share/airs-forgejo-runner`; registration/configuration files
are private. The user LaunchAgent is
`~/Library/LaunchAgents/io.cdot.airs-forgejo-runner.plist`. It starts in the
user's GUI login session, so a reboot requires that session before jobs resume.

Check service status over SSH:

```sh
ssh cdot@10.0.1.121 'launchctl list io.cdot.airs-forgejo-runner'
```

The runner uses Homebrew Node 22 and Python 3.13 through its own PATH. Xcode
command-line tools are installed. Forgejo preflight run 20 and frozen native
acceptance run 30 passed. Acceptance verifies the frozen executable hashes,
Apple Silicon architecture, existing ad hoc signatures, native credential
storage, Keychain lifecycle, executable contracts and staged Verdaccio install.
It does not establish Developer ID signing or notarization.

Fresh compilation requires at least 100 GiB free before starting. Owner-authorized
cleanup recovered enough space; approximately 243–250 GiB remained during the
build. Forgejo run 52 (internal ID 3232) compiled the alpha.12 source and passed
native/package acceptance on Jadzia.

The Developer ID Application identity for team `G5QLZ5A8TA` is installed in the
login Keychain. The notarization profile is `prisma-airs-harness-notary` in that
same Keychain. Credentials remain on the Mac. Forgejo run 50 validated signing,
Apple notarization, online ticket verification and native Keychain acceptance.
The signed-package workflow extends this to the freshly compiled artifact,
installed npm signature verification and Mac/Linux installed acceptance.

## Workflow migration inventory

| Forgejo workflow | Purpose and validation |
| --- | --- |
| `airs-harness-check.yml` | npm/packaging contracts; passes on main and candidate |
| `airs-harness-linux.yml` | GNU build and complete Rust suite; validation in progress |
| `airs-harness-linux-arm64.yml` | Zig cross-compiled `aarch64-unknown-linux-musl` candidate, QEMU version probe, native and npm candidate packaging; no installed acceptance |
| `airs-harness-macos-preflight.yml` | Native runner/tool/artifact wiring; passed |
| `airs-harness-macos-acceptance.yml` | Frozen native, Keychain and staged Verdaccio acceptance; passed |
| `airs-harness-macos-build.yml` | Fresh native compilation and artifact acceptance; passed on run 52 |
| `airs-harness-macos-signing.yml` | Frozen artifact signing, notarization and native acceptance; passed on run 50 |
| `airs-harness-signed-package.yml` | Fresh build artifact signing/notarization, npm packaging and installed Mac/Linux E2E; passed on run 79 |
| `airs-harness-windows-identity.yml` | Native Credential Manager checks; needs runner label `airs-windows-x64` |

The old npm, deployment and release-check contracts are covered by the package
and GNU jobs. Mac revalidation, storage and Keychain checks are consolidated in
native acceptance. The old Mac release build is ported separately. Windows
identity checks are prepared, but no Windows runner is registered and Windows
distribution remains outside the authorized targets.

Release publication, review-registry and optional GitHub
Packages publication automation have not been validated or activated on Forgejo.
Their old GitHub workflows remain disabled migration inputs. Routine source
pushes run validation and mirror Git refs; they do not publish packages or sign
releases. Migration is not complete until the required remaining workflows and
runner-dependent checks pass.

## Built-in MCP release workflow

The MCP remediation release starts from the runtime actually distributed in
alpha.12 (Codex 0.154, commit `19ce13981`), rather than regressing to the older
0.153 tree that remained on main. The integration preserves the prior main
history, distribution receipts, owned Forgejo tooling and native PTY fixes.
The abandoned custom MCP OAuth extension is not the onboarding path.

Use the owned Apple Silicon build workflow with full runtime/tooling commit
IDs. Its signed-package workflow takes immutable archive, binary and fixture
hashes and validates them before Developer ID signing and notarization. Native
MCP acceptance uses a disposable account through the existing Codex client.
The macOS npm acceptance workflow consumes the resulting signed package and
staged Linux package; it verifies combined archives, native storage, executable
tests, in-place upgrades and the installed MCP workflow. Run it after the
signing/acceptance job succeeds. Failed acceptance does not authorize promotion.

The `authenticated-mcp-internal-alpha` promotion receipt describes measured
native and npm acceptance. It is separate from the independently reviewed
release channel and makes no full-workspace or independent-review claim.
See [MCP authorization](https://cdot65.github.io/prisma-airs-harness/configuration/mcp/)
and [MCP servers with OAuth](GETTING-STARTED-MCP.md).
