//! Run phases in separate processes against an isolated native OS store.
use codex_airs_identity::CredentialStore;
use codex_airs_identity::WorkspaceCredentialFormat;
use codex_airs_identity::WorkspaceCredentialStore;
use codex_keyring_store::DefaultKeyringStore;
use codex_keyring_store::KeyringStore;
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    anyhow::ensure!(args.len() == 3, "usage: store_acceptance PHASE ACCOUNT");
    let account = &args[2];
    let service = "io.cdot.airs-terminal.acceptance";
    let value = "synthetic token bundle · 🔑".repeat(1000);
    match args[1].as_str() {
        "write" => CredentialStore.save(service, account, &value)?,
        "read-and-pend" => {
            anyhow::ensure!(
                CredentialStore.load(service, account)? == Some(value),
                "stored bundle changed"
            );
            CredentialStore.save(service, account, "refresh-pending")?;
        }
        "read-and-delete" => {
            anyhow::ensure!(
                CredentialStore.load(service, account)?.as_deref() == Some("refresh-pending"),
                "pending state lost"
            );
            CredentialStore.delete(service, account)?;
            anyhow::ensure!(
                CredentialStore.load(service, account)?.is_none(),
                "credential remains after logout"
            );
        }
        "workspace-legacy-write" => {
            DefaultKeyringStore.save(service, account, "synthetic-legacy-workspace-key")?;
        }
        "workspace-read-and-write-v2" => {
            let account = Uuid::parse_str(account)?;
            anyhow::ensure!(
                WorkspaceCredentialStore
                    .load(service, WorkspaceCredentialFormat::LegacyRaw, account)?
                    .as_deref()
                    == Some("synthetic-legacy-workspace-key"),
                "legacy workspace key changed"
            );
            anyhow::ensure!(
                WorkspaceCredentialStore
                    .load(service, WorkspaceCredentialFormat::ChunkedV2, account)?
                    .is_none(),
                "V2 read fell back to legacy key"
            );
            WorkspaceCredentialStore.save_new(service, account, &"A".repeat(16_384))?;
        }
        "workspace-read-both-delete-legacy" => {
            let account = Uuid::parse_str(account)?;
            anyhow::ensure!(
                WorkspaceCredentialStore
                    .load(service, WorkspaceCredentialFormat::LegacyRaw, account)?
                    .as_deref()
                    == Some("synthetic-legacy-workspace-key"),
                "V2 write replaced legacy workspace key"
            );
            anyhow::ensure!(
                WorkspaceCredentialStore.load(
                    service,
                    WorkspaceCredentialFormat::ChunkedV2,
                    account
                )? == Some("A".repeat(16_384)),
                "16 KiB workspace key changed"
            );
            WorkspaceCredentialStore.delete(
                service,
                WorkspaceCredentialFormat::LegacyRaw,
                account,
            )?;
        }
        "workspace-read-and-delete-v2" => {
            let account = Uuid::parse_str(account)?;
            anyhow::ensure!(
                WorkspaceCredentialStore
                    .load(service, WorkspaceCredentialFormat::LegacyRaw, account)?
                    .is_none(),
                "legacy workspace key remains after deletion"
            );
            anyhow::ensure!(
                WorkspaceCredentialStore.load(
                    service,
                    WorkspaceCredentialFormat::ChunkedV2,
                    account
                )? == Some("A".repeat(16_384)),
                "legacy deletion damaged V2 workspace key"
            );
            WorkspaceCredentialStore.delete(
                service,
                WorkspaceCredentialFormat::ChunkedV2,
                account,
            )?;
            anyhow::ensure!(
                WorkspaceCredentialStore
                    .load(service, WorkspaceCredentialFormat::ChunkedV2, account)?
                    .is_none(),
                "V2 workspace key remains after deletion"
            );
        }
        _ => anyhow::bail!("unknown phase"),
    }
    println!("{}", serde_json::json!({"phase": args[1], "passed": true}));
    Ok(())
}
