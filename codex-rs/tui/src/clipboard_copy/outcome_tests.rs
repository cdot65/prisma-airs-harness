//! Backend certainty and clipboard ownership, without host clipboard side effects.

use super::*;
use pretty_assertions::assert_eq;
use std::cell::RefCell;

#[test]
fn delivery_status_matches_the_backend_that_actually_succeeds() {
    for (ssh, wsl, native_ok, wsl_ok, expected, calls) in [
        (
            true,
            false,
            true,
            true,
            CopyStatus::Unconfirmed,
            vec!["terminal"],
        ),
        (
            true,
            true,
            true,
            true,
            CopyStatus::Unconfirmed,
            vec!["terminal"],
        ),
        (
            false,
            false,
            true,
            false,
            CopyStatus::Confirmed,
            vec!["native"],
        ),
        (
            false,
            true,
            false,
            true,
            CopyStatus::Confirmed,
            vec!["native", "wsl"],
        ),
        (
            false,
            true,
            false,
            false,
            CopyStatus::Unconfirmed,
            vec!["native", "wsl", "terminal"],
        ),
        (
            false,
            false,
            false,
            false,
            CopyStatus::Unconfirmed,
            vec!["native", "terminal"],
        ),
    ] {
        let observed = RefCell::new(Vec::new());
        let outcome = copy_to_clipboard_with(
            "private selected text",
            CopyFormat::PlainText,
            CopyEnvironment {
                ssh_session: ssh,
                wsl_session: wsl,
                tmux_session: false,
            },
            |_| panic!("not in tmux"),
            |_| {
                observed.borrow_mut().push("terminal");
                Ok(())
            },
            |_, _| {
                observed.borrow_mut().push("native");
                if native_ok {
                    Ok(None)
                } else {
                    Err("native unavailable".into())
                }
            },
            |_| {
                observed.borrow_mut().push("wsl");
                if wsl_ok {
                    Ok(())
                } else {
                    Err("wsl unavailable".into())
                }
            },
        )
        .unwrap_or_else(|error| panic!("{error}"));
        let mut lease = Some(ClipboardLease::test());
        assert_eq!(outcome.store(&mut lease), expected);
        assert!(lease.is_some(), "no replacement lease was supplied");
        assert_eq!(observed.into_inner(), calls);
    }
}

#[test]
fn empty_copy_does_not_touch_any_backend_or_existing_owner() {
    for ssh_session in [false, true] {
        let result = copy_to_clipboard_with(
            "",
            CopyFormat::Markdown,
            CopyEnvironment {
                ssh_session,
                wsl_session: true,
                tmux_session: true,
            },
            |_| panic!("empty tmux copy"),
            |_| panic!("empty terminal copy"),
            |_, _| panic!("empty native copy"),
            |_| panic!("empty WSL copy"),
        );
        assert!(matches!(result, Err(error) if error.contains("empty")));
    }
}

#[test]
fn confirmed_native_copy_keeps_its_owner_through_later_terminal_requests() {
    let mut lease = None;
    assert_eq!(
        CopyOutcome::Copied(Some(ClipboardLease::test())).store(&mut lease),
        CopyStatus::Confirmed
    );
    assert!(lease.is_some());
    assert_eq!(
        CopyOutcome::Requested.store(&mut lease),
        CopyStatus::Unconfirmed
    );
    assert!(lease.is_some());
    assert_eq!(
        CopyOutcome::Copied(None).store(&mut lease),
        CopyStatus::Confirmed
    );
    assert!(lease.is_some());
}
