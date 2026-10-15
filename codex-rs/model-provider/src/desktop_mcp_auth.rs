//! Process-immutable, memory-only bearer credentials for MCP servers supplied by
//! the native Prisma AIRS Desktop host.
//!
//! No config value, environment variable, keyring entry or credential file is
//! used. Each credential is bound to exactly one configured server name and URL.
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

use codex_api::AuthError;
use codex_api::AuthHeadersFuture;
use codex_api::AuthProvider;
use codex_api::SharedAuthProvider;
use http::HeaderMap;
use http::HeaderValue;
use http::header::AUTHORIZATION;

type SessionCheck = Arc<dyn Fn() -> std::io::Result<()> + Send + Sync>;
static DESKTOP_MCP_AUTH: OnceLock<HashMap<String, BoundCredential>> = OnceLock::new();

/// One Desktop-supplied MCP credential and the server it is bound to.
pub struct DesktopMcpCredential {
    pub server_name: String,
    pub url: String,
    /// Complete `Authorization` header value, for example `Bearer <token>`.
    pub authorization: HeaderValue,
}

struct BoundCredential {
    url: String,
    auth: SharedAuthProvider,
}

struct MemoryBearer {
    authorization: HeaderValue,
    session_check: SessionCheck,
}

impl AuthProvider for MemoryBearer {
    fn add_auth_headers(&self, headers: &mut HeaderMap) {
        if (self.session_check)().is_ok() {
            headers.insert(AUTHORIZATION, self.authorization.clone());
        }
    }

    fn resolve_auth_headers(&self) -> AuthHeadersFuture<'_> {
        Box::pin(async move {
            (self.session_check)().map_err(|_| {
                AuthError::Transient("Desktop MCP authentication was invalidated".into())
            })?;
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, self.authorization.clone());
            Ok(headers)
        })
    }
}

fn bind(
    credentials: Vec<DesktopMcpCredential>,
    session_check: &SessionCheck,
) -> std::io::Result<HashMap<String, BoundCredential>> {
    let mut bound = HashMap::with_capacity(credentials.len());
    for credential in credentials {
        let DesktopMcpCredential {
            server_name,
            url,
            mut authorization,
        } = credential;
        authorization.set_sensitive(true);
        let auth: SharedAuthProvider = Arc::new(MemoryBearer {
            authorization,
            session_check: Arc::clone(session_check),
        });
        if bound
            .insert(server_name, BoundCredential { url, auth })
            .is_some()
        {
            return Err(std::io::Error::other(
                "Desktop MCP server names must be unique",
            ));
        }
    }
    Ok(bound)
}

fn resolve(
    bound: &HashMap<String, BoundCredential>,
    server_name: &str,
    url: &str,
) -> Option<SharedAuthProvider> {
    bound
        .get(server_name)
        .filter(|credential| credential.url == url)
        .map(|credential| Arc::clone(&credential.auth))
}

/// Installs the Desktop MCP credentials for this process exactly once.
pub fn install_desktop_mcp_auth(
    credentials: Vec<DesktopMcpCredential>,
    session_check: SessionCheck,
) -> std::io::Result<()> {
    session_check()?;
    let bound = bind(credentials, &session_check)?;
    DESKTOP_MCP_AUTH
        .set(bound)
        .map_err(|_| std::io::Error::other("Desktop MCP credentials were already bound"))
}

/// Returns the memory-only credential bound to exactly this server name and URL.
pub fn desktop_mcp_auth_provider(server_name: &str, url: &str) -> Option<SharedAuthProvider> {
    DESKTOP_MCP_AUTH
        .get()
        .and_then(|bound| resolve(bound, server_name, url))
}

#[cfg(test)]
#[path = "desktop_mcp_auth_tests.rs"]
mod tests;
