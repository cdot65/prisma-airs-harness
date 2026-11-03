use super::is_desktop_helper;

#[test]
fn trusted_outer_and_inner_aliases_dispatch_before_account_setup() {
    for args in [
        vec![
            "/private/runtime/codex-linux-sandbox",
            "--desktop-command",
            "--",
            "/bin/true",
        ],
        vec![
            "codex-linux-sandbox",
            "--apply-seccomp-then-exec",
            "--desktop-command",
            "--no-proc",
            "--",
            "/bin/true",
        ],
        vec![
            "/private/runtime/airs-harness",
            "--codex-run-as-fs-helper",
            "--airs-desktop-owned-helper",
        ],
    ] {
        assert!(is_desktop_helper(args.into_iter().map(Into::into)));
    }
}

#[test]
fn user_arguments_cannot_select_early_helper_dispatch() {
    for args in [
        vec![],
        vec!["airs-harness", "--desktop-command"],
        vec!["codex-linux-sandbox", "--", "--desktop-command"],
        vec![
            "codex-linux-sandbox",
            "--sandbox-policy-cwd",
            "/project",
            "--",
            "/bin/true",
        ],
        vec!["other-codex-linux-sandbox", "--desktop-command"],
        vec!["airs-harness", "--codex-run-as-fs-helper"],
        vec!["airs-harness", "--airs-desktop-owned-helper"],
        vec![
            "airs-harness",
            "--codex-run-as-fs-helper",
            "--airs-desktop-owned-helper",
            "extra",
        ],
    ] {
        assert!(!is_desktop_helper(args.into_iter().map(Into::into)));
    }
}
