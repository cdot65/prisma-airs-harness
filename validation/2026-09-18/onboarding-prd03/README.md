# PRD 03 — native acceptance and review handoff

The branded AIRS onboarding candidate passed installed acceptance on Linux x64, native Linux ARM64 and Apple Silicon. All three PRDs have an **agent-assessed 9/10**, with owner review pending. The final native runtime is `0317a340486f83f4973ee71a22f061ea31226a76`; package assembly is `565f8c32388dcc44592677c45155a2d9accd84a1`; final acceptance tools are `d4070e3eb5`.

[Harness PR 47 and candidate download](https://git.cdot.io/cdot/prisma-airs-harness/pulls/47) · [Documentation PR 1](https://github.com/cdot65/prisma-airs-reference-architecture/pull/1) · [Terminal gallery](gallery/index.html)

## Try the candidate

Download the PR attachment `airs-onboarding-review-0.1.0-alpha.22.onboarding.1.tar.gz`, verify SHA256 `005825ef352bf14f9c3a337844bddb489c9d47605969ab757eb5d5ba93b4d7d4`, and extract it. From the extracted directory, with Python 3.11+ and Node.js 22.14+ in the 22.x line or Node.js 24+:

```sh
python3 install.py --prefix "$HOME/airs-onboarding-review"
export PATH="$HOME/airs-onboarding-review/bin:$PATH"
export AIRS_HARNESS_HOME="$HOME/airs-onboarding-review/review-home"
airs --version
airs cli --version
airs
```

Expect harness `0.1.0-alpha.22.onboarding.1` and bundled CLI `7.0.0`. The archive includes native packages for all three platforms, the managed product CLI, dependencies and eight skills. No separate product CLI installation or `--include=optional` flag is needed. The installer verifies manifest hashes, installs through a temporary loopback package source, and checks the installed binary. It requires a new prefix and leaves the global installation and shell startup files unchanged.

Keep the two exports active throughout the review. Use the documented gateway and company SSO settings, then follow the canonical SSO-to-ServiceNow walkthrough. For cleanup, log out of each review environment before removing the chosen review prefix. A fresh shell returns to the existing global installation.

This explicit review archive is **not published to npm**. `latest`, `alpha` and `migration` remain alpha.22; `gateway-validation` remains alpha.21. Docusaurus changes are tested in PR 1; they have not been merged or deployed to Pages.

## Acceptance

| Installed platform | Onboarding | Shell/terminal | Existing regressions | CLI / upgrade |
| --- | --- | --- | --- | --- |
| Linux x64 | 18 passed | 8 passed: bash/zsh/fish | 46 passed, 1 platform skip | 5 groups / 3 cases passed |
| Native Linux ARM64 on Jadzia | 18 passed | 8 passed: bash/zsh/fish | 46 passed, 1 platform skip | 5 groups / 3 cases passed |
| Apple Silicon on Jadzia | 12 passed | 7 passed: bash/zsh | 46 passed, 1 platform skip | 5 groups / 3 cases passed |

The CLI groups include 20 capability commands and native PDF/PNG/JPEG/SVG/DOCX handling. Upgrade cases preserve environment configuration and cover legacy launcher ownership. Native hashes and exact workflow runs are in [ACCEPTANCE.json](ACCEPTANCE.json). Developer ID signature and online notarization were verified on the installed Mac executable, in addition to successful GUI-context Keychain authentication and cleanup.

PRD 01/02 implementation validation includes 5,264 CLI/TUI passes with six skips, 256 final AIRS-specific passes, 129 Python cases (127 passed/two skipped), and 21 PTY checks. The corrected CLI/identity/HTTP run passed 1,045 of 1,051: all 931 CLI and 28 identity cases passed, while HTTP passed 86 of 92. Six unchanged HTTP TLS fallback/classification failures reproduce on the prior candidate source. [CUSTOM-CA-REGRESSION.json](CUSTOM-CA-REGRESSION.json) names them; this combined suite is not claimed fully green.

Docusaurus content and production build pass; all 20 browser tests pass. Desktop/mobile renders were inspected without viewport overflow. [Hosted check](https://github.com/cdot65/prisma-airs-reference-architecture/actions/runs/35311111734) passes at documentation commit `345559a`.

## Evidence and reproduction

`native-receipts.tar.gz` contains installation/network receipts, native build/signing metadata, CLI/upgrade receipts, per-platform logs and terminal galleries. Within each platform, **`onboarding-final` and `terminals-final` are authoritative for final visual acceptance**. Earlier capture directories remain historical: pyte originally ignored DEC 1049 alternate-buffer switches. The corrected driver and its regression test preserve clean alternate screens and restore the primary screen after resize. Affected installed onboarding and terminal checks were rerun on all three unchanged binaries.

Run the native fixtures only against disposable local review state, using the installed launcher and its native binary. On Linux, install pyte 0.8.2, OpenSSL, D-Bus, gnome-keyring, libsecret-tools, bubblewrap, ripgrep and the three shells. Run through a private D-Bus session:

```sh
dbus-run-session -- python3 scripts/validate_airs_onboarding.py \
  --binary /path/to/prefix/bin/airs \
  --native-binary /path/to/native/bin/airs-harness --output /path/to/evidence
python3 scripts/validate_airs_onboarding_terminals.py \
  --binary /path/to/prefix/bin/airs \
  --native-binary /path/to/native/bin/airs-harness --output /path/to/terminal-evidence
```

On Apple Silicon, use `validate_airs_onboarding_macos.py` from a GUI user session with the native login Keychain available; the terminal driver accepts `--shells bash zsh`. Keep the repository's test-only identity fixtures alongside the scripts. Do not point either driver at a production identity service. `scripts/test_airs_onboarding_terminal.py` checks alternate-buffer capture fidelity independently. The review installer/packager are `scripts/install_airs_review.py` and `scripts/package_airs_review.py`.

## Iterations and limits

- Native Mac acceptance found an actual OIDC custom-CA inconsistency. Discovery now reuses the existing shared HTTP custom-CA builder, with strict chain/hostname checks intact. All three native artifacts were rebuilt; earlier `c88ed444d3` artifacts are superseded.
- PTY cleanup now checks restored modes through the master on macOS and drains output during exit. The browser-unavailable fixture retains Node while hiding browser programs. Mac returning-user acceptance expects the direct agent path, and the local identity fixture implements refresh-token revocation so logout can complete truthfully.
- Capture review found pyte's missing alternate-buffer support; all affected native captures were regenerated after correcting the test driver. Two ARM capture-run setup omissions (test JWKS and libsecret-tools) were repaired in the disposable container before the passing rerun.
- Local HTTPS fixtures cover PKCE/device/manual SSO, persistence, hidden key entry, returning sessions, gateway denial, cancellation, native stores, environment/history preservation and fail-closed TLS. They do **not** establish production company SSO, a live ServiceNow call or hourly frontend renewal. Owner visual and attended end-to-end acceptance remain pending.
- The final score is 2/2 provenance, 2/2 native platforms, 1.5/2 end-to-end quality, 2/2 documentation/review usability and 1.5/2 regression/hygiene: **9/10**. Half a point is reserved for attended owner acceptance and half for the documented baseline HTTP test gap.

## Cleanup

Both owned ARM containers are removed. All three onboarding LaunchAgents are unloaded; the workspace-specific Bazel daemon is stopped. Jadzia's engine is restored to stopped, preserving its previously running Docker Desktop UI and existing image. The final metadata-only Keychain audit finds zero entries created during this effort under the two harness services; credential values were never requested by the audit. Native Keychain trust/search lists were not modified. Fixture state is isolated and removed by the drivers; review binaries and evidence remain available in the private work cache.
