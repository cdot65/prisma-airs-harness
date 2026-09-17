use super::Provider;
use anyhow::Context;
use openidconnect::AccessTokenHash;
use openidconnect::ClientId;
use openidconnect::IssuerUrl;
use openidconnect::Nonce;
use openidconnect::OAuth2TokenResponse;
use openidconnect::TokenResponse;
use openidconnect::core::CoreIdTokenVerifier;
use openidconnect::core::CoreJwsSigningAlgorithm;
use openidconnect::core::CoreTokenResponse;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub config: super::IdentityConfig,
    pub subject: String,
    pub display_name: Option<String>,
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tests;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tokens {
    pub identity: Identity,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub nonce: Option<String>,
}

#[derive(Deserialize)]
struct AccessClaims {
    sub: String,
    azp: String,
    exp: u64,
    iat: u64,
}

pub(crate) enum NoncePolicy<'a> {
    Browser(&'a str),
    Device,
    Refresh(Option<&'a str>),
}

impl Provider {
    pub(crate) fn verify(
        &self,
        response: CoreTokenResponse,
        policy: NoncePolicy<'_>,
    ) -> anyhow::Result<Tokens> {
        anyhow::ensure!(
            *response.token_type() == openidconnect::core::CoreTokenType::Bearer,
            "issuer returned an unsupported token type"
        );
        let nonce = match &policy {
            NoncePolicy::Browser(nonce) => Some(*nonce),
            NoncePolicy::Device => None,
            NoncePolicy::Refresh(nonce) => *nonce,
        };
        let verifier = CoreIdTokenVerifier::new_public_client(
            ClientId::new(self.config.client_id.clone()),
            IssuerUrl::new(self.config.issuer.clone())?,
            self.oidc_keys.clone(),
        )
        .set_allowed_algs([CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256]);
        let id_token = response
            .id_token()
            .context("issuer did not return an ID token")?;
        let nonce_check = |actual: Option<&Nonce>| -> Result<(), String> {
            match (nonce, actual) {
                (Some(expected), Some(actual)) if Nonce::new(expected.to_owned()) == *actual => {
                    Ok(())
                }
                (None, None) => Ok(()),
                (Some(_), None) if matches!(policy, NoncePolicy::Refresh(_)) => Ok(()),
                _ => Err("identity nonce mismatch".into()),
            }
        };
        let claims = id_token.claims(&verifier, nonce_check).map_err(|error| {
            use openidconnect::ClaimsVerificationError as E;
            let kind = match error {
                E::Expired(_) => "expiry",
                E::InvalidAudience(_) => "audience",
                E::InvalidIssuer(_) => "issuer",
                E::InvalidNonce(_) => "nonce",
                E::InvalidSubject(_) => "subject",
                E::SignatureVerification(_) => "signature",
                _ => "claims",
            };
            anyhow::anyhow!("ID token verification failed ({kind})")
        })?;
        anyhow::ensure!(
            claims
                .authorized_party()
                .is_none_or(|party| party.as_str() == self.config.client_id),
            "ID token authorized party mismatch"
        );
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                response.access_token(),
                id_token.signing_alg()?,
                id_token.signing_key(&verifier)?,
            )?;
            anyhow::ensure!(*expected == actual, "access token hash mismatch");
        }
        let access_token = response.access_token().secret();
        let header = jsonwebtoken::decode_header(access_token)
            .map_err(|_| anyhow::anyhow!("invalid access JWT"))?;
        anyhow::ensure!(
            header.alg == jsonwebtoken::Algorithm::RS256,
            "unsupported access JWT algorithm"
        );
        let kid = header.kid.context("access JWT has no key identifier")?;
        let key = self
            .access_keys
            .find(&kid)
            .context("access JWT uses unknown issuer key")?;
        let key = jsonwebtoken::DecodingKey::from_jwk(key).context("invalid issuer signing key")?;
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub", "iat"]);
        validation.leeway = 0;
        validation.validate_nbf = true;
        let access = jsonwebtoken::decode::<AccessClaims>(access_token, &key, &validation)
            .map_err(|_| anyhow::anyhow!("access JWT verification failed"))?
            .claims;
        let now = jsonwebtoken::get_current_timestamp();
        anyhow::ensure!(
            access.azp == self.config.client_id,
            "access JWT authorized party does not match the configured client"
        );
        anyhow::ensure!(
            access.sub == claims.subject().as_str(),
            "access JWT subject does not match the verified ID token"
        );
        anyhow::ensure!(
            access.iat <= now + 30,
            "access JWT was issued {} seconds ahead of this machine's clock; synchronize the system clock and retry sign-in",
            access.iat.saturating_sub(now)
        );
        anyhow::ensure!(
            access.exp > now + 15,
            "access JWT has only {} seconds remaining; check the system clock and issuer token lifetime, then retry sign-in",
            access.exp.saturating_sub(now)
        );
        let refresh_token = response
            .refresh_token()
            .context("issuer did not return a refresh token")?
            .secret()
            .clone();
        let display_name = claims
            .preferred_username()
            .map(|n| n.as_str().to_owned())
            .filter(|n| n.len() <= 128 && n.chars().all(|c| !c.is_control()));
        Ok(Tokens {
            identity: Identity {
                config: self.config.clone(),
                subject: access.sub,
                display_name,
            },
            access_token: access_token.to_owned(),
            refresh_token,
            expires_at: access.exp,
            nonce: nonce.map(str::to_owned),
        })
    }

    pub async fn refresh(&self, previous: &Tokens) -> anyhow::Result<Tokens> {
        anyhow::ensure!(
            previous.identity.config == self.config,
            "refresh identity configuration changed"
        );
        let response = self
            .token_request(&[
                ("client_id", &self.config.client_id),
                ("grant_type", "refresh_token"),
                ("refresh_token", &previous.refresh_token),
            ])
            .await?;
        let tokens = self.verify(response, NoncePolicy::Refresh(previous.nonce.as_deref()))?;
        anyhow::ensure!(
            tokens.identity.subject == previous.identity.subject
                && tokens.refresh_token != previous.refresh_token,
            "refresh identity changed or token was not rotated"
        );
        Ok(tokens)
    }

    pub async fn revoke(&self, tokens: &Tokens) -> anyhow::Result<()> {
        anyhow::ensure!(
            tokens.identity.config == self.config,
            "revocation identity configuration changed"
        );
        let endpoint = self
            .discovery
            .revocation_endpoint
            .as_ref()
            .context("issuer does not support revocation")?;
        let response = self
            .http
            .post(endpoint.clone())
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("token_type_hint", "refresh_token"),
                ("token", tokens.refresh_token.as_str()),
            ])
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("issuer revocation unavailable"))?;
        anyhow::ensure!(response.status().is_success(), "issuer revocation rejected");
        Ok(())
    }
}
