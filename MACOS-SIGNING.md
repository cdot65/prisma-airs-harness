# Apple Silicon release signing

Status: signed alpha10 executable received on 2026-09-09 in private draft
release `airs-harness-alpha10-signing-intake`. The downloaded ZIP and extracted
executable match both GitHub's asset digest and the owner's checksum receipt.
The owner reports Apple notarization **Accepted** for submission
`dc835ddf-8841-49bc-a7ee-2f370dcb3457` (created 2026-09-09T08:41:42.493Z).
Direct Mac signature, Gatekeeper and installed package acceptance remain pending.

The ZIP SHA-256 is
`8db8ef853c00cb6994040a5652cf1f77717d2765ceefb04dc5429428382ce129`;
the signed executable SHA-256 is
`2099b66324eef6d1c63a6ba06da681163f60031767081833888e34626a4dee2b`.
Its uploaded signing report identifies Developer ID Application team G5QLZ5A8TA
with hardened runtime enabled. Signing credentials remain on the owner's Mac;
SSH access and private-key export are not required.

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
required verification; receipt of signed bytes does not establish release readiness.

The [alpha10 signing handoff](validation/2026-09-09/authentication/alpha10-b012-macos/SIGNING-HANDOFF.md)
now identifies the immutable Apple Silicon download and both expected hashes.
It includes commands for the owner's prepared signing identity and local
notarization profile. Native Mac acceptance results are recorded separately.
