# Scoped npm installation feasibility — local experiment

Observed 2026-09-08 on Linux, Node v22.23.2, npm 10.9.8 and 12.0.2.
Status: feasibility evidence only. No application packaging change, publication,
real GitHub Packages installation, real credential use, or native build occurred.

## Decision

Do not implement the earlier shrinkwrap proposal as the general install solution.
Published shrinkwrap with the registry `_hasShrinkwrap: true` hint worked on npm10,
but npm12 no longer consumes shrinkwrap. Bundling the exact CLI 5.2.0 and its SDK
0.28.0 solves this synthetic same-scope dependency collision on both npm versions.
A separate change is necessary for native selection: use the actual scoped native
package name with an exact registry version, rather than a direct URL dependency.
This combination passed ordinary scoped-name installation and launcher execution
in both versions, with security defaults unchanged.

The current publisher changes native manifests to scoped names while retaining
unscoped optional-dependency keys whose values become tarball URLs
(`scripts/publish_airs_github_packages.py:106,123`). The matching scoped-key URL
control also failed on npm12; the mismatch is not required for that failure.
A future scoped-name resolution change must update launcher resolution and its
validators coherently, preserve private candidate gates, and verify final hashes.

## Observations and controls

All 26 original install cases are retained in receipts.ndjson, including failures.
The invocation used an isolated global prefix/cache/config and ordinary
`npm install -g @cdot65/prisma-airs-harness@0.1.0-alpha.9`; lifecycle scripts,
audit and funding requests were disabled. No tarball was supplied as the user
install argument, and no npm security-default override was supplied.

| Synthetic case | npm 10.9.8 | npm 12.0.2 |
| --- | --- | --- |
| Unbundled same-scope CLI, no lock or package-lock | Installation failed | Installation failed |
| Published shrinkwrap without registry hint | Installation failed | Installation failed |
| Published shrinkwrap with `_hasShrinkwrap` hint | Installed and ran | Installation failed |
| Bundled CLI/SDK; direct native URL, scoped key | Installed and ran | Install exited 0; launcher failed |
| Same, with `--include=optional` | Installed and ran | Install exited 0; launcher failed |
| Same, required native URL | Installed and ran | EALLOWREMOTE |
| Bundled CLI/SDK; unscoped native key and scoped tarball URL | Installed and ran | Install exited 0; launcher failed |
| Bundled CLI/SDK; scoped native exact registry version | Installed and ran | Installed and ran |
| Bundled CLI/SDK; npm alias to scoped native version | Installed and ran | Install exited 0; launcher failed |
| Native-only unbundled URL control | Installed and ran | Install exited 0; launcher failed |

A separate unbundled URL-alias control served CLI/SDK metadata through the private
registry with tarballs on the public host. npm12 returned EALLOWREMOTE; this mixed
origin case alone cannot isolate a native dependency failure. The native-only
control does isolate it. Alias failure is observed in this loopback registry
layout; it is not proof of all alias behavior against real GitHub Packages.

npm12's direct-URL denial is documented; optional-dependency failure can be hidden
behind installation exit 0. Therefore validation must launch the installed binary,
not only check npm's exit code. No npm downgrade or allow-remote override is proposed.
Original npm10 shrinkwrap failures had an error-code extraction regex that excluded
digits such as E404. Their original empty error-code arrays remain unchanged;
exit codes, request paths and absent runtime retain the failed result accurately.

Two positive auth controls additionally fetched an unrelated public-registry
module. Both hosts received their own synthetic Bearer identity, with zero
cross-host identities or token values in npm output. Neither CLI nor SDK metadata
was requested from either registry when bundled. This verifies the two loopback
origins only; GitHub redirects, TLS and teammate permissions remain untested.

Actual `npm pack --ignore-scripts` with both npm versions included the CLI and
transitive SDK from node_modules when only the CLI was named in bundleDependencies.
It did not bundle the separate native dependency. See npm-pack-controls.ndjson.
These were synthetic JS packages; the native marker is not a real Rust executable.

## Real dependency blocker: platform-native Sharp

The installed real CLI manifest pins SDK 0.28.0 and lists optional docx, pdf-lib,
piexifjs and sharp ^0.35.4. Installed Sharp 0.35.4 declares OS/CPU/libc-specific
@img native bindings and libvips packages. This Linux installation contains only
sharp-linuxmusl-x64 and sharp-libvips-linuxmusl-x64, plus platform-neutral colour.
The CLI and SDK themselves have no os/cpu restriction in their package manifests.
See real-installed-dependency-metadata.ndjson for exact manifests and hashes.

Consequently, blindly copying this host's complete installed CLI tree into a
universal launcher does not establish working DLP image generation on Apple
Silicon, Windows x64, or glibc Linux. A packaging design must either preserve
consumer-side selection of the required non-scoped native dependencies, or
provide and validate the supported platform payloads deliberately. This experiment
has not proven either design. Never add or build Intel Mac targets.

## Next acceptance before implementation or publication

1. Review the bundling/native naming design, exact transitive dependency integrity,
   license inventory, and platform-specific Sharp treatment.
2. Test actual candidate tarballs on fresh npm10/npm12 caches, scoped-name installs,
   both registry authentication contexts, and the real managed CLI and native binary.
3. Run real DLP image/document generation and credential/session tests on Apple
   Silicon, Windows x64 and supported Linux variants. Preserve signing and private
   candidate requirements; do not promote an unvalidated diagnostic artifact.
4. Verify actual GitHub package read access with a teammate and enforce immutable
   release/version/provenance checks. Full release readiness remains incomplete.

## Reproduction and retained evidence

The five .py.txt files are verbatim historical local experiment scripts, not supported
application tooling. Their text extension preserves exact experimental bytes. Copy
them to .py names and run only in a disposable directory containing the verified
npm10 tool at npm10/package/bin/npm-cli.js; they create isolated local registries,
synthetic packages, configs and caches alongside themselves. npm12 must be available
as `npm`. All auth sentinels in those files are fabricated. Do not substitute real
credentials or real registry endpoints. Sources and original receipts have hashes
in evidence-index.json. Original full artifacts remain under
/var/tmp/airs-npm-shrinkwrap-experiment. Each receipt is one JSON line.

Primary references:

- [npm12 package-lock documentation](https://github.com/npm/cli/blob/latest/docs/lib/content/configuring-npm/package-lock-json.md): shrinkwrap removal.
- [npm12 breaking changes](https://github.blog/changelog/2026-06-09-upcoming-breaking-changes-for-npm-v12/): remote dependency default denial.
- [npm registry metadata](https://github.com/npm/registry/blob/main/docs/responses/package-metadata.md): _hasShrinkwrap hint.
- [npm package.json reference](https://docs.npmjs.com/files/package.json/): bundled and optional dependencies.
- [Sharp installation](https://sharp.pixelplumbing.com/install/): supported native platforms and cross-platform installation.
