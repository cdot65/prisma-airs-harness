# Release tooling source validation

Reviewed source: `cce7b09157` stable upgrade/rollback, `8b756bb345` Git-object evidence audit, and `7c80de6947` explicit AIRS product gate. Version stamp: `363f085046`, 0.1.2-alpha.1.mcp.1. Independent source review found no remaining blockers. No numeric release score is assigned before exact native candidate/registry acceptance.

Executed checks: 65 release regressions, six new roundtrip/version contract tests, seven product-gate tests with real complete receipt chains, 13 adversarial Git-object audit tests, 31 launcher tests and eight version-related Rust tests passed. Scoped earlier Rust lint remains in focus3; final repository formatter passed and 19 unrelated baseline formatting changes were restored. No tests were repeated solely because of formatting.

The final development roundtrip used real conversation/MCP turns and native credentials across exact stable bytes, the new development executable, then exact stable bytes. It is deliberately labeled same-version-development-driver-only and npm_upgrade_acceptance:false; it cannot substitute for npm installation of the release candidate. The immutable stable baseline SPEC will supply previous native hashes/source for actual candidate checks.

Git audit verifies committed blob bytes, checksum inventory and stage-reference closure, without trusting working-tree output. Historical stable evidence and the newly committed focus3 evidence passed. Product-gate tests reject skipped required report/refresh cases, missing/broken rollback, stale receipts and focused/pending GNU records. Full GNU failure remains failure even when independent review establishes that it does not block AIRS product acceptance. Stable promotion is not authorized by the product gate.

Exact package acceptance on Linux x64, native Linux ARM64 and signed/notarized Apple Silicon, current full GNU diagnostics, versioned npm publication, anonymous registry checks, published documentation and final committed evidence audit remain pending in this source receipt. Latest stays 0.1.1.
