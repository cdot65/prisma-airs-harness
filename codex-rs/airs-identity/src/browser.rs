use super::Provider;
use super::Tokens;
use anyhow::Context;
use openidconnect::CsrfToken;
use openidconnect::Nonce;
use openidconnect::PkceCodeChallenge;
use openidconnect::PkceCodeVerifier;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use url::Url;

pub struct BrowserLogin {
    config: super::IdentityConfig,
    listener: TcpListener,
    redirect: String,
    authorization_url: Url,
    state: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
}

impl Provider {
    pub async fn browser_login(&self) -> anyhow::Result<BrowserLogin> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .context("cannot bind the login loopback callback")?;
        let redirect = format!(
            "http://127.0.0.1:{}/callback",
            listener.local_addr()?.port()
        );
        let state = CsrfToken::new_random();
        let nonce = Nonce::new_random();
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let mut authorization_url = self.discovery.authorization_endpoint.clone();
        authorization_url.query_pairs_mut().extend_pairs([
            ("client_id", self.config.client_id.as_str()),
            ("redirect_uri", redirect.as_str()),
            ("response_type", "code"),
            ("scope", "openid"),
            ("state", state.secret()),
            ("nonce", nonce.secret()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
        ]);
        Ok(BrowserLogin {
            config: self.config.clone(),
            listener,
            redirect,
            authorization_url,
            state,
            nonce,
            verifier,
        })
    }
}

impl BrowserLogin {
    pub fn authorization_url(&self) -> &Url {
        &self.authorization_url
    }

    pub async fn complete(self, provider: &Provider) -> anyhow::Result<Tokens> {
        anyhow::ensure!(
            self.config == provider.config,
            "login provider changed during authorization"
        );
        tokio::time::timeout(Duration::from_secs(300), self.callback(provider))
            .await
            .context("browser login timed out")?
    }

    async fn callback(self, provider: &Provider) -> anyhow::Result<Tokens> {
        for _ in 0..32 {
            let (mut socket, _) = self.listener.accept().await?;
            let mut data = Vec::new();
            let read = tokio::time::timeout(Duration::from_secs(3), async {
                let mut chunk = [0_u8; 1024];
                while !data.windows(4).any(|w| w == b"\r\n\r\n") {
                    let count = socket.read(&mut chunk).await?;
                    anyhow::ensure!(
                        count != 0 && data.len() + count <= 8192,
                        "invalid callback request"
                    );
                    data.extend_from_slice(&chunk[..count]);
                }
                Ok::<(), anyhow::Error>(())
            })
            .await;
            if !matches!(read, Ok(Ok(()))) {
                continue;
            }
            let code = parse_callback(&data, &self.redirect, &self.state, &provider.config.issuer);
            let message = if code.is_ok() {
                "You may return to AIRS Terminal to finish signing in."
            } else {
                "Invalid login callback."
            };
            let status = if code.is_ok() {
                "200 OK"
            } else {
                "400 Bad Request"
            };
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{message}",
                message.len()
            );
            let _ =
                tokio::time::timeout(Duration::from_secs(2), socket.write_all(reply.as_bytes()))
                    .await;
            let Ok(code) = code else {
                continue;
            };
            let code = code.context("issuer rejected login")?;
            let response = provider
                .token_request(&[
                    ("client_id", &provider.config.client_id),
                    ("grant_type", "authorization_code"),
                    ("code", &code),
                    ("redirect_uri", &self.redirect),
                    ("code_verifier", self.verifier.secret()),
                ])
                .await?;
            return provider.verify(
                response,
                super::tokens::NoncePolicy::Browser(self.nonce.secret()),
            );
        }
        anyhow::bail!("too many invalid login callbacks")
    }
}

fn parse_callback(
    data: &[u8],
    redirect: &str,
    state: &CsrfToken,
    issuer: &str,
) -> anyhow::Result<Option<String>> {
    let request = std::str::from_utf8(data).context("invalid callback encoding")?;
    let mut lines = request.split("\r\n");
    let parts: Vec<_> = lines
        .next()
        .context("missing callback request")?
        .split(' ')
        .collect();
    anyhow::ensure!(
        parts.len() == 3
            && parts[0] == "GET"
            && parts[2] == "HTTP/1.1"
            && parts[1].starts_with("/callback?"),
        "unexpected callback request"
    );
    let expected = Url::parse(redirect)?;
    let host = format!(
        "127.0.0.1:{}",
        expected.port().context("callback port missing")?
    );
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    anyhow::ensure!(hosts == [host.as_str()], "callback host mismatch");
    let url = expected.join(parts[1])?;
    anyhow::ensure!(
        url.path() == "/callback" && url.fragment().is_none(),
        "callback path mismatch"
    );
    let fields: Vec<_> = url.query_pairs().collect();
    let value = |key| -> anyhow::Result<&str> {
        let found: Vec<_> = fields.iter().filter(|(k, _)| k == key).collect();
        anyhow::ensure!(found.len() == 1, "missing or duplicated callback field");
        Ok(found[0].1.as_ref())
    };
    anyhow::ensure!(
        CsrfToken::new(value("state")?.to_owned()) == *state,
        "callback state mismatch"
    );
    anyhow::ensure!(value("iss")? == issuer, "callback issuer mismatch");
    if fields.iter().any(|(k, _)| k == "error") {
        return Ok(None);
    }
    let code = value("code")?;
    anyhow::ensure!(
        !code.is_empty() && code.len() <= 4096,
        "invalid authorization code"
    );
    Ok(Some(code.to_owned()))
}

#[cfg(test)]
#[path = "browser_tests.rs"]
mod tests;
