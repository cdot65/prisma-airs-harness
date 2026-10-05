//! A process-immutable, memory-only credential for the owned desktop transport.
use std::sync::Arc;
use std::sync::OnceLock;

use codex_api::AuthError;
use codex_api::AuthHeadersFuture;
use codex_api::AuthProvider;
use codex_api::SharedAuthProvider;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::error::CodexErr;
use http::HeaderMap;
use http::HeaderValue;

type SessionCheck = Arc<dyn Fn() -> std::io::Result<()> + Send + Sync>;
static DESKTOP_AUTH: OnceLock<DesktopGatewayAuth> = OnceLock::new();

struct DesktopGatewayAuth {
    provider: ModelProviderInfo,
    auth: SharedAuthProvider,
}

impl DesktopGatewayAuth {
    fn resolve(
        &self,
        provider: &ModelProviderInfo,
    ) -> codex_protocol::error::Result<SharedAuthProvider> {
        if provider != &self.provider {
            Err(CodexErr::Fatal(
                "Desktop Gateway provider binding changed".into(),
            ))
        } else {
            Ok(Arc::clone(&self.auth))
        }
    }
}

struct MemoryCredential {
    key: HeaderValue,
    session_check: SessionCheck,
}

impl AuthProvider for MemoryCredential {
    fn add_auth_headers(&self, headers: &mut HeaderMap) {
        if (self.session_check)().is_ok() {
            headers.insert("x-portkey-api-key", self.key.clone());
        }
    }

    fn resolve_auth_headers(&self) -> AuthHeadersFuture<'_> {
        Box::pin(async move {
            (self.session_check)().map_err(|_| {
                AuthError::Transient("Desktop Gateway authentication was invalidated".into())
            })?;
            let mut headers = HeaderMap::new();
            headers.insert("x-portkey-api-key", self.key.clone());
            Ok(headers)
        })
    }
}

/// Installs exactly one protected Gateway credential in this process.
/// No config, command helper, environment variable or persistent auth store is used.
/// Every later provider resolution must match this immutable provider binding.
pub fn install_desktop_gateway_auth(
    provider: ModelProviderInfo,
    mut key: HeaderValue,
    session_check: SessionCheck,
) -> std::io::Result<()> {
    if provider.gateway.is_none()
        || provider.base_url.is_none()
        || provider.auth.is_some()
        || provider.aws.is_some()
        || provider.env_key.is_some()
        || provider.experimental_bearer_token.is_some()
        || provider
            .env_http_headers
            .as_ref()
            .is_some_and(|headers| !headers.is_empty())
    {
        return Err(std::io::Error::other(
            "Invalid desktop Gateway provider binding",
        ));
    }
    session_check()?;
    key.set_sensitive(true);
    DESKTOP_AUTH
        .set(DesktopGatewayAuth {
            provider,
            auth: Arc::new(MemoryCredential { key, session_check }),
        })
        .map_err(|_| std::io::Error::other("Desktop Gateway identity was already bound"))
}

#[cfg(test)]
#[path = "desktop_gateway_auth_tests.rs"]
mod tests;

pub(crate) fn resolve(
    provider: &ModelProviderInfo,
) -> Option<codex_protocol::error::Result<SharedAuthProvider>> {
    DESKTOP_AUTH.get().map(|binding| binding.resolve(provider))
}
