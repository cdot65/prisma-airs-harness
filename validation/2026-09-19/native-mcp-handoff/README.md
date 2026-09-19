# Final mcp.4 handoff — September 19, 2026

Independent final review passed **9.5/10**, with the limitations below retained.

The [release ledger](../native-mcp-release/README.md) records immutable published
package identities and all three fresh registry checks. This addendum records
subsequent documentation deployment, corrected Ubuntu script delivery and cleanup.

Documentation commit `4a9f641600e49dec7616218b0640505a40db01ff` passed the production
build and 22 browser tests, then deployed successfully. Five live routes match
the Pages artifact byte for byte; browser hydration, seven diagrams and all 15
vault/source provenance hashes passed. The dependency change is limited to
`image-size` 2.0.2 to 2.0.4; npm audit returned zero vulnerabilities.

The corrected Ubuntu helper is supplied separately as
`/home/cdot/prepare-airs-ubuntu-mcp4.sh` on the dedicated host. It installs mcp.4,
uses the repaired Secret Service unlock behavior from `1fd617abb4`, and now prints
its actual shell-quoted invocation for future unlocks. Final SHA256:
`82498c03f9064b5037d4a19fe03e1bdde14f640f36a6a848a85d8f06a675443a`.
Only that final rerun hint differs from the already-passed isolated keyring
regression. Syntax and remote/source byte identity passed. The mcp.4 npm archive
is immutable and contains the older helper; use the corrected supplied copy.
The owner must enter the real keyring password locally. Synthetic checks do not
prove production SSO, workspace-key authorization or ServiceNow access.

Narrow cleanup removed only verified redundant transfer archives and extracted
inputs, with exact retained local equivalents. Mac free space increased to
81.81 GiB. Generated installations, evidence, owner VMs, compilation caches and
credentials remain. The next Mac build still requires at least 105 GiB free;
this handoff does not claim that new-build readiness gate is satisfied.

Vault validation retains its existing 58 errors and 2,080 warnings; no new errors
or warnings were introduced by the final handoff. Owner content review remains
pending. Full Rust workspace historical failures remain explicitly unclaimed.

All JSON receipts are copied unchanged except VAULT-CHECK.json, which summarizes
comparison of complete validation result sets. This private ledger is not a
public site export. SHA256SUMS records the retained file bytes.
