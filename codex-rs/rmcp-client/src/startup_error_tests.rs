use super::*;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn gateway_upstream_auth_requires_the_measured_structured_contract() {
    for (code, data, expected) in [
        (-32000, json!({"type":"upstream_auth_required"}), true),
        (-32000, json!({"type":"backend_auth_failed"}), false),
        (-32000, json!({"message":"Authorization required"}), false),
        (-32603, json!({"type":"upstream_auth_required"}), false),
    ] {
        let error = ClientOperationError::Service(ServiceError::McpError(rmcp::ErrorData::new(
            rmcp::model::ErrorCode(code),
            "Authorization required",
            Some(data),
        )));
        assert_eq!(is_authentication_required_error(&error.into()), expected);
    }
}

#[test]
fn dispatched_refresh_failure_requires_sign_in_but_store_failure_does_not() {
    for (error, expected) in [
        (AuthError::TokenRefreshFailed("response lost".into()), true),
        (
            AuthError::TokenRefreshRejected("invalid_grant".into()),
            true,
        ),
        (AuthError::CredentialStoreError("locked".into()), false),
    ] {
        assert_eq!(is_authentication_required_error(&error.into()), expected);
    }
}
