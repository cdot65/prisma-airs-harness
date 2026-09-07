use super::*;

#[test]
fn callback_binds_host_state_issuer_and_unique_fields() {
    let redirect = "http://127.0.0.1:12345/callback";
    let state = CsrfToken::new("expected-state".into());
    let query = "state=expected-state&iss=https%3A%2F%2Fissuer.example%2Frealm&code=opaque-code";
    let request =
        |query: &str, host: &str| format!("GET /callback?{query} HTTP/1.1\r\nHost: {host}\r\n\r\n");
    assert_eq!(
        parse_callback(
            request(query, "127.0.0.1:12345").as_bytes(),
            redirect,
            &state,
            "https://issuer.example/realm"
        )
        .unwrap(),
        Some("opaque-code".into())
    );
    for changed in [
        query.replace("expected-state", "wrong-state"),
        query.replace("issuer.example", "attacker.example"),
        format!("{query}&state=expected-state"),
        format!("{query}&code=another-code"),
    ] {
        assert!(
            parse_callback(
                request(&changed, "127.0.0.1:12345").as_bytes(),
                redirect,
                &state,
                "https://issuer.example/realm"
            )
            .is_err()
        );
    }
    assert!(
        parse_callback(
            request(query, "attacker.example:12345").as_bytes(),
            redirect,
            &state,
            "https://issuer.example/realm"
        )
        .is_err()
    );
}
