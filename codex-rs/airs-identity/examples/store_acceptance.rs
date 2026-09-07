//! Run phases in separate processes against an isolated native OS store.
use codex_airs_identity::CredentialStore;
use codex_keyring_store::KeyringStore;

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
        _ => anyhow::bail!("unknown phase"),
    }
    println!("{}", serde_json::json!({"phase": args[1], "passed": true}));
    Ok(())
}
