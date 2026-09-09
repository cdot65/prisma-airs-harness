# Apple Silicon release signing

Status: owner-operated signing host prepared on 2026-09-09. The owner supplied
Terminal output from macOS 26.6.2 arm64 showing a valid Developer ID Application
identity for team G5QLZ5A8TA and successfully validated notarization credentials
stored in the local Keychain profile `prisma-airs-harness-notary`. This is
owner-reported evidence; no final artifact has been signed or notarized yet.

The owner will run the signing workflow locally. SSH access and private-key
export are not required for this handoff. Supply an immutable Apple Silicon
artifact and checksum, and retain the resulting signed bytes, signature details,
notarization submission ID and result before packaging acceptance.

Before declaring the release signed, verify:


1. Apple Silicon architecture and installed Apple command-line signing tools.
2. A valid **Developer ID Application** identity with its private key available
   to the signing process in that Mac's Keychain, associated with the intended team.
3. Usable notarization authentication, preferably through a named local Keychain
   profile. If setup requires interactive authorization, the owner performs that
   step locally; the signing key does not need to leave the Mac.
4. Final frozen-source executable hashes before signing; hardened-runtime signing
   and Apple notarization results for the selected distribution format; final
   extracted signature and byte verification before packaging acceptance.
5. Installed secure-login, restart/resume, upgrade, gateway and MCP checks on the
   exact signed package. Hosted ad-hoc acceptance does not replace these results.

Do not rebuild solely to move signing between machines: retain the verified
compiled artifact and record signing as a separate transformation. A new product
version or runtime change still requires compilation and affected acceptance.
The alpha10 version preparation is not a publication or a reserved version.

Apple's [Developer ID guidance](https://developer.apple.com/developer-id/) describes
distribution outside the Mac App Store. Its [notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow)
documents `notarytool` and Keychain credential profiles. This checklist records
required verification; none of the pending checks is asserted as completed.
