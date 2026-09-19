use super::*;
use pretty_assertions::assert_eq;

#[test]
fn failed_checks_remain_a_report_and_invalid_output_is_redacted() {
    let bytes = br#"{"schema_version":1,"product":"Prisma AIRS Harness","authentication":"Workspace API key","checks":[{"name":"gateway_access","passed":false,"detail":"HTTP 403"}]}"#;
    let report = process::parse(bytes).unwrap();
    assert_eq!(
        (report.checks[0].passed, report.checks[0].detail.as_str()),
        (false, "HTTP 403")
    );
    for bytes in [
        b"secret-provider-error".to_vec(),
        vec![b'x'; 65537],
        String::from_utf8(bytes.to_vec())
            .unwrap()
            .replace("Prisma AIRS Harness", "Unknown")
            .into_bytes(),
    ] {
        let error = process::parse(&bytes).unwrap_err();
        assert_eq!(
            error,
            "Could not read diagnostics. Retry /doctor or run airs doctor in this environment."
        );
    }
    assert_eq!(display("safe\x1b\n\u{202e}text"), "safetext");
}

#[test]
fn verify_requires_an_explicit_action_and_cancel_keeps_attempt_identity() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let tx = crate::app_event_sender::AppEventSender::new(tx);
    let view = views::overview(
        "work",
        Err("Offline".into()),
        Vec::new(),
        /*thread*/ None,
    );
    assert!(rx.try_recv().is_err());
    (view.items[0].actions[0])(&tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        crate::app_event::AppEvent::AirsDoctor(Event::Open(Mode::Inspect))
    ));
    (view.items[1].actions[0])(&tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        crate::app_event::AppEvent::AirsDoctor(Event::ConfirmVerify)
    ));
    let consent = views::confirm_verify();
    (consent.items[0].actions[0])(&tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        crate::app_event::AppEvent::AirsDoctor(Event::Open(Mode::Verify))
    ));
    let waiting = views::waiting(42);
    (waiting.on_cancel.unwrap())(&tx);
    assert!(matches!(
        rx.try_recv().unwrap(),
        crate::app_event::AppEvent::AirsDoctor(Event::Cancel(42))
    ));
    assert!(rx.try_recv().is_err());
}
