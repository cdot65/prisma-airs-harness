# Private scoped native packaging validation

Stage: `7044c43641f4d6591b9f860f6e383a923b807cfe`. Native source: `154b4f0bcef7528844e85177d7e4dce611982044`. These are unpublished private alpha.9 fixtures; production packages were not changed and no native rebuild occurred.

Two fresh installations passed on Linux using Node 22.23.2 and npm 12.0.2: the new actual scoped root/native names and the historical unscoped source154 tarballs. Both launched native SHA256 `2fcb93d5b11e5c81f2851b94a200bf81da4adba0ecb9d60b20ab8b206e81e3ad` and the required CLI 5.2.0. The scoped wrapper invocation also resolved and hashed the installed native executable. Historical archives correctly retain absent package-tooling metadata rather than inventing it.

These installs used isolated loopback registry metadata and tarballs with fresh npm configuration and caches. This unbundled stage redirects real CLI dependencies to public npm. It does not establish GitHub Packages authorization, teammate installation, bundled isolation, macOS/Windows execution, signing, or full authentication release readiness.

Completed focused checks: Node launcher/managed CLI 15/15; packager 5/5 (including four scoped/private combinations); OIDC evidence/cycle/transcript 9/9. These counts describe previously completed checks; precise execution timestamps were not retained. Required formatting passed. Actual installation receipts and their hashes are retained alongside this report.
