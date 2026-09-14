use super::*;
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::KeyringStore;
use pretty_assertions::assert_eq;
use std::ops::RangeInclusive;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug)]
struct InterruptedStore {
    inner: MockKeyringStore,
    writes: Arc<AtomicUsize>,
    fail: RangeInclusive<usize>,
}

impl KeyringStore for InterruptedStore {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        self.inner.load(service, account)
    }

    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        let attempt = self.writes.fetch_add(1, Ordering::SeqCst) + 1;
        if self.fail.contains(&attempt) {
            return Err(CredentialStoreError::new(KeyringError::Invalid(
                "fixture".into(),
                "save".into(),
            )));
        }
        self.inner.save(service, account, value)
    }

    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        self.inner.delete(service, account)
    }
}

#[tokio::test]
async fn failed_intent_never_dispatches_refresh() -> Result<()> {
    let (_env, server, initial) = test_context().await?;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let store = ResolvedOAuthCredentialStore::Keyring(AuthKeyringBackendKind::Direct);
    let keyring = InterruptedStore {
        inner: MockKeyringStore::default(),
        writes: Arc::new(AtomicUsize::new(0)),
        fail: 1..=1,
    };
    store.save(&keyring.inner, &initial.server_name, &initial)?;
    let manager = Arc::new(TokioMutex::new(authorization_manager_for(&initial).await?));
    let persistor = OAuthPersistor::new(
        initial.server_name.clone(),
        initial.url.clone(),
        manager,
        store,
        Some(initial.clone()),
    );
    assert!(
        persistor
            .refresh_if_needed_in(&keyring, Duration::from_secs(1))
            .await
            .is_err()
    );
    assert_tokens_match_without_expiry(
        &store
            .load(&keyring, &initial.server_name, &initial.url)?
            .unwrap(),
        &initial,
    );
    server.verify().await;
    Ok(())
}

#[tokio::test]
async fn returned_generation_is_retried_without_repeating_the_exchange() -> Result<()> {
    persistence_failure(2..=3).await?;
    persistence_failure(2..=4).await
}

async fn persistence_failure(fail: RangeInclusive<usize>) -> Result<()> {
    let (_env, server, initial) = test_context().await?;
    Mock::given(method("POST")).and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "replacement", "refresh_token": "rotated", "token_type": "Bearer", "expires_in": 3600,
        }))).expect(1).mount(&server).await;
    let succeeds = *fail.end() == 3;
    let store = ResolvedOAuthCredentialStore::Keyring(AuthKeyringBackendKind::Direct);
    let keyring = InterruptedStore {
        inner: MockKeyringStore::default(),
        writes: Arc::new(AtomicUsize::new(0)),
        fail,
    };
    store.save(&keyring.inner, &initial.server_name, &initial)?;
    let manager = Arc::new(TokioMutex::new(authorization_manager_for(&initial).await?));
    let persistor = OAuthPersistor::new(
        initial.server_name.clone(),
        initial.url.clone(),
        manager,
        store,
        Some(initial.clone()),
    );
    assert_eq!(
        persistor
            .refresh_if_needed_in(&keyring, Duration::from_secs(1))
            .await
            .is_ok(),
        succeeds
    );
    assert_eq!(keyring.writes.load(Ordering::SeqCst), 4);
    let stored = store
        .load(&keyring, &initial.server_name, &initial.url)?
        .unwrap();
    if succeeds {
        assert_eq!(
            stored.token_response.0.access_token().secret(),
            "replacement"
        );
        assert_eq!(
            stored.token_response.0.refresh_token().unwrap().secret(),
            "rotated"
        );
    } else {
        assert!(!stored.has_refresh_token());
        assert!(!stored.access_token_is_usable_without_refresh());
        assert!(is_authentication_required_error(
            &persistor
                .refresh_if_needed_in(&keyring, Duration::from_secs(1))
                .await
                .unwrap_err()
        ));
    }
    server.verify().await;
    Ok(())
}
