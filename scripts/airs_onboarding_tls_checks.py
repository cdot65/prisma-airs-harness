"""Negative TLS acceptance against the local onboarding identity fixture."""

from validate_airs_onboarding_preview import Preview


def check_tls_rejection(binary, environment, home, directory, fixture):
    """Require issuer TLS to fail closed before authorization or credential writes."""
    invalid_bundle = directory / "invalid-ca.pem"
    invalid_bundle.write_text("This is not a certificate.\n")
    checks = []
    for label, bundle in [
        ("untrusted issuer certificate", None),
        ("invalid custom CA bundle", str(invalid_bundle)),
    ]:
        env = dict(environment)
        env.pop("SSL_CERT_FILE", None)
        env.pop("CODEX_CA_CERTIFICATE", None)
        if bundle:
            env["SSL_CERT_FILE"] = bundle
        before = len(fixture.requests)
        terminal = Preview(
            binary, arguments=["login"], environment=env, directory=directory
        )
        try:
            terminal.expect("Sign in to continue")
            terminal.send(b"1")
            terminal.expect("Continue with saved settings")
            terminal.send(b"1")
            terminal.expect("How would you like to sign in?")
            terminal.send(b"2")
            terminal.expect("Sign-in needs your attention")
            if bundle is None:
                terminal.expect("issuer discovery unavailable")
            else:
                terminal.expect("SSL_CERT_FILE")
            terminal.send(b"\x1b")
            terminal.finish("", status=1)
            assert len(fixture.requests) == before, (
                "Rejected TLS configuration reached an identity HTTP endpoint"
            )
            assert not (home / "credential-binding.json").exists()
            checks.append(f"{label} fails closed before authorization or persistence")
        finally:
            terminal.close()
    return checks
