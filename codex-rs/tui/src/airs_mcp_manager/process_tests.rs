use super::*;
use pretty_assertions::assert_eq;

#[test]
fn add_rejects_unsafe_inputs_and_passes_arguments_without_shell_interpretation() {
    assert_eq!(
        validate_new_connection("service-now", "https://gateway.example/mcp/service-now"),
        Ok(())
    );
    for (name, url) in [
        ("../other", "https://gateway.example/mcp"),
        ("", "https://gateway.example/mcp"),
        ("a", "http://gateway.example/mcp"),
        ("a", "https://key@gateway.example/mcp"),
        ("a", "https://gateway.example/mcp?key=secret"),
        ("a", "https://gateway.example/mcp#fragment"),
    ] {
        assert!(validate_new_connection(name, url).is_err());
    }
    assert_eq!(
        arguments(&Operation::Add {
            name: "service-now".into(),
            url: "https://gateway.example/mcp".into()
        }),
        [
            "mcp",
            "add",
            "--no-browser",
            "--scopes",
            "mcp:servers:read,mcp:tools:list,mcp:tools:call",
            "--url",
            "https://gateway.example/mcp",
            "--",
            "service-now"
        ]
    );
    assert_eq!(
        arguments(&Operation::Login("--flag".into())),
        ["mcp", "login", "--no-browser", "--", "--flag"]
    );
}

#[test]
fn only_versioned_https_authorization_events_can_open_a_dialog() {
    assert_eq!(
        authorization_url("provider output https://evil.example/secret"),
        Ok(None)
    );
    assert_eq!(
        authorization_url(r#"{"authorization_url":"https://evil.example"}"#),
        Ok(None)
    );
    assert_eq!(
        authorization_url(
            r#"{"airs_mcp":1,"authorization_url":"https://auth.example/authorize?state=nonce"}"#
        ),
        Ok(Some("https://auth.example/authorize?state=nonce".into()))
    );
    for url in [
        "javascript:alert(1)",
        "http://auth.example/authorize",
        "https://secret@auth.example/authorize",
        "https://auth.example/#secret",
        "https://auth.example/\u{1b}[31m",
    ] {
        let error = authorization_url(
            &serde_json::json!({"airs_mcp":1,"authorization_url":url}).to_string(),
        )
        .unwrap_err();
        assert!(!error.contains(url));
    }
}

#[tokio::test]
async fn line_reader_preserves_partial_input_on_cancellation_and_bounds_output() {
    let (mut writer, reader) = tokio::io::duplex(256);
    let mut reader = BufReader::new(reader);
    let mut line = Vec::new();
    writer.write_all(b"partial").await.unwrap();
    assert!(
        tokio::time::timeout(
            std::time::Duration::from_millis(10),
            bounded_line(&mut reader, &mut line)
        )
        .await
        .is_err()
    );
    writer.write_all(b" response\n").await.unwrap();
    assert_eq!(
        bounded_line(&mut reader, &mut line).await,
        Ok(Some("partial response\n".into()))
    );
    drop(writer);
    assert_eq!(bounded_line(&mut reader, &mut line).await, Ok(None));
    let input = vec![b'x'; 32 * 1024 + 1];
    assert!(
        bounded_line(&mut BufReader::new(input.as_slice()), &mut line)
            .await
            .is_err()
    );
}

#[test]
fn progress_messages_are_typed_and_timeout_never_echoes_provider_details() {
    for (wire, expected) in [
        ("exchanging_code", Progress::ExchangingCode),
        ("saving_credential", Progress::SavingCredential),
    ] {
        let line = serde_json::json!({"airs_mcp":1,"progress":wire}).to_string();
        assert_eq!(login_progress(&line), Ok(Some(expected)));
        assert_eq!(authorization_url(&line), Ok(None));
    }
    assert_eq!(
        login_progress(r#"{"progress":"saving_credential"}"#),
        Ok(None)
    );
    let error =
        login_progress(r#"{"airs_mcp":1,"progress":"timed_out","error_description":"PRIVATE"}"#)
            .unwrap_err();
    assert!(error.contains("fresh link"));
    assert!(!error.contains("PRIVATE"));
    assert!(
        !login_progress(r#"{"airs_mcp":1,"progress":"PRIVATE"}"#)
            .unwrap_err()
            .contains("PRIVATE")
    );
}

#[test]
fn structured_failures_preserve_partial_state_and_reject_untrusted_details() {
    let expected = McpFailure::new(FailureCode::Discovery, ConnectionState::Saved);
    let line = serde_json::json!({"airs_mcp":1,"error":expected}).to_string();
    assert_eq!(operation_failure(&line), Ok(Some(expected)));
    assert_eq!(authorization_url(&line), Ok(None));
    assert_eq!(login_progress(&line), Ok(None));
    assert_eq!(
        operation_failure(r#"{"error":{"code":"authorization"}}"#),
        Ok(None)
    );
    for error in [
        serde_json::json!({"code":"PRIVATE", "connection":"saved"}),
        serde_json::json!({"code":"authorization", "connection":"saved", "details":"PRIVATE"}),
        serde_json::json!({"code":"authorization", "connection":"PRIVATE"}),
    ] {
        let message =
            operation_failure(&serde_json::json!({"airs_mcp":1,"error":error}).to_string())
                .unwrap_err();
        assert!(!message.contains("PRIVATE"));
    }
}
