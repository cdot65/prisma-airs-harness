use super::*;

#[test]
fn airs_header_shows_environment_auth_gateway_and_permissions() {
    let mut cell = SessionHeaderHistoryCell::new(
        "airs-gateway-default".into(),
        Some(ReasoningEffortConfig::High),
        /*show_fast_status*/ true,
        PathBuf::from("/workspace/project"),
        "upstream-version",
    )
    .with_airs_context(crate::airs_branding::HeaderContext {
        environment: "work".into(),
        gateway: "https://airs.example/v1".into(),
        identity: "workspace credential".into(),
        permissions: "workspace-write".into(),
    });
    cell.version = "0.0.0";
    let render = |width| {
        cell.display_lines(width)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    };
    insta::assert_snapshot!("airs_header_wide", render(80));
    insta::assert_snapshot!("airs_header_narrow", render(42));
    let raw = cell
        .raw_lines()
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("airs_header_transcript", raw);
}
