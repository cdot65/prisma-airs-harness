//! Exercise startup probe replay and the eventual overlay policy in a real PTY.

use super::focus_palette::PtyCodex;
use super::focus_palette::write_test_config;
use anyhow::Result;
use anyhow::ensure;
use std::time::Duration;
use std::time::Instant;

const SECONDARY_QUERY: &[u8] = b"\x1b[>c";
const ENTER_ALT: &[u8] = b"\x1b[?1049h";

#[derive(Clone, Copy)]
enum Connection {
    Ssh,
    Local,
    TmuxOverSsh,
}

#[derive(Clone, Copy)]
enum Reply {
    Apple,
    Other,
    Missing,
}

fn check_terminal(
    connection: Connection,
    reply: Reply,
    args: &[&str],
    expect_alt: bool,
) -> Result<()> {
    let repo_root = codex_utils_cargo_bin::repo_root()?;
    let home = tempfile::tempdir()?;
    write_test_config(home.path(), &repo_root)?;
    let ssh = !matches!(connection, Connection::Local);
    let tmux = matches!(connection, Connection::TmuxOverSsh);
    let mut terminal = PtyCodex::start_with_env(
        &repo_root,
        home,
        args,
        &[
            (
                "SSH_CONNECTION",
                ssh.then_some("192.0.2.1 1234 192.0.2.2 22"),
            ),
            ("SSH_TTY", None),
            ("TMUX", tmux.then_some("/tmp/terminal-test,1,0")),
            ("TMUX_PANE", None),
            ("STY", None),
            ("TERM_PROGRAM", None),
        ],
    )?;
    let deadline = Instant::now() + Duration::from_secs(/*secs*/ 30);
    let mut replied = false;
    loop {
        terminal.read_output(Duration::from_millis(/*millis*/ 5))?;
        terminal.answer_startup_queries()?;
        if !replied && terminal.output_contains(SECONDARY_QUERY) {
            let secondary: &[u8] = match reply {
                Reply::Apple => b"\x1b[>1;95;0c",
                Reply::Other => b"\x1b[>64;2500;0c",
                Reply::Missing => b"",
            };
            // Interleaved typeahead survives; neither DA reply may become composer text.
            terminal.write_input(b"before-da")?;
            terminal.write_input(secondary)?;
            terminal.write_input(b"-after-da")?;
            replied = true;
        }
        if terminal.screen_contains("/model to change") && terminal.screen_contains("gpt-5.6-terra")
        {
            break;
        }
        terminal.ensure_running()?;
        ensure!(
            Instant::now() < deadline,
            "startup timed out: {}",
            terminal.screen_contents()
        );
    }
    ensure!(
        terminal.output_contains(SECONDARY_QUERY) == (ssh && !tmux),
        "wrong identity query scope"
    );
    if replied {
        terminal.wait_for_screen("before-da-after-da")?;
        ensure!(
            !terminal.screen_contains("95;0c"),
            "device attributes leaked into the composer"
        );
        // Discard the draft without issuing an inference request.
        terminal.write_input(b"\x15")?;
    }
    // Transcript overlay is the existing alternate-screen consumer in this AIRS baseline.
    terminal.write_input(b"\x14")?;
    terminal.wait_for_screen("T R A N S C R I P T")?;
    ensure!(
        terminal.output_contains(ENTER_ALT) == expect_alt,
        "wrong overlay screen mode: {}",
        terminal.screen_contents()
    );
    Ok(())
}

#[test]
fn apple_terminal_ssh_auto_keeps_native_scrollback_and_typeahead() -> Result<()> {
    check_terminal(
        Connection::Ssh,
        Reply::Apple,
        &[],
        /*expect_alt*/ false,
    )
}

#[test]
fn apple_terminal_ssh_always_override_enters_alternate_screen() -> Result<()> {
    check_terminal(
        Connection::Ssh,
        Reply::Apple,
        &["-c", "tui.alternate_screen=\"always\""],
        /*expect_alt*/ true,
    )
}

#[test]
fn apple_terminal_ssh_cli_override_wins_over_always() -> Result<()> {
    check_terminal(
        Connection::Ssh,
        Reply::Apple,
        &["--no-alt-screen", "-c", "tui.alternate_screen=\"always\""],
        /*expect_alt*/ false,
    )
}

#[test]
fn other_terminal_ssh_auto_retains_alternate_screen() -> Result<()> {
    check_terminal(Connection::Ssh, Reply::Other, &[], /*expect_alt*/ true)
}

#[test]
fn missing_secondary_reply_finishes_startup_without_claiming_apple_terminal() -> Result<()> {
    check_terminal(
        Connection::Ssh,
        Reply::Missing,
        &[],
        /*expect_alt*/ true,
    )
}

#[test]
fn local_terminal_does_not_query_remote_identity() -> Result<()> {
    check_terminal(
        Connection::Local,
        Reply::Apple,
        &[],
        /*expect_alt*/ true,
    )
}

#[test]
fn tmux_over_ssh_does_not_apply_outer_terminal_identity() -> Result<()> {
    check_terminal(
        Connection::TmuxOverSsh,
        Reply::Apple,
        &[],
        /*expect_alt*/ true,
    )
}
