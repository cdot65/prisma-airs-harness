use super::*;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use pretty_assertions::assert_eq;
use std::sync::mpsc;
use tempfile::TempDir;

#[test]
fn verified_restore_preserves_the_active_session_epoch() {
    let home = TempDir::new().unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    let client = AirsSessionGuard::capture(home.path()).unwrap();
    let before = read_auth_generation(home.path()).unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .complete_restore()
        .unwrap();
    assert_eq!(read_auth_generation(home.path()).unwrap(), before);
    client.check().unwrap();
}

#[test]
fn restore_cannot_reactivate_a_logged_out_or_replaced_session() {
    let home = TempDir::new().unwrap();
    assert!(
        LoginAttempt::begin(home.path())
            .unwrap()
            .complete_restore()
            .is_err()
    );
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    let attempt = LoginAttempt::begin(home.path()).unwrap();
    revoke(home.path()).unwrap();
    assert!(attempt.complete_restore().is_err());
    assert!(
        LoginAttempt::begin(home.path())
            .unwrap()
            .complete_restore()
            .is_err()
    );
    let newer = TempDir::new().unwrap();
    LoginAttempt::begin(newer.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    let attempt = LoginAttempt::begin(newer.path()).unwrap();
    LoginAttempt::begin(newer.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    assert!(attempt.complete_restore().is_err());
}

#[test]
fn logout_rejects_an_in_progress_login_before_it_installs() {
    let home = TempDir::new().unwrap();
    let attempt = LoginAttempt::begin(home.path()).unwrap();
    revoke(home.path()).unwrap();
    let result = attempt.commit(|| panic!("obsolete login must not install"));
    assert!(result.unwrap_err().to_string().contains("cancelled"));
    assert!(home.path().join("logged-out").exists());
    assert_eq!(
        read_auth_generation(home.path()).unwrap().unwrap().state,
        AuthGenerationState::Revoked
    );
}

#[test]
fn relogin_activates_a_new_epoch_without_resurrecting_old_clients() {
    let home = TempDir::new().unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    let old_client = AirsSessionGuard::capture(home.path()).unwrap();
    revoke(home.path()).unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| {
            std::fs::remove_file(home.path().join("logged-out"))?;
            Ok(())
        })
        .unwrap();
    assert!(old_client.check().is_err());
    AirsSessionGuard::capture(home.path())
        .unwrap()
        .check()
        .unwrap();
}

#[test]
fn failed_install_leaves_the_session_revoked() {
    let home = TempDir::new().unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    let client = AirsSessionGuard::capture(home.path()).unwrap();
    let attempt = LoginAttempt::begin(home.path()).unwrap();
    assert!(
        attempt
            .commit(|| anyhow::bail!("injected filesystem failure"))
            .is_err()
    );
    assert!(client.check().is_err());
    assert_eq!(
        read_auth_generation(home.path()).unwrap().unwrap().state,
        AuthGenerationState::Revoked
    );
}

#[test]
fn revocation_does_not_wait_for_a_refresh_configuration_lock() {
    let home = TempDir::new().unwrap();
    let path = home.path().to_owned();
    let config_lock = airs_environment::lock(&path).unwrap();
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || sender.send(revoke(&path)).unwrap());
    let result = receiver.recv_timeout(Duration::from_secs(2));
    drop(config_lock);
    worker.join().unwrap();
    result
        .expect("logout must not wait for configuration lock")
        .unwrap();
    assert!(AirsSessionGuard::capture(home.path()).is_err());
}

#[test]
fn short_state_lock_has_a_bounded_wait() {
    let home = TempDir::new().unwrap();
    let path = home.path().to_owned();
    let held = state_lock(&path).unwrap();
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender
            .send(LoginAttempt::begin(&path).err().map(|e| e.to_string()))
            .unwrap();
    });
    let result = receiver.recv_timeout(Duration::from_secs(3));
    drop(held);
    worker.join().unwrap();
    assert_eq!(
        result.unwrap().unwrap(),
        "Authentication state is busy; retry the command"
    );
}

#[test]
fn a_second_completed_login_cancels_the_first_attempt() {
    let home = TempDir::new().unwrap();
    let first = LoginAttempt::begin(home.path()).unwrap();
    LoginAttempt::begin(home.path())
        .unwrap()
        .commit(|| Ok(()))
        .unwrap();
    assert!(
        first
            .commit(|| panic!("older login must not install"))
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn state_lock_does_not_follow_a_symlink() {
    let home = TempDir::new().unwrap();
    let other = home.path().join("other");
    std::fs::write(&other, b"unchanged").unwrap();
    std::os::unix::fs::symlink(&other, home.path().join(".auth-generation.lock")).unwrap();
    assert!(revoke(home.path()).is_err());
    assert_eq!(std::fs::read(other).unwrap(), b"unchanged");
}
