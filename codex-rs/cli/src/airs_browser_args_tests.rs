use super::*;
use clap::Parser;

#[test]
fn manual_browser_requires_company_identity_and_excludes_device_flow() {
    let identity = [
        "--issuer-url",
        "https://auth.example/realms/company",
        "--oidc-client-id",
        "harness",
        "--audience",
        "resource",
    ];
    for prefix in [
        vec!["airs-harness", "login"],
        vec![
            "airs-harness",
            "setup-mcp",
            "--name",
            "airs",
            "--url",
            "https://mcp.example/mcp",
        ],
    ] {
        let mut args = prefix.clone();
        args.extend(identity);
        args.push("--no-browser");
        assert!(MultitoolCli::try_parse_from(&args).is_ok());
        args.push("--device-auth");
        assert!(MultitoolCli::try_parse_from(&args).is_err());
        let mut missing_identity = prefix;
        missing_identity.push("--no-browser");
        assert!(MultitoolCli::try_parse_from(missing_identity).is_err());
    }
}
