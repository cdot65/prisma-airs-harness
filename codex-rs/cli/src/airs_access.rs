//! A single disclosed authenticated Responses probe. Results never authorize MCP access.
use super::airs_credentials;
use codex_http_client::ClientRouteClass;
use codex_http_client::HttpClientBuilder;
use codex_http_client::HttpClientFactory;
use codex_http_client::OutboundProxyPolicy;
use codex_model_provider_info::ModelProviderInfo;
use codex_utils_home_dir::airs_session::AirsSessionGuard;
use http::HeaderValue;
use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use url::Url;
use uuid::Uuid;

pub(super) const DISCLOSURE: &str = "Checking gateway access with one minimal inference request (up to 16 output tokens). This sends only a fixed connectivity message, with no local files or tools. The request asks the provider not to store the response; gateway logging policy still applies.";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const OVERALL_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE_BYTES: usize = 64 * 1024;
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Failure {
    Configuration,
    Credential,
    SignedOut,
    Offline,
    Timeout,
    Denied,
    Redirect,
    Rejected,
    InvalidResponse,
    OversizedResponse,
}

impl Failure {
    fn detail(self) -> &'static str {
        match self {
            Self::Configuration => "The selected gateway or model configuration is invalid.",
            Self::Credential => {
                "The saved credential could not be read or refreshed. Run airs-harness status to inspect local sign-in."
            }
            Self::SignedOut => "Authentication changed or this environment was signed out.",
            Self::Offline => "The gateway connection failed. Check connectivity, DNS and TLS.",
            Self::Timeout => "The gateway access check reached its time limit.",
            Self::Denied => {
                "The gateway denied access. Check this credential's workspace and policy permissions."
            }
            Self::Redirect => {
                "The gateway requested a redirect. Credentials were not forwarded; check its configured API root."
            }
            Self::Rejected => {
                "The gateway rejected the probe. Check its route and policy configuration."
            }
            Self::InvalidResponse => {
                "The gateway did not return a successful Responses API result."
            }
            Self::OversizedResponse => {
                "The gateway response exceeded the 64 KiB verification limit."
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Verification {
    pub(super) request_id: Uuid,
    pub(super) outcome: Result<(), Failure>,
}

impl Verification {
    pub(super) fn summary(&self) -> String {
        let result = match self.outcome {
            Ok(()) => "Gateway access verified by one inference response. MCP permissions were not tested.".to_owned(),
            Err(reason) => format!("Gateway access not yet verified. {}", reason.detail()),
        };
        format!("{result}\nClient correlation ID: {}", self.request_id)
    }

    pub(super) fn after_login(&self) -> String {
        match self.outcome {
            Ok(()) => self.summary(),
            Err(reason) => format!(
                "Credential saved; gateway access not yet verified. {}\nClient correlation ID: {}\nRetry: airs-harness doctor --verify-access (select the same environment).",
                reason.detail(),
                self.request_id,
            ),
        }
    }
}

struct Prepared {
    endpoint: Url,
    header_name: &'static str,
    credential: HeaderValue,
    body: Value,
    session: AirsSessionGuard,
}

fn read_public_file(path: &Path) -> Result<String, Failure> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(path).map_err(|_| Failure::Configuration)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if file
            .metadata()
            .map_err(|_| Failure::Configuration)?
            .file_attributes()
            & 0x0400
            != 0
        {
            return Err(Failure::Configuration);
        }
    }
    if !file
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() <= MAX_CONFIG_BYTES)
        || !std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
    {
        return Err(Failure::Configuration);
    }
    let mut value = String::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_string(&mut value)
        .map_err(|_| Failure::Configuration)?;
    if value.len() > MAX_CONFIG_BYTES as usize {
        return Err(Failure::Configuration);
    }
    Ok(value)
}

fn probe_configuration(config: &toml::Value, catalog: &Value) -> Result<(Url, Value), Failure> {
    let provider: ModelProviderInfo = config
        .get("model_providers")
        .and_then(|providers| providers.get("airs"))
        .ok_or(Failure::Configuration)?
        .clone()
        .try_into()
        .map_err(|_| Failure::Configuration)?;
    provider.validate().map_err(|_| Failure::Configuration)?;
    if provider.gateway.is_none() {
        return Err(Failure::Configuration);
    }
    let base_url = provider.base_url.as_deref().ok_or(Failure::Configuration)?;
    if base_url.len() > 2_048 || !base_url.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(Failure::Configuration);
    }
    let mut endpoint = Url::parse(base_url).map_err(|_| Failure::Configuration)?;
    let loopback = match endpoint.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if !(endpoint.scheme() == "https" || endpoint.scheme() == "http" && loopback)
        || endpoint.host_str().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(Failure::Configuration);
    }
    let selected = config
        .get("model")
        .and_then(toml::Value::as_str)
        .ok_or(Failure::Configuration)?;
    if selected.len() > 2_048 || !selected.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(Failure::Configuration);
    }
    if !catalog
        .get("models")
        .and_then(Value::as_array)
        .is_some_and(|models| {
            models
                .iter()
                .any(|model| model.get("slug").and_then(Value::as_str) == Some(selected))
        })
    {
        return Err(Failure::Configuration);
    }
    let route = provider
        .request_model(selected)
        .map_err(|_| Failure::Configuration)?;
    let mut body = serde_json::json!({
        "input": "Reply only with OK. This is a Prisma AIRS Harness connectivity check.",
        "max_output_tokens": 16, "store": false, "stream": false,
    });
    if let Some(route) = route {
        body["model"] = Value::String(route);
    }
    endpoint.set_path(&format!(
        "{}/responses",
        endpoint.path().trim_end_matches('/')
    ));
    Ok((endpoint, body))
}

fn validate_token(token: &str) -> Result<&str, Failure> {
    let token = token.trim();
    if token.is_empty()
        || token.len() > 16_384
        || !token.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Failure::Credential);
    }
    Ok(token)
}

async fn helper_token(mut command: Command) -> Result<String, Failure> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| Failure::Credential)?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .ok_or(Failure::Credential)?
        .take(16_387)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| Failure::Credential)?;
    if bytes.len() > 16_386 {
        return Err(Failure::Credential);
    }
    if !child
        .wait()
        .await
        .map_err(|_| Failure::Credential)?
        .success()
    {
        return Err(Failure::Credential);
    }
    let token = std::str::from_utf8(&bytes).map_err(|_| Failure::Credential)?;
    Ok(validate_token(token)?.to_owned())
}

async fn preparation(home: &Path) -> Result<Prepared, Failure> {
    let session = AirsSessionGuard::capture(home).map_err(|_| Failure::SignedOut)?;
    // Wait cooperatively so another process's browser login cannot defeat the check deadline.
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let lock = options
        .open(home.join(".configuration.lock"))
        .map_err(|_| Failure::Configuration)?;
    if !lock.metadata().is_ok_and(|metadata| metadata.is_file()) {
        return Err(Failure::Configuration);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if lock
            .metadata()
            .map_err(|_| Failure::Configuration)?
            .file_attributes()
            & 0x0400
            != 0
        {
            return Err(Failure::Configuration);
        }
    }
    loop {
        match lock.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => {
                tokio::time::sleep(Duration::from_millis(25)).await
            }
            Err(std::fs::TryLockError::Error(_)) => return Err(Failure::Configuration),
        }
        session.check().map_err(|_| Failure::SignedOut)?;
    }
    session.check().map_err(|_| Failure::SignedOut)?;
    let config: toml::Value = toml::from_str(&read_public_file(&home.join("config.toml"))?)
        .map_err(|_| Failure::Configuration)?;
    let catalog_path = config
        .get("model_catalog_json")
        .and_then(toml::Value::as_str)
        .ok_or(Failure::Configuration)?;
    let catalog = serde_json::from_str(&read_public_file(Path::new(catalog_path))?)
        .map_err(|_| Failure::Configuration)?;
    let (endpoint, body) = probe_configuration(&config, &catalog)?;
    let bound = if home
        .join("credential-binding.json")
        .try_exists()
        .map_err(|_| Failure::Credential)?
    {
        Some(
            airs_credentials::read_binding(home)
                .map_err(|_| Failure::Credential)?
                .id,
        )
    } else {
        None
    };
    let variable = config
        .get("model_providers")
        .and_then(|value| value.get("airs"))
        .and_then(|value| value.get("env_http_headers"))
        .and_then(|value| value.get("x-portkey-api-key"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    // The trusted helper owns its own configuration lock while resolving/refreshing secrets.
    drop(lock);
    let (header_name, token) = if let Some(binding) = bound {
        let mut command = Command::new(std::env::current_exe().map_err(|_| Failure::Credential)?);
        command
            .arg("credential")
            .arg("--home")
            .arg(home)
            .arg("--binding")
            .arg(binding.to_string());
        let token = helper_token(command).await?;
        ("authorization", format!("Bearer {token}"))
    } else {
        let token =
            std::env::var(variable.ok_or(Failure::Credential)?).map_err(|_| Failure::Credential)?;
        ("x-portkey-api-key", validate_token(&token)?.to_owned())
    };
    session.check().map_err(|_| Failure::SignedOut)?;
    let mut credential = HeaderValue::from_str(&token).map_err(|_| Failure::Credential)?;
    credential.set_sensitive(true);
    Ok(Prepared {
        endpoint,
        header_name,
        credential,
        body,
        session,
    })
}

async fn probe(prepared: Prepared, request_id: Uuid) -> Result<(), Failure> {
    let client = HttpClientBuilder::new()
        .without_redirects()
        .without_request_logging()
        .connect_timeout(CONNECT_TIMEOUT)
        .build_respecting_outbound_proxy_policy(
            &HttpClientFactory::new(OutboundProxyPolicy::ReqwestDefault),
            prepared.endpoint.as_str(),
            ClientRouteClass::Api,
        )
        .map_err(|_| Failure::Offline)?;
    prepared.session.check().map_err(|_| Failure::SignedOut)?;
    let mut response = client
        .post(prepared.endpoint.as_str())
        .header(prepared.header_name, prepared.credential)
        .header("x-client-request-id", request_id.to_string())
        .header(
            "user-agent",
            format!("airs-harness/{}", super::airs_harness::version()),
        )
        .json(&prepared.body)
        .timeout(OVERALL_TIMEOUT)
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                Failure::Timeout
            } else {
                Failure::Offline
            }
        })?;
    let status = response.status();
    if status.is_redirection() {
        return Err(Failure::Redirect);
    }
    if matches!(status.as_u16(), 401 | 403) {
        return Err(Failure::Denied);
    }
    if !status.is_success() {
        return Err(Failure::Rejected);
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(Failure::OversizedResponse);
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| {
        if error.is_timeout() {
            Failure::Timeout
        } else {
            Failure::InvalidResponse
        }
    })? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(Failure::OversizedResponse);
        }
        bytes.extend_from_slice(&chunk);
        prepared.session.check().map_err(|_| Failure::SignedOut)?;
    }
    prepared.session.check().map_err(|_| Failure::SignedOut)?;
    let body: Value = serde_json::from_slice(&bytes).map_err(|_| Failure::InvalidResponse)?;
    if body.get("object").and_then(Value::as_str) != Some("response")
        || !matches!(
            body.get("status").and_then(Value::as_str),
            Some("completed" | "incomplete")
        )
        || body.get("error").is_some_and(|error| !error.is_null())
        || !body
            .get("output")
            .and_then(Value::as_array)
            .is_some_and(|output| {
                output
                    .iter()
                    .any(|item| item.get("type").and_then(Value::as_str) == Some("message"))
            })
    {
        return Err(Failure::InvalidResponse);
    }
    Ok(())
}

/// The caller discloses the probe first; failures preserve all saved credentials.
pub(super) async fn verify(home: &Path) -> Verification {
    let request_id = Uuid::new_v4();
    let outcome = tokio::time::timeout(OVERALL_TIMEOUT, async {
        let session = AirsSessionGuard::capture(home).map_err(|_| Failure::SignedOut)?;
        let operation = async {
            let prepared = preparation(home).await?;
            probe(prepared, request_id).await
        };
        tokio::pin!(operation);
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            tokio::select! {
                biased;
                _ = interval.tick() => session.check().map_err(|_| Failure::SignedOut)?,
                result = &mut operation => {
                    session.check().map_err(|_| Failure::SignedOut)?;
                    return result;
                }
            }
        }
    })
    .await
    .unwrap_or(Err(Failure::Timeout));
    Verification {
        request_id,
        outcome,
    }
}

#[cfg(test)]
#[path = "airs_access_tests.rs"]
mod tests;
