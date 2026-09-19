# Publish and verify an AIRS test package

Use `scripts/airs_test_release.py` for the owner-authorized **mcp test channel**.
It verifies candidate inputs, runs installed acceptance on three native hosts,
stages metadata changes, publishes native packages before the launcher, and
checks fresh anonymous registry installations. It does not promote a stable
release. Existing non-`mcp` tags, including `latest`, `alpha` and `onboarding`,
must remain unchanged.

This runbook describes the process; it is not a publication receipt. Consult the
actual release specification, acceptance evidence and `PUBLICATION.json` before
claiming that a version is ready. See [Getting started](GETTING-STARTED.md) for
installation and identity workflows, and [RELEASE.md](RELEASE.md) for the wider
release policy and existing platform build/signing procedures.

## Prepare immutable inputs

Build the versioned native candidates using the existing release workflows.
Retain their build provenance and the Apple Developer ID signing/notarization
receipt. Package all three native candidates plus the bundled launcher with
`scripts/package_airs_npm.py`; this runbook starts with that candidate directory,
including `NPM-PACKAGES.json`, `tarballs/`, and its unpacked package directories.

Use three explicitly recorded, full 40-character commit IDs:

| Specification field | What it identifies |
| --- | --- |
| `source_commit` | The runtime source used to build the native binaries |
| `tooling_commit` | The committed validators, fixture tests and release tools used for acceptance |
| `packaging_commit` | The committed npm launcher and packaging tools used to create the candidates |

These are separate identities even when two values happen to match. Do not use
`HEAD`, a branch name, or a validation commit as a substitute for the runtime
build's recorded source. Commit the validation tools before snapshotting them;
modified or untracked Python files in `scripts/` cannot be represented as bytes
from an older commit. Do not rebuild or edit the native payload during staging.

Create `SPEC.json` with the following fields. Replace every angle-bracket value
with evidence from the actual build. The example version is illustrative; it
does not establish that `0.1.0-alpha.22.mcp.3` has been published.

```json
{
  "schema_version": 1,
  "scope": "owner-authorized-test",
  "version": "0.1.0-alpha.22.mcp.3",
  "tag": "mcp",
  "registry": "https://npm.cdot.io",
  "source_commit": "<40-character runtime commit>",
  "tooling_commit": "<40-character validation commit>",
  "packaging_commit": "<40-character packaging commit>",
  "previous_version": "0.1.0-alpha.22.mcp.2",
  "developer_id_team": "<10-character Developer ID team>",
  "platforms": [
    {
      "target": "x86_64-unknown-linux-musl",
      "binary_sha256": "<64-character native binary SHA-256>"
    },
    {
      "target": "aarch64-unknown-linux-musl",
      "binary_sha256": "<64-character native binary SHA-256>"
    },
    {
      "target": "aarch64-apple-darwin",
      "binary_sha256": "<64-character signed native binary SHA-256>"
    }
  ]
}
```

The previous version must be an exact existing version, distinct from the new
version, available anonymously from the selected registry. The upgrade check
installs it and exercises configuration and legacy-command preservation.
Registry URLs must use HTTPS and contain no credentials, query or fragment.

Use canonical absolute paths for every working directory. On macOS, `/tmp` and
`/var` may be system symlinks; use `/private/tmp`, `/private/var`, or a canonical
`/Users/...` path. Evidence helpers reject symlink ancestors and linked outputs.
Resolve the intended parent directory before starting, then keep evidence in
real directories beneath it.

The following examples use operator-selected paths:

```sh
export AIRS_RELEASE_ROOT=/absolute/canonical/release-work
export AIRS_VALIDATION_ROOT=/absolute/canonical/validation-checkout
export AIRS_CANDIDATES=/absolute/canonical/candidate-npm
export AIRS_TOOLING_COMMIT='<full-validation-commit>'
```

The release root contains `SPEC.json`. Keep candidate inputs, evidence, staged
packages and publication output in distinct directories. Do not place outputs
inside the validator source or package inputs.

## Snapshot, inspect and accept the candidates

On the source host, bind the committed validator files to the transfer manifest:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" snapshot-tooling \
  --scripts "$AIRS_VALIDATION_ROOT/scripts" \
  --commit "$AIRS_TOOLING_COMMIT" \
  --output "$AIRS_VALIDATION_ROOT/ACCEPTANCE-TOOLING.json"

python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" inspect \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --packages "$AIRS_CANDIDATES"
```

Transfer the same specification, complete candidates, validator `scripts/*.py`,
the complete `scripts/fixtures/` tree, and `ACCEPTANCE-TOOLING.json` to each
acceptance host. Also transfer
`codex-rs/airs-identity/src/fixtures/test-only-*`, preserving paths relative to the
validation root. These are synthetic fixture keys. The manifest remains beside
`scripts/`; additional, missing or changed Python/fixture files are rejected. The recursive
fixture inventory includes the npm command-wrapper reference templates.
Do not copy real account credentials into this payload.

Run acceptance on **native Linux x64, native Linux ARM64, and Apple Silicon**.
Jadzia's ARM64 Linux VM is the established ARM64 acceptance host. Cross-compiling
a binary does not satisfy installed native acceptance. Each host needs supported
Node/npm, Python 3.11 or newer (the installed tests import `tomllib`),
`pyte==0.8.2` available to that Python interpreter, OpenSSL, and the prerequisites
used by the existing native fixtures. Prepare a dedicated Python environment and
invoke this runbook with its `python3`; validators use that same interpreter.
Linux credential fixtures use an isolated D-Bus session and native credential
service (`dbus-run-session`, `gnome-keyring-daemon` and `secret-tool`); macOS fixtures use owned test entries and verify their cleanup. Apple
acceptance also verifies the installed binary's Developer ID and notarization.

On each host, set the path variables to its local copies and run:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" accept \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --packages "$AIRS_CANDIDATES" \
  --scripts "$AIRS_VALIDATION_ROOT/scripts" \
  --output "$AIRS_RELEASE_ROOT/candidate-acceptance"
```

The command selects the actual native target and writes
`candidate-acceptance/<target>/ACCEPTANCE.json`. Required stages include normal
npm installation; local SSO/workspace-key credential fixtures; terminal behavior;
all `test_airs_harness*.py` fixtures; `/mcp` and `/doctor`; the managed CLI;
upgrade; public command output; and installed Mac signature checks where
applicable. Empty or entirely skipped unittest suites cannot pass.

Copy each complete `<target>/` directory back to the collector's
`candidate-acceptance/`, including its logs, files and receipts. Keep the sibling
`<target>.work/` on its execution host for resume; it contains the disposable
installed prefix and is not portable acceptance evidence. The collector does
not need the remote machine's `node_modules` tree.

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" verify-all \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --acceptance "$AIRS_RELEASE_ROOT/candidate-acceptance" \
  --installation candidate
```

## Stage and publish

Create a fresh staging directory only after all three candidate acceptances pass:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" stage \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --packages "$AIRS_CANDIDATES" \
  --acceptance "$AIRS_RELEASE_ROOT/candidate-acceptance" \
  --output "$AIRS_RELEASE_ROOT/staged"
```

Staging re-verifies acceptance and candidate archives, retains the original
candidate archives, and permits only the defined publication/provenance metadata
changes. Runtime file bytes must remain identical. Staged output contains
archives and retained candidates; unpacked package directories are not required
for subsequent registry or upgrade checks.

Publish using an existing private npm user configuration file authorized for
this registry. On Unix it must have owner-only permissions. Supply its path,
never a token value, and keep it outside the evidence payload:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" publish \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --packages "$AIRS_RELEASE_ROOT/staged" \
  --acceptance "$AIRS_RELEASE_ROOT/candidate-acceptance" \
  --output "$AIRS_RELEASE_ROOT/publication" \
  --userconfig /absolute/private/npm-userconfig
```

Publication re-verifies staged evidence before mutation. It publishes Linux x64,
Linux ARM64 and Apple Silicon native packages, then `airs-harness`. It checks the
immutable archive integrity and `mcp` tag after each package, and preserves every
non-`mcp` tag. `PUBLICATION.json` records the checkpoint and completion state.
A publication receipt alone does not establish a working fresh registry install.

## Verify fresh anonymous registry installations

Copy the complete staged directory and the collected **candidate** acceptance
set to each native host. Keep these immutable. Run the same installed acceptance
through a fresh anonymous registry install, with separate output:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" verify \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --packages "$AIRS_RELEASE_ROOT/staged" \
  --acceptance "$AIRS_RELEASE_ROOT/candidate-acceptance" \
  --scripts "$AIRS_VALIDATION_ROOT/scripts" \
  --output "$AIRS_RELEASE_ROOT/registry-acceptance"
```

This invokes `validate_airs_test_registry_install.py` with isolated npm
configuration and a fresh cache. It checks published immutable archive integrity,
installs without publication credentials, verifies actual installed file bytes,
and runs the same remaining acceptance stages. It does not claim packet-capture
proof of network isolation. Copy each completed target's portable evidence back
to the collector, then run:

```sh
python3 "$AIRS_VALIDATION_ROOT/scripts/airs_test_release.py" verify-all \
  --spec "$AIRS_RELEASE_ROOT/SPEC.json" \
  --acceptance "$AIRS_RELEASE_ROOT/registry-acceptance" \
  --installation registry
```

Candidate and registry receipts have different installation identities and
cannot substitute for one another. Declare the test version ready only after
publication and all three fresh registry acceptances are verified.

## Resume without skipping checks

Re-run `accept` or `verify` with the same arguments plus `--resume`. There is no
`--start-at` option. A completed stage is reused only after its input identity,
command receipt, predecessor receipts and every retained output hash still
match. The actual installed native, launcher and managed bundle are rechecked on
the execution host before reuse/final acceptance. Missing local install state,
changed tooling, changed archives or tampered evidence stops resume. Restore the
original evidence, or use a fresh output directory and run acceptance again.

Re-run `publish` with exactly the same specification, staged inputs, candidate
evidence and publication directory to continue a partial publication. It reads
its checkpoint and the registry again. A matching existing immutable version
must also already be the `mcp` target. Different bytes or changed protected tags
stop publication; the tool does not overwrite versions or silently move tags.
`stage` requires a fresh output directory rather than a partial-stage resume.

Legacy validators run as normal Python subprocesses with optimization disabled;
the orchestrator's own receipt checks remain active under `python3 -O`. A test
failure, unavailable native prerequisite, or absent target receipt is not an
accepted skip.

## Acceptance boundaries

These stages use synthetic identity/gateway fixtures and disposable installs.
They do not attest to a new human SSO session, a real AI Gateway workspace key,
or an attended ServiceNow operation. The owner's Ubuntu session investigation
and live identity/ServiceNow acceptance were explicitly deferred until the
follow-up session. Keep those items marked **deferred**, not passed, and retain
that distinction in the test release notes. Stable promotion is a separate
release decision.
