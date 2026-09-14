//! Keep SDK refresh requests within the permissions actually granted at login.
use oauth2::TokenResponse;
use rmcp::transport::auth::AuthorizationMetadata;

use super::StoredOAuthTokens;

pub(crate) fn restrict_refresh_scopes(
    mut metadata: AuthorizationMetadata,
    tokens: &StoredOAuthTokens,
) -> AuthorizationMetadata {
    let offline_granted = tokens.token_response.0.scopes().is_some_and(|scopes| {
        scopes
            .iter()
            .any(|scope| scope.as_str() == "offline_access")
    });
    // RMCP 3.2 appends offline_access whenever discovery advertises it, including
    // during refresh. Server support is not a grant to this client. RFC 6749 §6
    // forbids expanding the original authorization in a refresh request.
    if !offline_granted && let Some(scopes) = metadata.scopes_supported.as_mut() {
        scopes.retain(|scope| scope != "offline_access");
    }
    metadata
}

#[cfg(test)]
#[path = "refresh_scopes_tests.rs"]
mod tests;
