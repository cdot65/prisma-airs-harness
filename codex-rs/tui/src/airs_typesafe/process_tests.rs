#[cfg(unix)]
mod unix {
    use super::super::*;
    use std::os::unix::fs::PermissionsExt;

    fn executable(root: &Path, source: &str) -> std::path::PathBuf {
        let path = root.join("fixture");
        std::fs::write(&path, format!("#!/bin/sh\n{source}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    #[tokio::test]
    async fn set_binds_home_and_passes_key_only_over_stdin() {
        let temp = tempfile::tempdir().unwrap();
        let exe = executable(
            temp.path(),
            "test \"$#\" = 4 && test \"$1 $2 $3 $4\" = 'env typesafe set --stdin' || exit 1\ntest -d \"$AIRS_HARNESS_HOME\" || exit 1\nkey=$(cat)\ntest \"$key\" = PRIVATE-KEY || exit 1\nprintf '%s' \"$key\"",
        );
        let result = run_executable(
            &exe,
            temp.path(),
            Operation::Set,
            Some("PRIVATE-KEY".into()),
        )
        .await
        .unwrap();
        assert!(result.contains("API key saved"));
        assert!(!result.contains("PRIVATE-KEY"));
    }

    #[tokio::test]
    async fn failure_output_is_not_exposed_and_oversized_status_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let exe = executable(
            temp.path(),
            "echo PRIVATE-KEY >&2; echo PRIVATE-KEY; exit 1",
        );
        let error = run_executable(&exe, temp.path(), Operation::Clear, None)
            .await
            .unwrap_err();
        assert!(!error.contains("PRIVATE-KEY"));
        let exe = executable(temp.path(), "head -c 8193 /dev/zero");
        let error = run_executable(&exe, temp.path(), Operation::Status, None)
            .await
            .unwrap_err();
        assert!(error.contains("size limit"));
    }
}
