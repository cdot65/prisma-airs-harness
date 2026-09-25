use super::*;
use pretty_assertions::assert_eq;

#[test]
fn innermost_typed_failure_survives_generic_context_without_exposing_details() {
    let expected = McpFailure::new(FailureCode::CredentialStore, ConnectionState::Saved);
    let error = with_fallback(
        anyhow::anyhow!("PRIVATE credential / callback / provider payload").context(expected),
        McpFailure::new(FailureCode::Configuration, ConnectionState::Unknown),
    );
    assert_eq!(classify(&error), expected);
    let wire = serde_json::to_string(&classify(&error)).unwrap();
    assert!(!wire.contains("PRIVATE"));
    assert!(classify(&error).to_string().contains("Do not add it again"));
}

#[test]
fn untyped_errors_are_not_guessed_from_provider_text() {
    let error = anyhow::anyhow!("HTTP 403 invalid_grant secret-tool locked PRIVATE");
    assert_eq!(
        classify(&error),
        McpFailure::new(FailureCode::Unknown, ConnectionState::Unknown)
    );
    assert!(!classify(&error).to_string().contains("PRIVATE"));
}

#[test]
fn fallback_classifies_untyped_configuration_errors() {
    let expected = McpFailure::new(FailureCode::Configuration, ConnectionState::Unchanged);
    let error = with_fallback(anyhow::anyhow!("PRIVATE path"), expected);
    assert_eq!(classify(&error), expected);
}

#[test]
fn typed_timeout_preserves_the_saved_connection_state() {
    let error = anyhow::Error::new(codex_rmcp_client::McpOAuthLoginTimeout).context(
        McpFailure::new(FailureCode::Authorization, ConnectionState::Saved),
    );
    assert_eq!(
        classify(&error),
        McpFailure::new(FailureCode::TimedOut, ConnectionState::Saved),
    );
}
