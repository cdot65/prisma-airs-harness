use super::*;
use pretty_assertions::assert_eq;

#[test]
fn catalog_validation_is_read_only_and_rejects_oversized_or_invalid_content() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join("catalog.json");
    for (contents, valid) in [
        ("{\"models\":[]}".to_owned(), true),
        ("PRIVATE-CATALOG-CANARY".to_owned(), false),
        (" ".repeat(1024 * 1024 + 1), false),
    ] {
        std::fs::write(&path, &contents).unwrap();
        assert_eq!(valid_catalog(&path), valid);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    }
    assert!(!valid_catalog(home.path()));
    assert!(!valid_catalog(&home.path().join("missing.json")));
}

#[cfg(unix)]
#[test]
fn catalog_probe_rejects_symlinks_without_reading_the_target() {
    let home = tempfile::tempdir().unwrap();
    let target = home.path().join("target.json");
    std::fs::write(&target, "{}").unwrap();
    let link = home.path().join("catalog.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(!valid_catalog(&link));
}

#[cfg(unix)]
#[tokio::test]
async fn stalled_catalog_helper_times_out_without_waiting_for_its_exit() {
    let started = std::time::Instant::now();
    let mut command = Command::new("sleep");
    command.arg("60");
    assert_eq!(
        probe(&mut command, Duration::from_millis(40)).await,
        Outcome::TimedOut
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_uses_only_the_exit_status_and_discards_helper_output() {
    for (code, outcome) in [
        (0, Outcome::Valid),
        (2, Outcome::Invalid),
        (1, Outcome::Unavailable),
    ] {
        let mut command = Command::new("sh");
        command.args([
            "-c",
            &format!("printf PRIVATE-CATALOG-CANARY; printf PRIVATE-ERROR-CANARY >&2; exit {code}"),
        ]);
        assert_eq!(probe(&mut command, Duration::from_secs(2)).await, outcome);
    }
    let mut command = Command::new("/nonexistent-airs-catalog-helper");
    assert_eq!(
        probe(&mut command, Duration::from_secs(2)).await,
        Outcome::Unavailable
    );
}
