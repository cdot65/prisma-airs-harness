//! Exercise Find through real terminal input in both AIRS-supported transcript modes.
use super::focus_palette::PtyCodex;
use super::focus_palette::write_test_config;
use anyhow::Result;
use anyhow::ensure;
use std::time::Duration;
use std::time::Instant;

#[test]
fn terminal_find_preserves_draft_in_inline_and_fullscreen_modes() -> Result<()> {
    let root = codex_utils_cargo_bin::repo_root()?;
    for fullscreen in [false, true] {
        let home = tempfile::tempdir()?;
        write_test_config(home.path(), &root)?;
        let mut terminal = if fullscreen {
            PtyCodex::start_with_env(
                &root,
                home,
                &[
                    "-c",
                    "tui.alternate_screen=\"always\"",
                    "-c",
                    "tui.fullscreen_transcript=true",
                ],
                &[],
            )?
        } else {
            PtyCodex::start(&root, home, &["--no-alt-screen"])?
        };
        terminal.wait_for_startup()?;
        terminal.wait_for_screen("gpt-5.6-terra default")?;
        terminal.write_input(b"preserve-unsent-draft")?;
        terminal.wait_for_screen("preserve-unsent-draft")?;
        // F3 is delivered by the terminal decoder, not a synthetic App event.
        terminal.write_input(b"\x1b[13~")?;
        terminal.wait_for_screen("Find:")?;
        terminal.write_input(b"absent-query")?;
        terminal.wait_for_screen("Find: absent-query")?;
        terminal.write_input(b"\x1b")?;
        let deadline = Instant::now() + Duration::from_secs(/*secs*/ 5);
        while terminal.screen_contains("Find:") && Instant::now() < deadline {
            terminal.read_output(Duration::from_millis(/*millis*/ 20))?;
            terminal.ensure_running()?;
        }
        ensure!(
            !terminal.screen_contains("Find:"),
            "Find failed to close: {}",
            terminal.screen_contents()
        );
        if !fullscreen {
            terminal.write_input(b"q")?;
        }
        terminal.wait_for_screen("preserve-unsent-draft")?;
        ensure!(
            !terminal.screen_contains("absent-query"),
            "query leaked into draft: {}",
            terminal.screen_contents()
        );
        terminal.ensure_running()?;
    }
    Ok(())
}
