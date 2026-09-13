use super::Provider;
use super::Tokens;
use super::provider::bounded_json;
use anyhow::Context;
use openidconnect::PkceCodeChallenge;
use openidconnect::PkceCodeVerifier;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::Instant;
use url::Url;

pub struct DeviceLogin {
    config: super::IdentityConfig,
    details: DeviceDetails,
    verifier: PkceCodeVerifier,
    deadline: Instant,
}

#[derive(Deserialize)]
struct DeviceDetails {
    device_code: String,
    user_code: String,
    verification_uri: Url,
    expires_in: u64,
    interval: Option<u64>,
}

impl Provider {
    pub async fn device_login(&self) -> anyhow::Result<DeviceLogin> {
        anyhow::ensure!(
            self.config.resource.is_none() && self.config.scopes.is_empty(),
            "direct MCP OAuth requires browser PKCE login"
        );
        let endpoint = self
            .discovery
            .device_authorization_endpoint
            .as_ref()
            .context("issuer does not support device login")?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let response = self
            .http
            .post(endpoint.clone())
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("scope", "openid"),
                ("code_challenge", challenge.as_str()),
                ("code_challenge_method", "S256"),
            ])
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("device authorization unavailable"))?;
        anyhow::ensure!(
            response.status().is_success(),
            "device authorization rejected"
        );
        let details: DeviceDetails = serde_json::from_value(bounded_json(response).await?)
            .map_err(|_| anyhow::anyhow!("invalid device authorization response"))?;
        anyhow::ensure!(
            (1..=900).contains(&details.expires_in)
                && !details.device_code.is_empty()
                && details.device_code.len() <= 4096
                && !details.user_code.is_empty()
                && details.user_code.len() <= 128
                && details.user_code.bytes().all(|c| c.is_ascii_graphic()),
            "invalid device code or lifetime"
        );
        anyhow::ensure!(
            details.verification_uri.origin() == endpoint.origin()
                && details.verification_uri.username().is_empty()
                && details.verification_uri.password().is_none()
                && details.verification_uri.fragment().is_none(),
            "untrusted device verification URL"
        );
        let deadline = Instant::now() + Duration::from_secs(details.expires_in);
        Ok(DeviceLogin {
            config: self.config.clone(),
            details,
            verifier,
            deadline,
        })
    }
}

impl DeviceLogin {
    pub fn verification_uri(&self) -> &Url {
        &self.details.verification_uri
    }
    pub fn user_code(&self) -> &str {
        &self.details.user_code
    }

    pub async fn complete(self, provider: &Provider) -> anyhow::Result<Tokens> {
        anyhow::ensure!(
            self.config == provider.config,
            "login provider changed during authorization"
        );
        let deadline = self.deadline;
        tokio::time::timeout_at(deadline, self.poll(provider))
            .await
            .context("device authorization expired")?
    }

    async fn poll(self, provider: &Provider) -> anyhow::Result<Tokens> {
        let mut interval = self.details.interval.unwrap_or(5).max(1);
        loop {
            tokio::time::sleep(Duration::from_secs(interval)).await;
            let response = provider
                .http
                .post(provider.discovery.token_endpoint.clone())
                .form(&[
                    ("client_id", provider.config.client_id.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", self.details.device_code.as_str()),
                    ("code_verifier", self.verifier.secret()),
                ])
                .send()
                .await
                .map_err(|_| anyhow::anyhow!("device token exchange interrupted; sign in again"))?;
            let status = response.status();
            let body = bounded_json(response).await?;
            if status.is_success() {
                let response = serde_json::from_value(body)
                    .map_err(|_| anyhow::anyhow!("invalid device token response"))?;
                return provider.verify(response, super::tokens::NoncePolicy::Device);
            }
            anyhow::ensure!(
                status == reqwest::StatusCode::BAD_REQUEST,
                "device authorization rejected"
            );
            match body.get("error").and_then(serde_json::Value::as_str) {
                Some("authorization_pending") => {}
                Some("slow_down") => interval = interval.saturating_add(5),
                _ => anyhow::bail!("device authorization denied or expired"),
            }
        }
    }
}
