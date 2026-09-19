# Primary failure lost when cleanup also fails

Evidence classification: source-level control-flow finding; no Rust test was run for this inventory.
Source `a826d22a63d9d6889157477a0109e242871796f5`, `codex-rs/cli/src/airs_credential_transaction.rs`, SHA256 `7c5552170c2666adbe46025cc8ed1df01f8dc3635db0f4de2a58224f17100ab2`.

```rust
258:     let result = (|| {
259:         store.save(kind, binding.id, token)?;
260:         anyhow::ensure!(
261:             store.load(kind, binding.id)?.as_deref() == Some(token),
262:             "Credential storage verification failed; sign-in was not completed"
263:         );
264:         if let Err(error) = install() {
265:             // Restore both files even if the first restore fails. Snapshot bytes
266:             // stay in memory; the durable cleanup journal never copies secrets.
267:             let config_restore =
268:                 restore_file(&home.join("config.toml"), previous_config.as_deref());
269:             let binding_restore =
270:                 restore_file(&home.join("credential-binding.json"), previous.as_deref());
271:             config_restore.context("Credential configuration rollback failed; retry login after resolving the filesystem error")?;
272:             binding_restore.context("Credential binding rollback failed; retry login after resolving the filesystem error")?;
273:             return Err(error);
274:         }
275:         Ok(())
276:     })();
277:     if !already_owned {
278:         recover(home, store)?;
279:     }
280:     result
```

`result` retains the first save/read/configuration/activation failure. For a newly owned account, the following `recover(home, store)?` immediately returns its own error if deletion fails, so the earlier error never reaches `result` at the end. This directly explains why a pending-cleanup diagnostic can conceal whether initial saving or verification failed. It does not establish why the owner's native service was unavailable.

Existing `failed_cleanup_retains_only_account_metadata_and_can_retry_in_another_call` enables both fake read and delete failures but asserts only `is_err()` plus journal safety/recovery. It does not assert preservation of the initiating failure. `failed_oidc_cleanup_retries_only_the_recorded_native_format` likewise covers namespace safety, not error composition.

Proposed failing regression: fake read failure + fake delete failure must report both bounded operation contexts, preserve the nonsecret journal and prior configuration, retain typed `CredentialRecovery` detection, and never render credential/backend canaries. Follow with retry restoring availability and proving only the recorded unowned account is deleted. Do not alter journal schema or fail-closed ownership to improve the displayed result.
