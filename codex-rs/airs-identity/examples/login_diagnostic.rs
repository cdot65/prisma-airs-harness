//! Diagnose production OIDC validation without writing credentials to native storage.
use codex_airs_identity::IdentityConfig;
use codex_airs_identity::Provider;
use std::io::Write;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    println!("AIRS sign-in diagnostic: alpha.20 validation with specific failure messages");
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!(
            "Usage: airs-harness-signin-diagnostic\nChecks production company sign-in. Prints no tokens and saves no credentials."
        );
        return Ok(());
    }
    anyhow::ensure!(args.is_empty(), "unexpected arguments; use --help");
    println!(
        "This checks company sign-in without saving credentials. Close the browser tab after returning here."
    );
    let provider = Provider::discover(
        IdentityConfig {
            issuer: "https://auth.redtail.cdot.io/realms/redtail".into(),
            client_id: "prisma-airs-harness".into(),
            audience: "stack-airs-inference".into(),
        },
        codex_http_client::NetworkPolicy::unmanaged(),
    )
    .await?;
    let login = provider.browser_login().await?;
    println!(
        "Open this URL on this machine:\n{}",
        login.authorization_url()
    );
    std::io::stdout().flush()?;
    let tokens = login.complete(&provider).await?;
    println!(
        "PASS: access and ID token signatures, issuer, audience, client, subject, nonce, and lifetime validated."
    );
    // No refresh request or revocation: both can affect a shared SSO session.
    // Drop the diagnostic tokens without touching the user's native credential store.
    drop(tokens);
    Ok(())
}
