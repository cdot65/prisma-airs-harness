use super::*;
use clap::Parser;
use pretty_assertions::assert_eq;
use std::io::Cursor;

#[test]
fn guided_creation_is_parseable_and_keeps_the_million_token_default() {
    let cli = crate::MultitoolCli::try_parse_from(["airs-harness", "env", "create"]).unwrap();
    let Some(crate::Subcommand::Env {
        command: Some(crate::airs_environment::Command::Create { args, .. }),
    }) = cli.subcommand
    else {
        panic!("expected env create command");
    };
    assert_eq!(
        (args.gateway_url, args.context_window, args.credential_env),
        (String::new(), 1_000_000, "AIRS_API_KEY".to_owned())
    );
}

#[test]
fn guided_setup_uses_default_name_without_collecting_credentials() {
    let mut output = Vec::new();
    let (name, args) = collect(
        /*requested_name*/ None,
        &SetupArgs::default(),
        &mut Cursor::new("\nhttps://gateway.example/v1\n"),
        &mut output,
    )
    .unwrap();
    assert_eq!(
        (name, args.gateway_url, args.context_window),
        (
            "work".to_owned(),
            "https://gateway.example/v1".to_owned(),
            1_000_000
        )
    );
    insta::assert_snapshot!(String::from_utf8(output).unwrap(), @r"
    Welcome to Prisma AIRS Harness

    Connect to your organization's AI Gateway, then sign in.
    Ask your administrator for the gateway URL if you don't have it.
    Environment name [work]: AI Gateway URL: 
    ");
}

#[test]
fn requested_name_and_explicit_capabilities_survive_the_wizard() {
    let template = SetupArgs {
        context_window: 128_000,
        model: vec!["@provider/model".to_owned()],
        ..Default::default()
    };
    let (name, args) = collect(
        Some("company"),
        &template,
        &mut Cursor::new("https://gateway.example/v1\n"),
        &mut Vec::new(),
    )
    .unwrap();
    assert_eq!(
        (name, args.model, args.context_window),
        ("company".to_owned(), template.model, 128_000)
    );
}

#[test]
fn invalid_or_cancelled_setup_input_never_produces_a_configuration() {
    for input in [
        "",
        "work\n",
        "work\n\n",
        "bad/name\nhttps://gateway.example/v1\n",
        "work\nhttps://gateway.example/\x1b[2J\n",
    ] {
        assert!(
            collect(
                /*requested_name*/ None,
                &SetupArgs::default(),
                &mut Cursor::new(input),
                &mut Vec::new(),
            )
            .is_err()
        );
    }
}

#[test]
fn wizard_output_cannot_overwrite_an_existing_named_environment() {
    let root = tempfile::tempdir().unwrap();
    let (name, args) = collect(
        Some("work"),
        &SetupArgs::default(),
        &mut Cursor::new("https://gateway.example/v1\n"),
        &mut Vec::new(),
    )
    .unwrap();
    airs_environment::setup(root.path(), &name, &args).unwrap();
    let original = std::fs::read(root.path().join("environments.json")).unwrap();
    let changed = SetupArgs {
        gateway_url: "https://different.example/v1".to_owned(),
        ..args
    };
    assert!(airs_environment::setup(root.path(), &name, &changed).is_err());
    assert_eq!(
        std::fs::read(root.path().join("environments.json")).unwrap(),
        original
    );
}

#[test]
fn gateway_validation_rejects_controls_even_in_explicit_setup() {
    let home = tempfile::tempdir().unwrap();
    for gateway_url in [
        "",
        "https://gateway.example/\npath",
        "https://gateway.example/\x1b[2J",
    ] {
        let args = SetupArgs {
            gateway_url: gateway_url.to_owned(),
            ..Default::default()
        };
        assert!(super::super::airs_harness::setup_in(&args, home.path()).is_err());
        assert!(!home.path().join("config.toml").exists());
    }
}
