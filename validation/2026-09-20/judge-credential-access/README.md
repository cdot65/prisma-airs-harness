# Judge credential access — alpha.5 release evidence

Runtime and packaging source: `4509d98550b8919de0ff687e40ab4f80f00f122c`.
Validation tooling: `86563415a79e8b8a95ef3b12139e3e1276eb422a`.
The initial inventory check failed before package acceptance; its logs are
preserved under `failed-tooling-inventory`. The corrected inventory binds the
embedded skill and detects skill mutation/removal; all 17 validator tests passed.
Harness `0.1.2-alpha.5.mcp.1` bundles CLI `7.1.5` on private npm's `mcp` channel.
Stable `latest` remains `0.1.1`.

The bundled skill requests per-command approval for live native credential-store
and TypeSafe network access. A saved binding that cannot be read is no longer
misreported as absent setup. Dry-run/replay remain offline paths.

Candidate and anonymous registry acceptance cover Linux x64, native Linux ARM64,
and signed/notarized Apple Silicon. The `typesafe-node` stage now also runs the
actual agent tool executor and approval UI, requiring embedded skill bytes to
match source. It verifies saved native keys, inherited-key precedence, rejection
without any provider call, fresh SDK artifacts, verbatim response strings, and
no keys in inference bodies, tool output or judge artifacts.

Inference decisions are scripted and TypeSafe responses come from a loopback
fixture. These checks do not prove model instruction-following, live Jev accuracy,
or an owner-account paid evaluation. No paid Jev requests were made.

The earlier local source candidate and original alpha.4 evidence remain distinct.
The owner report exposed an execution-boundary gap in the older subprocess-only
acceptance; those old receipts are not proof of in-session credential access.
Historical platform artifacts were relocated only after backup verification;
backup archives remain outside Git and their receipts are retained here.
