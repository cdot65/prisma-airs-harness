//! Operator protocol fixture: stdout carries login instructions and status only.
use codex_airs_identity::IdentityConfig;
use codex_airs_identity::Provider;
use serde_json::json;
use std::io::Write;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let provider = Provider::discover(IdentityConfig {
        resource: None,
        scopes: Vec::new(),
        issuer: "https://auth.dev.cdot.io/realms/truffles".into(),
        client_id: "airs-terminal-pilot".into(),
        audience: "airs-terminal-inference".into(),
    })
    .await?;
    let tokens = if std::env::args().any(|a| a == "--device") {
        let login = provider.device_login().await?;
        println!(
            "{}",
            json!({"verification_uri":login.verification_uri().as_str(), "user_code":login.user_code()})
        );
        std::io::stdout().flush()?;
        login.complete(&provider).await?
    } else {
        let login = provider.browser_login().await?;
        println!(
            "{}",
            json!({"authorization_url":login.authorization_url().as_str()})
        );
        std::io::stdout().flush()?;
        login.complete(&provider).await?
    };
    let first = provider.refresh(&tokens).await?;
    let second = provider.refresh(&first).await?;
    anyhow::ensure!(
        tokens.identity.subject == second.identity.subject,
        "subject changed"
    );
    provider.revoke(&second).await?;
    anyhow::ensure!(
        provider.refresh(&second).await.is_err(),
        "revoked session still refreshes"
    );
    println!(
        "{}",
        json!({"case":"rust_oidc_login_refresh_logout", "passed":true,
        "subject":tokens.identity.subject, "audience":tokens.identity.config.audience})
    );
    Ok(())
}
