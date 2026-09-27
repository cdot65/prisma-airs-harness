//! Verify math/list settings through the actual terminal and streaming client.

use super::focus_palette::PtyCodex;
use super::focus_palette::write_test_config;
use anyhow::Result;
use anyhow::ensure;
use core_test_support::responses;
use pretty_assertions::assert_eq;
use wiremock::matchers::body_string_contains;

const SOURCE: &str = "- [x] **Ready**\n\nEquation: $\\alpha^2 + \\beta_{10}$.\n\n\\[\\begin{aligned}x&=1\\\\y&=2\\end{aligned}\\]\n\nMath ready.\n";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rich_math_and_lists_render_together() -> Result<()> {
    exercise("", &["☑ Ready", "Equation: α² + β₁₀.", "x =1", "y =2"]).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disabled_math_retains_tex_and_rich_lists() -> Result<()> {
    exercise(
        "[tui.rendering]\nmath = false\n",
        &[
            "☑ Ready",
            r"Equation: $\alpha^2 + \beta_{10}$.",
            r"\[\begin{aligned}x&=1\\y&=2\end{aligned}\]",
        ],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rich_math_is_independent_of_textual_lists() -> Result<()> {
    exercise(
        "[tui.rendering]\nlists = false\n",
        &["- [x] Ready", "Equation: α² + β₁₀.", "x =1"],
    )
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_mode_preserves_math_and_list_source() -> Result<()> {
    exercise(
        "[tui]\nraw_output_mode = true\n",
        &[
            "- [x] **Ready**",
            r"Equation: $\alpha^2 + \beta_{10}$.",
            r"\[\begin{aligned}x&=1\\y&=2\end{aligned}\]",
        ],
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
            responses::ev_assistant_message("title", r#"{"title":"Math rendering"}"#),
            responses::ev_completed("title"),
        ]),
    )
    .await;
    let response = responses::mount_sse_once_match(
        &server,
        body_string_contains(r#"\"thread_source\":\"user\""#),
        responses::sse(vec![
            responses::ev_message_item_added("math", ""),
            responses::ev_output_text_delta(SOURCE),
            responses::ev_assistant_message("math", SOURCE),
            responses::ev_completed("math"),
        ]),
    )
    .await;
    let path = home.path().join("config.toml");
    let base = std::fs::read_to_string(&path)?;
    let url = server.uri();
    std::fs::write(
        path,
        format!(
            "{base}\n{preference}\n[model_providers.test]\nname = \"Mock\"\nbase_url = \"{url}/v1\"\nwire_api = \"responses\"\nrequires_openai_auth = false\nsupports_websockets = false\n"
        ),
    )?;
    let mut terminal = PtyCodex::start(
        &workspace_path,
        home,
        &["-c", "model_provider=\"test\"", "Show the fixture equation"],
    )?;
    terminal.wait_for_startup()?;
    terminal.wait_for_screen("Math ready.")?;
    // Partial previews can precede the stable commit queue.
    for &line in expected {
        terminal.wait_for_screen(line)?;
    }
    let screen = terminal.screen_contents();
    for &line in expected {
        ensure!(screen.contains(line), "missing {line:?}; screen:\n{screen}");
    }
    assert_eq!(response.requests().len(), 1, "expected one user request");
    Ok(())
}
