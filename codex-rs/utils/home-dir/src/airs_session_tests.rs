use super::*;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn publish(home: &Path, state: AuthGenerationState, nonce: &str) {
    fs::write(
        home.join(AUTH_GENERATION_FILE),
        AuthGeneration::new(state, nonce.to_owned())
            .unwrap()
            .encode(),
    )
    .unwrap();
}

#[test]
fn logout_then_login_never_resurrects_existing_guard() {
    let home = TempDir::new().unwrap();
    publish(
        home.path(),
        AuthGenerationState::Active,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let original = AirsSessionGuard::capture(home.path()).unwrap();
    let retained = original.clone();
    publish(
        home.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(original.check().is_err());
    publish(
        home.path(),
        AuthGenerationState::Active,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert!(retained.check().is_err());
    assert!(
        AirsSessionGuard::capture(home.path())
            .unwrap()
            .check()
            .is_ok()
    );
}

#[test]
fn legacy_clients_stop_on_first_epoch_and_missing_records_fail_closed() {
    let home = TempDir::new().unwrap();
    let legacy = AirsSessionGuard::capture(home.path()).unwrap();
    publish(
        home.path(),
        AuthGenerationState::Active,
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert!(legacy.check().is_err());
    let current = AirsSessionGuard::capture(home.path()).unwrap();
    fs::remove_file(home.path().join(AUTH_GENERATION_FILE)).unwrap();
    assert!(current.check().is_err());
}

#[test]
fn logout_marker_and_invalid_records_are_fail_closed() {
    let home = TempDir::new().unwrap();
    fs::write(home.path().join("logged-out"), "").unwrap();
    assert!(AirsSessionGuard::capture(home.path()).is_err());
    fs::remove_file(home.path().join("logged-out")).unwrap();
    for value in [
        "",
        "v1 active bad\n",
        "v1 active aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\n",
        "v1 active aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\u{1b}",
        &"x".repeat(129),
    ] {
        fs::write(home.path().join(AUTH_GENERATION_FILE), value).unwrap();
        assert!(read_auth_generation(home.path()).is_err());
        assert!(AirsSessionGuard::capture(home.path()).is_err());
    }
}

#[test]
fn round_trip_and_other_environment_isolation() {
    let home = TempDir::new().unwrap();
    let other = TempDir::new().unwrap();
    let guard = AirsSessionGuard::capture(home.path()).unwrap();
    publish(
        other.path(),
        AuthGenerationState::Revoked,
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert!(guard.check().is_ok());
    assert_eq!(
        read_auth_generation(other.path()).unwrap(),
        Some(
            AuthGeneration::new(
                AuthGenerationState::Revoked,
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned()
            )
            .unwrap()
        )
    );
}
