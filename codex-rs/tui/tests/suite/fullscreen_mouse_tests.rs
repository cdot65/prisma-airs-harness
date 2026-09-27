//! Exercise actual SGR mouse decoding and terminal-only clipboard delivery over a PTY.
use super::*;
use pretty_assertions::assert_eq;

#[test]
fn fullscreen_sgr_right_click_preserves_unconfirmed_selection_and_draft() -> Result<()> {
    let root = codex_utils_cargo_bin::repo_root()?;
    let home = tempfile::tempdir()?;
    write_test_config(home.path(), &root)?;
    let path = home.path().join("config.toml");
    let config = std::fs::read_to_string(&path)?;
    std::fs::write(
        &path,
        format!(
            "tui.fullscreen_transcript = true\ntui.alternate_screen = \"always\"\ntui.disable_paste_burst = true\ntui.status_line = [\"thread-id\"]\n{config}"
        ),
    )?;
    // SSH clipboard requests are emitted to the terminal, never to this host's clipboard.
    let mut terminal = PtyCodex::start_with_env(
        &root,
        home,
        &[],
        &[("SSH_CONNECTION", Some("127.0.0.1 1234 127.0.0.1 22"))],
    )?;
    terminal.wait_for_startup()?;
    let deadline = Instant::now() + STARTUP_TIMEOUT;
    loop {
        terminal.read_output(Duration::from_millis(/*millis*/ 50))?;
        terminal.answer_startup_queries()?;
        if terminal
            .screen_contents()
            .split_whitespace()
            .any(|word| uuid::Uuid::parse_str(word).is_ok())
        {
            break;
        }
        ensure!(
            Instant::now() < deadline,
            "thread not ready: {}",
            terminal.screen_contents()
        );
        terminal.ensure_running()?;
    }
    assert!(terminal.parser.screen().alternate_screen());
    terminal.write_input(b"hello world")?;
    terminal.wait_for_screen("hello world")?;
    // A PTY read can end after text painting but before cursor restoration. Hit-test the
    // painted cells instead of assuming the terminal cursor already marks the draft end.
    let screen = terminal.parser.screen();
    let (rows, columns) = screen.size();
    let (row, start) = (0..rows)
        .find_map(|row| {
            (0..columns.saturating_sub(10)).find_map(|column| {
                "hello world"
                    .chars()
                    .enumerate()
                    .all(|(offset, character)| {
                        screen
                            .cell(row, column + offset as u16)
                            .is_some_and(|cell| {
                                let content = cell.contents();
                                content == character.to_string()
                                    || (character == ' ' && content.is_empty())
                            })
                    })
                    .then_some((row, column))
            })
        })
        .context("locate the painted draft for SGR hit testing")?;
    for (button, column, suffix) in [(0, start, 'M'), (32, start + 5, 'M'), (0, start + 5, 'm')] {
        terminal
            .write_input(format!("\x1b[<{button};{};{}{suffix}", column + 1, row + 1).as_bytes())?;
    }
    let deadline = Instant::now() + FOCUS_INPUT_TIMEOUT;
    while !terminal
        .parser
        .screen()
        .cell(row, start)
        .is_some_and(vt100::Cell::inverse)
    {
        terminal.read_output(Duration::from_millis(/*millis*/ 20))?;
        ensure!(
            Instant::now() < deadline,
            "SGR drag did not select the draft"
        );
    }
    for _ in 0..2 {
        let before = terminal.output.len();
        terminal.write_input(format!("\x1b[<2;{};{}M", start + 3, row + 1).as_bytes())?;
        let deadline = Instant::now() + FOCUS_INPUT_TIMEOUT;
        while !contains_bytes(&terminal.output[before..], b"\x1b]52;c;aGVsbG8=") {
            terminal.read_output(Duration::from_millis(/*millis*/ 20))?;
            ensure!(
                Instant::now() < deadline,
                "right click did not request exact selected clipboard text"
            );
        }
        assert!(terminal.screen_contents().contains("hello world"));
        assert!(
            terminal
                .parser
                .screen()
                .cell(row, start)
                .is_some_and(vt100::Cell::inverse)
        );
    }
    // An unconfirmed copy retains the editor selection; replacement changes only selected text.
    terminal.write_input(b"X")?;
    terminal.wait_for_screen("X world")?;
    assert_eq!(terminal.parser.screen().alternate_screen(), true);
    Ok(())
}
