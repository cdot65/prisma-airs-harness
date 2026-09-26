use super::*;
use codex_keyring_store::tests::MockKeyringStore;
use pretty_assertions::assert_eq;

fn binding() -> Binding {
    let identity = Identity {
        config: IdentityConfig {
            issuer: "https://identity.example/realms/work".into(),
            client_id: "harness".into(),
            audience: "gateway".into(),
        },
        subject: "person-one".into(),
        display_name: None,
    };
    let gateway_url = "https://gateway.example/v1".to_owned();
    Binding {
        schema_version: 1,
        id: Uuid::new_v4(),
        credential_fingerprint: fingerprint(&gateway_url, &identity).unwrap(),
        gateway_url,
        source: Some(Source::Oidc { identity }),
    }
}

#[tokio::test]
async fn definitive_rejection_survives_subsequent_credential_reads() {
    let binding = binding();
    let store = MockKeyringStore::default();
    save_in(&binding, &Stored::RefreshPending, &store).unwrap();
    let error = complete_refresh(
        &binding,
        Err(codex_airs_identity::TokenExchangeError::RefreshRejected.into()),
        &store,
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::SignInRequired)
    );
    for _ in 0..2 {
        let error = load_active_from(&binding, &store).err().unwrap();
        assert_eq!(
            error.downcast_ref::<CredentialRecovery>(),
            Some(&CredentialRecovery::SignInRequired)
        );
    }
    assert!(
        !store
            .saved_value(&binding.id.to_string())
            .unwrap()
            .contains("refresh_token")
    );
}

#[tokio::test]
async fn uncertain_exchange_keeps_the_tokenless_pending_record() {
    let binding = binding();
    let store = MockKeyringStore::default();
    save_in(&binding, &Stored::RefreshPending, &store).unwrap();
    let pending = store.saved_value(&binding.id.to_string());
    let error = complete_refresh(
        &binding,
        Err(codex_airs_identity::TokenExchangeError::OutcomeUnknown.into()),
        &store,
    )
    .await
    .unwrap_err();
    assert_eq!(
        error.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::OutcomeUnknown)
    );
    assert_eq!(store.saved_value(&binding.id.to_string()), pending);
    let next = load_active_from(&binding, &store).err().unwrap();
    assert_eq!(
        next.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::OutcomeUnknown)
    );
}

#[tokio::test]
async fn returned_generation_is_saved_before_becoming_available() {
    let binding = binding();
    let store = MockKeyringStore::default();
    save_in(&binding, &Stored::RefreshPending, &store).unwrap();
    let Some(Source::Oidc { identity }) = &binding.source else {
        panic!("OIDC fixture")
    };
    let tokens = Tokens {
        identity: identity.clone(),
        access_token: "new-access".into(),
        refresh_token: "new-refresh".into(),
        expires_at: 9999999999,
        nonce: None,
    };
    assert_eq!(
        complete_refresh(&binding, Ok(tokens), &store)
            .await
            .unwrap(),
        "new-access"
    );
    let loaded = load_active_from(&binding, &store).unwrap();
    assert_eq!(
        (loaded.access_token.as_str(), loaded.refresh_token.as_str()),
        ("new-access", "new-refresh")
    );
}

#[tokio::test]
async fn policy_denied_admission_preserves_active_credential_bytes() {
    let binding = binding();
    let store = MockKeyringStore::default();
    let Some(Source::Oidc { identity }) = &binding.source else {
        panic!("OIDC fixture")
    };
    save_in(
        &binding,
        &Stored::Active {
            tokens: Tokens {
                identity: identity.clone(),
                access_token: "previous-access".into(),
                refresh_token: "previous-refresh".into(),
                expires_at: 1,
                nonce: None,
            },
        },
        &store,
    )
    .unwrap();
    let before = store.saved_value(&binding.id.to_string());
    let admission: anyhow::Result<std::future::Ready<anyhow::Result<Tokens>>> =
        Err(codex_http_client::NetworkPolicyDenied::Destination.into());
    let error = refresh_after_admission(&binding, admission, &store)
        .await
        .unwrap_err();
    assert_eq!(
        error.downcast_ref::<codex_http_client::NetworkPolicyDenied>(),
        Some(&codex_http_client::NetworkPolicyDenied::Destination)
    );
    assert_eq!(store.saved_value(&binding.id.to_string()), before);
    assert_eq!(
        load_active_from(&binding, &store).unwrap().refresh_token,
        "previous-refresh"
    );
}

#[tokio::test]
async fn policy_revoked_after_admission_never_restores_rotating_predecessor() {
    let binding = binding();
    let store = MockKeyringStore::default();
    let exchange = async {
        assert!(matches!(
            load_active_from(&binding, &store)
                .err()
                .unwrap()
                .downcast_ref::<CredentialRecovery>(),
            Some(CredentialRecovery::OutcomeUnknown)
        ));
        Err(codex_http_client::NetworkPolicyDenied::Revoked.into())
    };
    let error = refresh_after_admission(&binding, Ok(exchange), &store)
        .await
        .unwrap_err();
    assert_eq!(
        error.downcast_ref::<CredentialRecovery>(),
        Some(&CredentialRecovery::OutcomeUnknown)
    );
    assert_eq!(
        store.saved_value(&binding.id.to_string()),
        Some("{\"state\":\"refresh-pending\"}".into())
    );
}
