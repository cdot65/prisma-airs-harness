# Authenticated review-registry preflight

[Run 34292142979](https://github.com/cdot65/airs-harness/actions/runs/34292142979)
passed using only repository contents/read and packages/read permissions.
The original evidence ZIP matches GitHub's SHA-256 digest independently.

At 2026-09-08 23:46:55 UTC, the repository workflow token could read the launcher,
Linux x64 and Apple Silicon scoped packages. All contained published alpha.9;
none contained `0.1.0-alpha.10`. This is a point-in-time availability check,
not a version reservation or proof of teammate access. Recheck immutable version
availability immediately before any later publication. No package or tag was
written, no native build ran, and production was unchanged.

The local npm client had no GitHub registry credential; an authenticated local
operator attempt was also denied. Those local access restrictions do not prove
the packages are absent. The CI preflight keeps 401/403/404 and redirects distinct
from an unused version and requires the known published baseline to be readable.
Three focused mocked tests passed before the real read. Their initial draft used
the wrong Python Request accessor; it was corrected to `get_method()` before CI.

Apple signing remains a separate dependency. The owner confirms Apple Developer
access on other Macs; remote signing-machine details and usable certificate/private
key and notarization configuration have not yet been verified. Browser login alone
is not signing evidence. See Apple's [Developer ID guidance](https://developer.apple.com/developer-id/)
and [notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow).
