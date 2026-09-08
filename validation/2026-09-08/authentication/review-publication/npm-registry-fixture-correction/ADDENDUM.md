# Correction to npm URL-denial fixture evidence

The earlier independent review treated a passing negative test in `f6bb7db09` as evidence that npm12 URL dependencies reached and were blocked by the rejecting proxy. That conclusion was too broad and is withdrawn. The test only required a nonempty list of unexpected requests; it did not require the intended dependency request to be present.

The first real maintained-bundle installation exposed two unrelated `/npm` update-notifier lookups. npm itself exited successfully, but the validator rejected those lookups. The unchanged network receipt is retained as `original-real-bundle-notifier-failure.json`. This background traffic could satisfy the earlier generic assertion and produce false-positive URL coverage. The original negative tests did not retain per-case request receipts; this addendum does not invent a historical raw log.

Corrected source `c15a6af20` disables npm's update notifier in the isolated validation environment. Its negative fixture first verifies a complete synthetic five-package inventory, then omits only `airs-bundle-missing-fixture` from the served archive. The inventory bytes remain in the archive and are bound to the staging receipt by SHA-256. Each test must fail before native execution and must not create a successful install-verification receipt.

The corrected fixture and eight retained receipts establish these distinct outcomes:

| Consumer | Missing dependency form | Observed rejection |
| --- | --- | --- |
| npm10.9.8 | Registry version | Exact missing-package metadata request denied |
| npm10.9.8 | npm registry tarball URL | Rewritten local tarball path denied |
| npm10.9.8 | Other HTTPS URL | External CONNECT denied |
| npm10.9.8 | Other loopback HTTP URL | External-target GET denied |
| npm12.0.2 | Registry version | Exact missing-package metadata request denied |
| npm12.0.2 | Each of the three URL forms | No unexpected/external request; npm exits zero, then the required installed inventory rejects the missing package.json |

The npm10 request assertions compare the exact denial category/path set. An unrelated `/npm` lookup cannot satisfy them. The npm12 no-request branch checks successful npm exit, the missing package's installed-inventory failure, zero public redirects, and absence of a successful validation receipt. It is inventory enforcement evidence, **not npm12 proxy URL-denial evidence**. All eight receipts have a verified five-package/five-license baseline and no notifier request. The release agent reported the complete four-method suite passing under both npm versions; this review independently reconciled the source and retained per-case receipts without rerunning the tests.

After disabling the notifier, the next real bundle attempt made only the expected four staged package requests. Its unchanged network receipt is retained as `subsequent-real-bundle-network-only.json`. That run subsequently failed an installed-file hash check caused by npm's handling of a command shebang. Therefore clean network behavior alone is not recorded as complete bundle acceptance. A later exact-source installation result must establish the remaining file and runtime checks.

This correction concerns private npm packaging fixtures. It awards no native platform, owner-device, production-signing, publication, teammate-access, or full-release score. Historical review statements remain preserved alongside this explicit correction.
