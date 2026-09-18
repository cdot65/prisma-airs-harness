# Branded onboarding npm publication

Published `0.1.0-alpha.22.onboarding.1` at `https://npm.cdot.io` on September 18, 2026, after the owner reported that the new version was missing from npm. The prior delivery provided only a review archive; this receipt establishes the subsequent registry publication.

```sh
npm install -g airs-harness@latest --registry=https://npm.cdot.io
airs --version
airs cli --version
```

Expected harness: `airs 0.1.0-alpha.22.onboarding.1`. Bundled product CLI: `7.0.0`. No extra CLI installation or `--include=optional` flag is required under ordinary npm settings. If a shell still selects the isolated review archive, start a fresh terminal and inspect `type -a airs`.

All four packages (`airs-harness`, `airs-harness-linux-x64`, `airs-harness-linux-arm64`, `airs-harness-darwin-arm64`) have `latest`, `alpha` and `onboarding` at the new version. Historical `migration` remains alpha.22 and `gateway-validation` remains alpha.21. Existing versions were not overwritten.

## Provenance and scope

Native runtime source: `0317a340486f83f4973ee71a22f061ea31226a76`. Original launcher/bundle assembly: `565f8c32388dcc44592677c45155a2d9accd84a1`. Publication preparation starts from `fd83757474cc297a6bca9a53983c1fdb5fa26b72` and binds the prior installed acceptance to a truthful owner-requested prerelease receipt. Full production gateway lifecycle and independent release review are not claimed.

The private candidate packages were copied into a separate publication stage. Only package/provenance/validation metadata changed. `NPM-PACKAGES.json` compares every archive entry against the review candidate and proves that native executables, launcher code, bundled CLI dependencies and skills are unchanged. Mac signing bytes remain unchanged. Original candidate metadata remains in the published validation evidence. The original review archive and its checksum remain immutable.

`PUBLICATION.json` records exact archive integrity and the original registry tags. All native packages were published first under `onboarding`, followed by the launcher. `latest` and `alpha` were promoted only after fresh anonymous registry installs passed on every supported native platform.

## Fresh registry acceptance

- Linux x64: native hash verified; 47 executable tests (46 passed, one platform skip); seven environment lifecycle/preservation checks; bundled CLI 7.0.0.
- Native Linux ARM64 on Jadzia: the same checks pass on the actual ARM64 Docker VM. No emulated acceptance is substituted.
- Apple Silicon: the same checks pass, plus installed signature verification and native Keychain login/inference/logout acceptance.
- An additional unpinned anonymous Linux install resolves the new default correctly without `--include=optional`; exact binary hash and bundled CLI version match.
- Documentation content/build and all 20 browser tests pass with the npm instructions. The documentation PR remains the review surface; this action does not itself deploy Pages.

The earlier [onboarding acceptance](../onboarding-prd03/README.md) contains the full HTTPS OAuth/terminal tests and captures on these same runtime bytes. Its archive-only and unchanged-tag statements describe the earlier review stage and are superseded by this publication receipt. Six unchanged baseline HTTP tests remain documented; production SSO and ServiceNow acceptance still require an attended owner session.

## Operational cleanup

The owned ARM registry-test container and Mac LaunchAgent are removed. Docker's engine is restored to stopped while its existing Desktop UI remains available. The final metadata-only Keychain audit finds zero recent harness fixture entries. The temporary publication credential file is deleted. The original global installation and production environments were not modified.

The first Docker start was unresponsive; restarting Desktop restored the engine, and native acceptance was then rerun successfully. Its failed startup log is retained separately from the passing registry acceptance.
