use super::RefreshCredentialLock;
use anyhow::Result;
use pretty_assertions::assert_eq;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn shared_and_environment_home_can_resolve_to_the_same_directory() -> Result<()> {
    let home = tempdir()?;
    let equivalent = home.path().join(".");
    let _guard = RefreshCredentialLock::acquire_for_homes(
        home.path(),
        &equivalent,
        "entry",
        Duration::from_millis(100),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn acquisition_times_out_without_stealing() -> Result<()> {
    let codex_home = tempdir()?;
    let store_key = "test-store-key";
    let held_lock = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;

    let error = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 50),
    )
    .await
    .err()
    .expect("contending lock acquisition should time out");
    assert!(
        error
            .to_string()
            .contains("timed out after 50ms waiting for OAuth refresh lock"),
        "unexpected error: {error:#}"
    );

    drop(held_lock);
    let _reacquired = RefreshCredentialLock::acquire_in(
        codex_home.path(),
        store_key,
        Duration::from_millis(/*millis*/ 100),
    )
    .await?;
    Ok(())
}

#[tokio::test]
async fn separate_environment_homes_share_the_native_credential_lock() -> Result<()> {
    let shared = tempdir()?;
    let first = tempdir()?;
    let second = tempdir()?;
    let held = RefreshCredentialLock::acquire_for_homes(
        shared.path(),
        first.path(),
        "same-native-entry",
        Duration::from_secs(1),
    )
    .await?;
    assert!(
        RefreshCredentialLock::acquire_for_homes(
            shared.path(),
            second.path(),
            "same-native-entry",
            Duration::from_millis(50)
        )
        .await
        .is_err()
    );
    let unrelated = RefreshCredentialLock::acquire_for_homes(
        shared.path(),
        second.path(),
        "different-native-entry",
        Duration::from_secs(1),
    )
    .await?;
    drop(held);
    let _next = RefreshCredentialLock::acquire_for_homes(
        shared.path(),
        second.path(),
        "same-native-entry",
        Duration::from_secs(1),
    )
    .await?;
    drop(unrelated);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn refuses_a_symlink_lock_without_modifying_its_target() -> Result<()> {
    use sha2::Digest;
    use sha2::Sha256;
    let home = tempdir()?;
    let target = home.path().join("untouched");
    std::fs::write(&target, "unchanged")?;
    let locks = home.path().join(super::REFRESH_LOCK_DIR);
    std::fs::create_dir(&locks)?;
    let path = locks.join(format!("{:x}.lock", Sha256::digest(b"entry")));
    std::os::unix::fs::symlink(&target, path)?;
    assert!(
        RefreshCredentialLock::acquire_in(home.path(), "entry", Duration::from_millis(50))
            .await
            .is_err()
    );
    assert_eq!(std::fs::read_to_string(target)?, "unchanged");
    Ok(())
}
