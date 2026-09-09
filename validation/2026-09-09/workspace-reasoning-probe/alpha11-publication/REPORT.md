# Prisma AIRS Harness alpha.11 workspace-key verification fix

The gateway connectivity check now recognizes valid Responses API output containing reasoning without a message. Previously, a successful bounded probe on the workspace default reasoning route could be reported as failed, even though the credential worked.

The probe remains limited to 16 output tokens and a fixed connectivity prompt. Default routing still omits the model field. Credential storage, gateway routing, and remote MCP authorization are unchanged.

Runtime source: `ff5e337e4250771933ad012f0b89109ee9a22568`. Product version: `0.1.0-alpha.11`. Bundled Prisma AIRS CLI: `5.2.0`; SDK: `0.28.0`.

Validation includes the actual reported reasoning-only response shape, negative/error probe cases, live Linux workspace-key inference with local tools and resume, native Linux Secret Service, and signed Mac Keychain lifecycle tests. Mac signature and notarization are checked before packaging and after npm installation.

Independent Linux and signed Mac reviews each scored the bounded prerelease distribution scope 9/10. This does not claim a completed audit of every authentication scenario or a new live Mac OIDC/MCP test.

Publication succeeded in [run 34415694911](https://github.com/cdot65/airs-harness/actions/runs/34415694911). All three private packages are version `0.1.0-alpha.11` on GitHub Packages, tagged `auth-review`; downloaded archive SHA256/SRI values match the reviewed plan.

Fresh package-name installation succeeded in [run 34415883459](https://github.com/cdot65/airs-harness/actions/runs/34415883459), covering Apple Silicon macOS26 and Linux x64 with npm10.9.8 and npm12.0.2. Installed native hashes, Mac signing/notarization, Keychain, managed CLI, and deterministic integration tests passed.

The publication gate rejected an intermediate package because the owner-reported notarization ID was absent from its signing receipt. Workflow commit `563c32d3e` propagates and checks the exact submission before packaging. The gate was not relaxed; corrected Mac run34414804362 passed without a native rebuild.

Update from an already authenticated npm installation:

```sh
npm install -g @cdot65/prisma-airs-harness@0.1.0-alpha.11 --include=optional --registry=https://npm.pkg.github.com
airs-harness --version
airs-harness doctor --verify-access
```

Expected version: `airs-harness 0.1.0-alpha.11`. Use the existing environment and stored sign-in. The affected owner's actual Mac workspace-key check remains the final hands-on confirmation; CI does not establish teammate package authorization or every authentication scenario.


Final independent delivery review: **9/10 for the published prerelease distribution**, with the scope limits above retained. See `FINAL-DELIVERY-REVIEW.json` for the exact evidence bindings.
