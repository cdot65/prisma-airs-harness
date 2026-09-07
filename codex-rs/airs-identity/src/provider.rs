use anyhow::Context;
use openidconnect::core::CoreJsonWebKeySet;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IdentityConfig {
    pub issuer: String,
    pub client_id: String,
    pub audience: String,
}

#[derive(Deserialize)]
pub(crate) struct Discovery {
    issuer: String,
    pub(crate) authorization_endpoint: Url,
    pub(crate) token_endpoint: Url,
    jwks_uri: Url,
    pub(crate) device_authorization_endpoint: Option<Url>,
    pub(crate) revocation_endpoint: Option<Url>,
    code_challenge_methods_supported: Vec<String>,
}

pub struct Provider {
    pub(crate) config: IdentityConfig,
    pub(crate) discovery: Discovery,
    pub(crate) http: reqwest::Client,
    pub(crate) oidc_keys: CoreJsonWebKeySet,
    pub(crate) access_keys: jsonwebtoken::jwk::JwkSet,
}

fn secure_url(url: &Url) -> anyhow::Result<()> {
    anyhow::ensure!(
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none(),
        "identity endpoints require HTTPS without credentials, query or fragment"
    );
    Ok(())
}

pub(crate) async fn bounded_json(response: reqwest::Response) -> anyhow::Result<serde_json::Value> {
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| anyhow::anyhow!("identity response interrupted"))?
    {
        anyhow::ensure!(
            bytes.len() + chunk.len() <= 262_144,
            "identity response exceeds size limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| anyhow::anyhow!("invalid identity response"))
}

impl Provider {
    pub async fn discover(config: IdentityConfig) -> anyhow::Result<Self> {
        let issuer = Url::parse(&config.issuer).context("invalid issuer URL")?;
        secure_url(&issuer)?;
        anyhow::ensure!(
            !config.issuer.ends_with('/')
                && !config.client_id.is_empty()
                && !config.audience.is_empty(),
            "issuer, client and resource audience are required"
        );
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(15))
            .build()?;
        let response = http
            .get(format!(
                "{}/.well-known/openid-configuration",
                config.issuer
            ))
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("issuer discovery unavailable"))?;
        anyhow::ensure!(response.status().is_success(), "issuer discovery rejected");
        let discovery: Discovery = serde_json::from_value(bounded_json(response).await?)
            .map_err(|_| anyhow::anyhow!("invalid issuer metadata"))?;
        anyhow::ensure!(
            discovery.issuer == config.issuer,
            "discovery issuer mismatch"
        );
        anyhow::ensure!(
            discovery
                .code_challenge_methods_supported
                .iter()
                .any(|m| m == "S256"),
            "issuer must support PKCE S256"
        );
        for endpoint in [
            &discovery.authorization_endpoint,
            &discovery.token_endpoint,
            &discovery.jwks_uri,
        ]
        .into_iter()
        .chain(discovery.device_authorization_endpoint.iter())
        .chain(discovery.revocation_endpoint.iter())
        {
            secure_url(endpoint)?;
            anyhow::ensure!(
                endpoint.origin() == issuer.origin(),
                "identity endpoint must share the issuer origin"
            );
        }
        let response = http
            .get(discovery.jwks_uri.clone())
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("issuer keys unavailable"))?;
        anyhow::ensure!(response.status().is_success(), "issuer keys rejected");
        let keys = bounded_json(response).await?;
        let oidc_keys =
            serde_json::from_value(keys.clone()).context("invalid issuer public keys")?;
        let access_keys = serde_json::from_value(keys).context("invalid issuer public keys")?;
        Ok(Self {
            config,
            discovery,
            http,
            oidc_keys,
            access_keys,
        })
    }

    pub(crate) async fn token_request(
        &self,
        fields: &[(&str, &str)],
    ) -> anyhow::Result<openidconnect::core::CoreTokenResponse> {
        let response = self
            .http
            .post(self.discovery.token_endpoint.clone())
            .form(fields)
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("token exchange interrupted; sign in again"))?;
        anyhow::ensure!(
            response.status().is_success(),
            "token exchange rejected; sign in again"
        );
        serde_json::from_value(bounded_json(response).await?)
            .map_err(|_| anyhow::anyhow!("invalid token response; sign in again"))
    }
}
