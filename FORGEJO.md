# Source, CI and mirror operations

The canonical Rust repository is https://git.cdot.io/cdot/prisma-airs-harness.
The private GitHub repository `cdot65/prisma-airs-harness` is a one-way push
mirror. Push branches to Forgejo. The local `origin` points to Forgejo and
`github` retains the mirror URL. The independent TypeScript repositories in
other Forgejo namespaces are not this project.

Forgejo mirrors all branches and tags on push and hourly using a dedicated
GitHub deploy key. Compare branch/tag hashes before repairing a failed mirror;
never push divergent development directly to GitHub. GitHub Actions is disabled
at repository level. Existing `.github/workflows` files are historical migration
inputs; `.forgejo/workflows` owns execution. Inherited upstream automation must
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

Setup requires the correct Mac short username and its authorized SSH setup key.
Verify `uname -m` returns `arm64`, Xcode command-line tools are installed, and
there is sufficient disk space for the Rust build/cache. Install the official
Homebrew `forgejo-runner` formula plus required build tools. Register a
**repository-scoped** runner from this repository's Settings → Actions → Runners,
with the unique label `airs-macos-arm64:host` and capacity one. Keep the
registration file and token private. Use a user LaunchAgent so native Keychain
checks execute in the intended user's session; verify login/Keychain availability
before claiming unattended acceptance. Do not attach shared owner signing
credentials to untrusted pull-request jobs.

Native Mac compilation, Keychain tests and signing remain pending until this
runner is online and the migrated workflows pass. Signing additionally requires
the Developer ID identity and notary profile documented in the frozen candidate's
`validation/2026-09-13/upstream-0.154/SIGNING-HANDOFF.md`. Runner registration alone
does not satisfy signing or release gates. Windows-native identity validation
also needs a Windows Forgejo runner; no Windows distribution is authorized.
