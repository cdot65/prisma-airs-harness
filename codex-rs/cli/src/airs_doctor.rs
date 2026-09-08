//! Bounded, redacted diagnostics for the standalone gateway environment.
use anyhow::Context;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Serialize)]
struct Check {
    name: &'static str,
    passed: bool,
    detail: String,
}

pub async fn run(home: &Path, args: &super::doctor::DoctorCommand) -> anyhow::Result<()> {
    let mut checks = Vec::new();
    let tools = if cfg!(target_os = "linux") {
        &["sh", "git", "rg", "bwrap"][..]
    } else {
        &["git", "rg"][..]
    };
    let missing: Vec<_> = tools
        .iter()
        .filter(|tool| which::which(tool).is_err())
        .copied()
        .collect();
    checks.push(Check {
        name: "local_tools",
        passed: missing.is_empty(),
        detail: if missing.is_empty() {
            "Required local executables found; this does not exercise kernel sandbox support".into()
        } else {
            format!("Install missing local executables: {}", missing.join(", "))
        },
    });
    #[cfg(target_os = "linux")]
    {
        let sandbox = async {
            let executable = which::which("bwrap")?;
            let status = tokio::process::Command::new(executable)
                .args([
                    "--die-with-parent",
                    "--unshare-user",
                    "--unshare-pid",
                    "--ro-bind",
                    "/",
                    "/",
                    "--",
                    "/bin/true",
                ])
                .env_clear()
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true)
                .status()
                .await?;
            Ok::<bool, anyhow::Error>(status.success())
        };
        let passed = matches!(
            tokio::time::timeout(Duration::from_secs(3), sandbox).await,
            Ok(Ok(true))
        );
        checks.push(Check {
            name: "linux_user_namespaces",
            passed,
            detail: if passed {
                "Bubblewrap created user/PID namespaces; full workspace policy still applies at execution".into()
            } else {
                "Bubblewrap could not create required namespaces; check host/kernel/container policy. No unsandboxed fallback".into()
            },
        });
    }
    let configuration = super::airs_status::configuration(home);
    match configuration {
        Ok((gateway, config)) => {
            checks.push(Check { name: "configuration", passed: true, detail: gateway.clone() });
            if !args.verify_access {
                let credential = super::airs_status::inspect(home);
                checks.push(Check {
                    name: "credential_configuration",
                    passed: credential.is_ok(),
                    detail: match credential {
                        Ok(inspection) => inspection.detail.into(),
                        Err(error) => error.to_string(),
                    },
                });
            }
            // --verify-access resolves credentials only through the bounded
            // helper below. A second native read here could prompt or block
            // before that deadline starts; gateway_access reports its result.
            let capabilities = (|| -> anyhow::Result<()> {
                let path = config.get("model_catalog_json").and_then(toml::Value::as_str)
                    .context("missing model capability catalog")?;
                let _: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
                anyhow::ensure!(config.get("model_context_window").and_then(toml::Value::as_integer).is_some_and(|v| v > 0), "invalid context window");
                Ok(())
            })();
            checks.push(Check {
                name: "capabilities", passed: capabilities.is_ok(),
                detail: match capabilities { Ok(()) => "Local catalog readable; backend limits still apply".into(), Err(error) => error.to_string() },
            });
            // No credential or conversation is sent by this reachability probe.
            let health = (async {
                let endpoint = url::Url::parse(&format!("{}/health", gateway.trim_end_matches('/')))?;
                let response = codex_login::default_client::create_client_without_request_logging()
                    .get(endpoint.as_str()).timeout(Duration::from_secs(8)).send().await?;
                anyhow::ensure!(response.status().is_success(), "health endpoint returned HTTP {}", response.status().as_u16());
                Ok::<(), anyhow::Error>(())
            }).await;
            checks.push(Check {
                name: "gateway_health", passed: health.is_ok(),
                detail: match health { Ok(()) => "HTTPS/HTTP health response successful; inference and policy not exercised".into(), Err(_) => "Health probe failed; check gateway DNS, TLS and its API-root health endpoint".into() },
            });
            let mcp_count = config.get("mcp_servers").and_then(toml::Value::as_table).map_or(0, toml::map::Map::len);
            checks.push(Check { name: "mcp_configuration", passed: true, detail: format!("{mcp_count} configured server(s); use /mcp and invoke a tool to verify remote authorization") });
        }
        Err(_) => checks.push(Check { name: "configuration", passed: false, detail: "Cannot read environment configuration; run setup or select a configured environment".into() }),
    }
    if args.verify_access {
        eprintln!("{}", super::airs_access::DISCLOSURE);
        let access = super::airs_access::verify(home).await;
        checks.push(Check {
            name: "gateway_access",
            passed: access.outcome.is_ok(),
            detail: access.summary(),
        });
    }
    let passed = checks.iter().all(|check| check.passed);
    let report = serde_json::json!({
        "schema_version": 1, "product": "Prisma AIRS Harness",
        "version": super::airs_harness::version(), "state_directory": home,
        "passed": passed, "checks": checks,
    });
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "Prisma AIRS Harness {} — doctor",
            super::airs_harness::version()
        );
        println!("State: {}", home.display());
        for check in checks {
            let status = if check.passed { "PASS" } else { "FAIL" };
            println!("{status} {}: {}", check.name, check.detail);
        }
    }
    anyhow::ensure!(passed, "one or more AIRS Harness checks failed");
    Ok(())
}
