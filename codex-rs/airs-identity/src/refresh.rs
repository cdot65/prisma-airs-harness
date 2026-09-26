//! Admission precedes durable refresh state; execution never proves that a token was not consumed.
use super::Provider;
use super::Tokens;
use super::tokens::NoncePolicy;

/// An admitted one-shot exchange. The caller must persist its pending marker before execution.
/// Any error after execution begins may have consumed the rotating refresh token.
pub struct PreparedRefresh<'a> {
    pub(crate) provider: &'a Provider,
    pub(crate) previous: &'a Tokens,
    pub(crate) permit: codex_http_client::NetworkPermit,
}

impl PreparedRefresh<'_> {
    pub async fn complete(self) -> anyhow::Result<Tokens> {
        let provider = self.provider;
        let previous = self.previous;
        self.permit
            .run(async {
                let response = provider
                    .token_request_under_permit(&[
                        ("client_id", &provider.config.client_id),
                        ("grant_type", "refresh_token"),
                        ("refresh_token", &previous.refresh_token),
                    ])
                    .await?;
                let tokens =
                    provider.verify(response, NoncePolicy::Refresh(previous.nonce.as_deref()))?;
                anyhow::ensure!(
                    tokens.identity.subject == previous.identity.subject
                        && tokens.refresh_token != previous.refresh_token,
                    "refresh identity changed or token was not rotated"
                );
                Ok(tokens)
            })
            .await?
    }
}
