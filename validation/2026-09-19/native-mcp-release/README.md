# Native MCP release evidence — September 19, 2026

`airs-harness@0.1.0-alpha.22.mcp.4` was published under the `mcp` tag and passed fresh anonymous registry acceptance on Linux x64, native Linux ARM64 and Apple Silicon. The `latest`, `alpha` and `onboarding` tags remained `0.1.0-alpha.22.onboarding.4`. These are installed-client checks against synthetic identity and gateway fixtures, not production SSO or ServiceNow acceptance.

This directory retains selected receipts exactly as written. `RECEIPT-MANIFEST.json` maps each copied file to its original scratch-relative location, byte length and SHA-256. `SHA256SUMS` covers all retained files except itself. `BUNDLE-CHECK.json` records the collection checks. No application code, release package or original receipt was modified while collecting this evidence.

## Release identities

| Boundary | Recorded identity |
| --- | --- |
| Native runtime and acceptance tooling source | `3e43436ce0bbffef796fac0cd5eee0527e5ead15` |
| Final package assembly source | `667822f50c1cea5a93c783d7d3811cec0f4dc845` |
| Canonical specification digest | `18967908e19961324dc20ec6feb4d7d579e2a230689213165966e046813771dd` |
| Final registry evidence digest | `d5e424a6480bd3f76691ab96115fbced6bb3a71ef9cb597743fafa4b431fb1e0` |
| Linux x64 native SHA-256 | `ba61eb9016130a6b9c4c834b215149fe2142b2abdc145e93ef81462a010ca236` |
| Linux ARM64 native SHA-256 | `ab4d9860c625466fb4acf1279105a93aff26fd3f38d87aa855c0dc91e575007f` |
| Apple Silicon native SHA-256 | `fd18bb6000be5758e122544e953a40e74c3e3ef4ce3d98a1f8d21238cea462f8` |

The packaging revision corrected the bundled Ubuntu helper's default package version to mcp.4 and its accompanying guide. It did not change the already built native executables or acceptance tooling. All three standard candidate matrices and six additional lifecycle/upgrade checks were completed against this final revision, called R2 in the original records. R1 was superseded before publication.

Publication added validated release metadata to the candidate archives. `publication/PUBLICATION.json` records the changed archive entries, original candidate hashes, final published archive hashes and unchanged runtime payloads. Candidate archive hashes therefore differ from published archive hashes; native executable hashes remain the same. `packages/` retains both package metadata receipts. A launcher's JavaScript hash is not the native executable hash.

The specification and collectors contain canonical JSON digests. `SHA256SUMS` and the receipt manifest contain hashes of actual file bytes. These can differ without indicating a mismatch.

## What is retained

- `SPEC.json`, the candidate and registry collectors, and the four-package publication receipt bind the version, platforms, runtime, packaging and channel state.
- `candidate-acceptance/` and `registry-acceptance/` contain per-platform acceptance records, the frozen tooling manifests, individual stage receipts and structured stage outputs. The stages cover installation, onboarding, terminals, installed regressions, MCP management, diagnostics, the bundled CLI, in-place upgrade and command output. Apple Silicon additionally verifies signing and notarization.
- `PHASE4-INDEPENDENT-REVIEW.json` records the 9.6/10 candidate review and its nine gate groups. Supporting artifact, packaging and candidate reviews retain the evidence used at that time.
- `extra-candidate/` retains each native platform's approximately 178-second lifecycle result and upgrade from onboarding.4. Each lifecycle check completed two post-expiry cycles for both inference and MCP, three verified tool turns and native credential cleanup. The standard upgrade receipts cover mcp.3; both upgrade origins passed three cases per platform.
- `ubuntu-supplementary/` records an additional isolated Ubuntu check of the final candidate. Its scope and the later helper limitation below must be read together.
- `lesson-review/` records the final six Education source hashes, independent 9.5/10 content review, and comparison with the vault's existing validation findings. It does not establish successful site export, browser tests or deployed GitHub Pages content; those require separate deployment receipts.

The required native-store positive test ran on all three platforms: MCP OAuth persisted to the native store, a second process reused the saved credential without another authorization exchange, logout removed the record, and inference remained usable. Existing storage-policy preservation and bundled CLI checks ran on every target. Platform-specific skips are listed in the original installed-regression outputs; they do not waive the native positive check.

## Ubuntu helper limitation discovered after publication

The mcp.4 package includes the Ubuntu preparation helper from packaging commit `667822f50c1cea5a93c783d7d3811cec0f4dc845`. A separate owner-session repair subsequently identified that activating Secret Service and then running bare `gnome-keyring-daemon --unlock` could start a competing daemon instead of unlocking the service already owning the D-Bus name. The original encrypted credentials were preserved; the user's real store still required local password entry at the time of that repair record.

The separate correction is recorded in commit `834df8e91d` and uses an explicit replacement/unlock flow with collection-state checks. That correction and its independently delivered script are **not** in the published mcp.4 package. The correction was integrated separately as `1fd617abb4`; `ubuntu-supplementary/CORRECTED-SCRIPT-HANDOFF.json` binds its delivered script to SHA-256 `bc31a5a289e038456e191fcb40bbaf34a630c718039caca47a34f6725d259322` and an mcp.4 installation default. The accompanying review records four passing checks from a new isolated D-Bus/XDG regression: encrypted keyring creation and native write/read/delete, unlocked-service reuse, locked/wrong-password preservation, and successful recovery with the correct password. The test used an installed-version override to preserve the owner's existing mcp.3 prefix. It did not enter the owner's real keyring password. This bundle does not claim that the old packaged helper is ready for an existing locked owner keyring, nor that publishing the harness resolved the owner's session incident.

The supplementary Ubuntu pass used a private D-Bus session, disposable native credentials and an isolated installation. It established package/fixture behavior on that host; it did not test the old helper against the owner's existing locked collection. The native harness artifacts and their recorded acceptance identities are unchanged by this later shell-helper finding.

## Timing and remaining scope

The 60-minute active and 35-minute quiet-idle observations are retained separately in [native-mcp-lifecycle](../native-mcp-lifecycle/README.md). They used a recorded development executable, not the published mcp.4 bytes. The active run measured 3,601.079 seconds and seven post-expiry cycles per resource. The idle run measured 2,100.085 quiet seconds without HTTP requests and renewed on return while the fixture's refresh grants remained valid. Those results do not override a production issuer's idle policy or establish production revocation and concurrent refresh behavior.

Phase 4 reviews were written before publication and correctly state that registry verification was then pending. The later `REGISTRY-VERIFIED.json` closes that package-installation gate; historical review text was preserved rather than rewritten. Neither result establishes a full Rust workspace pass, production company SSO, a real workspace-key route, upstream ServiceNow authentication or the attended owner credential-store recovery.

This is a curated private-repository evidence ledger. It excludes raw stdout/stderr, terminal transcripts, observation event streams, credentials, private-key material, package archives and unpacked installations. Original structured receipts retain nonsecret registry endpoints, command paths and hashes of omitted outputs. Their original relative references describe the source run layout; use the receipt manifest to find relocated retained files. The bundle is not a standalone rerun workspace for the complete release collector, and must not be exported wholesale to the public educational site.

To check retained bytes from this directory:

```sh
sha256sum -c SHA256SUMS
```
