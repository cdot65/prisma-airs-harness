# macOS15 alpha10 acceptance

Run34298115142 completed successfully using runtime/tooling source b012ba55e.
The original evidence ZIP was downloaded by immutable artifact10085134055 and
independently verified against GitHub SHA-256
e732e1ae364b5a59249a340a86c347b2e74392427500953902bd1c91b2c50b7a.

The broad native suite passed37/39 (68.880s), with the npm-managed CLI test and
Linux session-bus recovery test skipped. The installed suite passed37/38 (71.178s),
with only the Linux session-bus recovery test skipped. Actual native and installed CLI
Keychain fixtures passed, as did the raw native store fixture. Extracted
package integrity and installed native checksum verification passed. Both
identify native3393285d9230f53cfd63702881b253e56f2f864e58acd7c407266167c875c5f0.
The managed CLI5.2.0 contract passed, including20 capability helps and actual
PDF/PNG/JPEG/SVG/DOCX generation.

This original run used the legacy private npm package layout. Separate macOS26
acceptance validates the scoped bundled package. These are hosted ad-hoc
candidate checks, not owner-device testing, Developer ID signing, notarization,
public registry access or real Mac gateway/Keycloak integration. The native
fixture gateway is loopback; Linux live integration has its own evidence.
