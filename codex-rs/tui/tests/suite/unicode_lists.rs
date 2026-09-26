//! Exercise rendering preferences through a real terminal and streamed response.

use super::focus_palette::PtyCodex;
use super::focus_palette::write_test_config;
use anyhow::Result;
use anyhow::ensure;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use wiremock::matchers::body_string_contains;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unicode_lists_render_in_the_terminal() -> Result<()> {
    exercise("", &["☐ Pending", "☑ Done", "• Ordinary"]).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disabled_lists_keep_textual_markers_in_the_terminal() -> Result<()> {
    exercise(
        "[tui.rendering]\nlists = false\n",
        &["- [ ] Pending", "- [x] Done", "- Ordinary"],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_mode_keeps_source_markers_in_the_terminal() -> Result<()> {
    exercise(
        "[tui]\nraw_output_mode = true\n",
        &["- [ ] Pending", "- [x] **Done**", "- Ordinary"],
    )
    .await
}

async fn exercise(preference: &str, expected: &[&str]) -> Result<()> {
    let workspace = tempfile::tempdir()?;
    let workspace_path = workspace.path().canonicalize()?;
    let home = tempfile::tempdir()?;
    write_test_config(home.path(), &workspace_path)?;
    let server = responses::start_mock_server().await;
    let _title = responses::mount_sse_once_match(
        &server,
        body_string_contains(r#"\"thread_source\":\"system\""#),
        responses::sse(vec![
            responses::ev_assistant_message("title", r#"{"title":"List rendering"}"#),
            responses::ev_completed("title"),
        ]),
    )
    .await;
    let source = "- [ ] Pending\n- [x] **Done**\n- Ordinary\n\nLists ready.";
    let response = responses::mount_sse_once_match(
        &server,
        body_string_contains(r#"\"thread_source\":\"user\""#),
        responses::sse(vec![
            responses::ev_message_item_added("lists", ""),
            responses::ev_output_text_delta(source),
            responses::ev_assistant_message("lists", source),
            responses::ev_completed("lists"),
        ]),
    )
    .await;
    let path = home.path().join("config.toml");
    let base = std::fs::read_to_string(&path)?;
    let url = server.uri();
    std::fs::write(
        path,
        format!(
            "{base}\n{preference}\n[model_providers.test]\nname = \"Mock\"\n\
             base_url = \"{url}/v1\"\nwire_api = \"responses\"\n\
             requires_openai_auth = false\nsupports_websockets = false\n"
        ),
    )?;
    let mut terminal = PtyCodex::start(
        &workspace_path,
        home,
        &["-c", "model_provider=\"test\"", "Show the fixture list"],
    )?;
    terminal.wait_for_startup()?;
    terminal.wait_for_screen("Lists ready.")?;
    let screen = terminal.screen_contents();
    for &line in expected {
        ensure!(screen.contains(line), "missing {line:?}; screen:\n{screen}");
    }
    assert_eq!(response.requests().len(), 1, "expected one user request");
    Ok(())
}
