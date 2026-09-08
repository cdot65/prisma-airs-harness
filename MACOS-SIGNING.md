# Apple Silicon release signing

Status: signing access verification pending. The owner has Apple Developer access
on other Macs; a browser session on those Macs does not establish a usable local
signing identity or notarization credential. Existing hosted candidate artifacts
are ad-hoc signed and must retain that distinction.

The preferred signing location is an owner-controlled Apple Silicon Mac. Obtain
its reachable SSH hostname and username, and the Apple Developer Team ID if known.
Do not request passwords, exported private keys or recovery codes through chat.

Once access is available, verify:

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
