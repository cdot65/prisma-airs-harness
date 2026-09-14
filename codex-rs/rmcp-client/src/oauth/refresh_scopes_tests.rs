use oauth2::AccessToken;
use oauth2::RefreshToken;
use oauth2::Scope;
use oauth2::basic::BasicTokenType;
use pretty_assertions::assert_eq;
use rmcp::transport::auth::AuthorizationManager;
use rmcp::transport::auth::AuthorizationMetadata;
use rmcp::transport::auth::OAuthTokenResponse;
use rmcp::transport::auth::VendorExtraTokenFields;
use serde_json::json;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;
use wiremock::matchers::method;
use wiremock::matchers::path;

use super::super::StoredOAuthTokens;
use super::super::WrappedOAuthTokenResponse;
use super::super::install_tokens_in_manager;
use super::restrict_refresh_scopes;

#[tokio::test]
async fn sdk_refresh_preserves_granted_scopes_without_adding_offline_access() -> anyhow::Result<()>
{
    for granted in [vec!["read"], vec!["read", "offline_access"]] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "refreshed", "token_type": "Bearer", "expires_in": 300,
                "refresh_token": "next-refresh", "scope": granted.join(" ")
            })))
            .expect(1)
            .mount(&server)
            .await;
        let metadata: AuthorizationMetadata = serde_json::from_value(json!({
            "issuer": server.uri(),
            "authorization_endpoint": format!("{}/authorize", server.uri()),
            "token_endpoint": format!("{}/token", server.uri()),
            "response_types_supported": ["code"],
            "scopes_supported": ["read", "offline_access"]
        }))?;
        let mut response = OAuthTokenResponse::new(
            AccessToken::new("expired".to_owned()),
            BasicTokenType::Bearer,
            VendorExtraTokenFields::default(),
        );
        response.set_refresh_token(Some(RefreshToken::new("refresh".to_owned())));
        response.set_scopes(Some(
            granted
                .iter()
                .map(|scope| Scope::new((*scope).to_owned()))
                .collect(),
        ));
        let tokens = StoredOAuthTokens {
            server_name: "read-tools".to_owned(),
            url: format!("{}/mcp", server.uri()),
            issuer: Some(server.uri()),
            client_id: "public-native-client".to_owned(),
            token_response: WrappedOAuthTokenResponse(response),
            expires_at: Some(0),
        };
        let mut manager = AuthorizationManager::new(tokens.url.clone()).await?;
        manager.set_metadata(restrict_refresh_scopes(metadata, &tokens));
        install_tokens_in_manager(&mut manager, &tokens).await?;
        manager.refresh_token().await?;
        let requests = server.received_requests().await.unwrap();
        let request = requests
            .iter()
            .find(|request| request.url.path() == "/token")
            .unwrap();
        let fields: std::collections::BTreeMap<_, _> = url::form_urlencoded::parse(&request.body)
            .into_owned()
            .collect();
        assert_eq!(fields.get("scope"), Some(&granted.join(" ")));
        assert_eq!(fields.get("resource"), Some(&tokens.url));
    }
    Ok(())
}
