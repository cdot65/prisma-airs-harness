use super::*;
use clap::Parser;

#[test]
fn manual_company_browser_keeps_identity_and_device_constraints() {
    let args = [
        "airs-harness",
        "login",
        "--no-browser",
        "--issuer-url",
        "https://idp.example/realms/company",
        "--oidc-client-id",
        "native",
        "--audience",
        "inference",
    ];
    assert!(MultitoolCli::try_parse_from(args).is_ok());
    assert!(MultitoolCli::try_parse_from(args.into_iter().chain(["--device-auth"])).is_err());
    assert!(MultitoolCli::try_parse_from(["airs-harness", "login", "--no-browser"]).is_err());
}

#[test]
fn builtin_mcp_add_parses_scoped_manual_oauth_onboarding() {
    let cli = mcp_cmd::McpCli::try_parse_from([
        "mcp",
        "add",
        "airs",
        "--url",
        "https://mcp.example/mcp",
        "--oauth-client-id",
        "native-mcp",
        "--scopes",
        "gateway.read,profiles.read",
        "--no-browser",
    ])
    .unwrap();
    let mcp_cmd::McpSubcommand::Add(args) = cli.subcommand else {
        panic!("expected add")
    };
    assert!(args.no_browser);
    let transport = args.transport_args.streamable_http.unwrap();
    assert_eq!(transport.scopes, vec!["gateway.read", "profiles.read"]);
    assert_eq!(transport.oauth_client_id.as_deref(), Some("native-mcp"));
    assert!(transport.oauth_resource.is_none());
}
