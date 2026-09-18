# Prisma AIRS welcome mark — onboarding.2

The welcome screen now uses the owner-supplied cyan Prisma AIRS mark. Its transparent triangular cutout and proportions remain fixed while a subtle light sweep animates the fill. Static and colorless modes are covered by snapshots and actual terminal captures.

Runtime source: `71f2ea33144cc0535772e8408a3037502a82b02e`. Subsequent tooling-only commits extend upgrade acceptance to onboarding versions; they do not change native binaries.

- 4,334 TUI tests passed, six skipped; eight home-directory tests passed.
- 21 preview checks passed, including dark/light backgrounds, static/colorless operation, narrow terminals and terminal restoration.
- Installed local HTTPS OAuth/native-store checks: Linux x64 18, native ARM64 18, Apple Silicon 12.
- Installed shell/terminal checks: 8 / 8 / 7 respectively.
- Each platform passed 46 executable checks with one platform-inapplicable skip, five bundled CLI check groups, and three upgrade cases from onboarding.1 with preserved configuration.
- Apple Silicon is Developer ID signed and notarized; installed bytes and native Keychain operation passed.

The original attachments were PNGs. The checked-in SVG is a vector transcription of the actual logo reference; the second attachment was an SVG file icon. [Terminal gallery and screenshots](https://git.cdot.io/attachments/37a6667d-129c-498b-96cc-c6d6469de32c).

ARM64 acceptance used Jadzia's native Linux VM. Its disposable fixture required `libsecret-tools` to probe its isolated keyring; `secret-tool` is not a harness runtime dependency. Initial fixture failures and their correction are recorded in the acceptance scope. Upgrade tests were rerun after correcting onboarding-version parsing.

`0.1.0-alpha.22.onboarding.2` is published at `https://npm.cdot.io` under `latest`, `alpha` and `onboarding`. Anonymous fresh registry installations passed on all three platforms. An additional unpinned install selected the correct version without `--include=optional`. Registry integrity and default-tag verification are recorded in `PUBLICATION.json`, the platform registry receipts, `FINAL-TAGS.json` and `DEFAULT-INSTALL.json`. Publication preserves the accepted executable, launcher and bundled CLI payloads; only authorization/provenance metadata changes from candidate tarballs. The original review archive and previous npm versions remain immutable.

These fixture checks do not establish an attended production SSO login or ServiceNow call. Historical gateway-lifecycle and baseline HTTP test limitations remain explicit in package validation metadata.

The owned test container and LaunchAgents are removed, Jadzia’s Docker engine is restored to stopped, and the metadata-only Keychain audit found zero recent service entries. Production credentials and trust settings were preserved.
