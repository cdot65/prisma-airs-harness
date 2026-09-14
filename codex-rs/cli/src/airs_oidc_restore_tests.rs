use super::*;

#[test]
fn restore_pins_signed_subject_and_configuration_but_not_display_name() {
    let identity = Identity {
        config: IdentityConfig {
            issuer: "https://identity.example/realms/work".into(),
            client_id: "harness".into(),
            audience: "gateway".into(),
        },
        subject: "person-one".into(),
        display_name: Some("Original name".into()),
    };
    let gateway_url = "https://gateway.example/v1".to_owned();
    let binding = Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        credential_fingerprint: fingerprint(&gateway_url, &identity).unwrap(),
        gateway_url,
        source: Some(Source::Oidc {
            identity: identity.clone(),
        }),
    };
    let mut returned = identity.clone();
    returned.display_name = Some("Renamed person".into());
    ensure_restore_identity(&binding, &returned).unwrap();
    returned.subject = "person-two".into();
    assert!(ensure_restore_identity(&binding, &returned).is_err());
    for field in ["issuer", "client", "audience"] {
        let mut returned = identity.clone();
        match field {
            "issuer" => returned.config.issuer = "https://other.example".into(),
            "client" => returned.config.client_id = "other".into(),
            "audience" => returned.config.audience = "other".into(),
            _ => unreachable!(),
        }
        assert!(ensure_restore_identity(&binding, &returned).is_err());
    }
}
