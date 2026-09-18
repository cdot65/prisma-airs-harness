//! Interactive, synthetic preview. It never creates environments or authenticates.
use codex_tui::AirsOnboarding;
use codex_tui::OnboardingContext;
use codex_tui::OnboardingInput;
use codex_tui::OnboardingMenuItem;
use codex_tui::OnboardingOptions;
use codex_tui::OnboardingProgress;
use codex_tui::OnboardingResult;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let screen = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map_or("welcome", String::as_str);
    let mut ui = AirsOnboarding::open(
        OnboardingContext {
            environment: "work".into(),
            gateway: Some("https://gateway.example.com/v1".into()),
        },
        OnboardingOptions {
            animations: !args.iter().any(|arg| arg == "--static"),
            color: !args.iter().any(|arg| arg == "--no-color"),
        },
    )?;
    let result = match screen {
        "input" => ui
            .input(OnboardingInput {
                title: "Connect your environment".into(),
                label: "AI Gateway URL".into(),
                help: "Use the inference URL supplied by your administrator.".into(),
                value: String::new(),
                error: None,
            })?
            .map_selected(|value| format!("public-input:{value}")),
        "waiting" => {
            let (_sender, receiver) = tokio::sync::watch::channel(OnboardingProgress {
                title: "Waiting for company sign-in".into(),
                detail: "Preview only. No browser or authorization request was started.".into(),
                link: Some("https://sso.example.com/authorize?client_id=harness-native".into()),
            });
            ui.wait(
                tokio::time::sleep(Duration::from_secs(/*secs*/ 120)),
                receiver,
            )
            .await?
            .map_selected(|()| "preview-finished".to_owned())
        }
        "progress" => {
            let (sender, receiver) = tokio::sync::watch::channel(OnboardingProgress {
                title: "Checking connection".into(),
                detail: "Synthetic preview operation.".into(),
                link: None,
            });
            let operation = async move {
                tokio::time::sleep(Duration::from_millis(/*millis*/ 400)).await;
                sender.send_replace(OnboardingProgress {
                    title: "Saving sign-in".into(),
                    detail: "Preview only. Nothing is saved.".into(),
                    link: None,
                });
                tokio::time::sleep(Duration::from_millis(/*millis*/ 600)).await;
                "preview-operation-completed".to_owned()
            };
            ui.wait(operation, receiver).await?
        }
        "recovery" => ui
            .menu(
                "Sign-in needs your attention",
                &[
                    OnboardingMenuItem {
                        label: "Try again".into(),
                        detail: "Unlock your OS credential store, then retry".into(),
                    },
                    OnboardingMenuItem {
                        label: "Back to sign-in options".into(),
                        detail: "Your existing environment is preserved".into(),
                    },
                ],
            )?
            .map_selected(|index| format!("recovery-action:{index}")),
        _ => ui
            .menu(
                "Sign in to continue",
                &[
                    OnboardingMenuItem {
                        label: "Sign in with company SSO".into(),
                        detail: "Continue securely in your browser".into(),
                    },
                    OnboardingMenuItem {
                        label: "Use a workspace API key".into(),
                        detail: "Enter a key from your administrator".into(),
                    },
                    OnboardingMenuItem {
                        label: "Choose another environment".into(),
                        detail: "Keep each workspace and identity separate".into(),
                    },
                ],
            )?
            .map_selected(|index| format!("menu-action:{index}")),
    };
    drop(ui);
    println!("{result:?}");
    Ok(())
}

/// Keep synthetic preview result conversion outside the public onboarding API.
trait PreviewResult<T> {
    fn map_selected<U>(self, map: impl FnOnce(T) -> U) -> OnboardingResult<U>;
}

impl<T> PreviewResult<T> for OnboardingResult<T> {
    fn map_selected<U>(self, map: impl FnOnce(T) -> U) -> OnboardingResult<U> {
        match self {
            OnboardingResult::Selected(value) => OnboardingResult::Selected(map(value)),
            OnboardingResult::Cancelled => OnboardingResult::Cancelled,
        }
    }
}
