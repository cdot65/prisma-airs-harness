//! Bounded token bundles with an atomic OS-store manifest commit.
//! Each hex chunk fits Windows' 2,560-byte UTF-16 credential limit.
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::KeyringStore;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use uuid::Uuid;

const CHUNK_BYTES: usize = 512;
const MAX_BYTES: usize = 131_072;

#[path = "generation_journal.rs"]
mod generation_journal;

#[derive(Debug)]
pub(super) struct ChunkedStore<S>(pub(super) S);

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    generation: Uuid,
    chunks: usize,
    sha256: String,
}

fn invalid() -> CredentialStoreError {
    CredentialStoreError::new(keyring::Error::Invalid(
        "identity bundle".into(),
        "missing, corrupt or oversized record".into(),
    ))
}

impl Manifest {
    fn parse(raw: &str) -> Result<Self, CredentialStoreError> {
        if raw.len() > 1024 {
            return Err(invalid());
        }
        let value: Self = serde_json::from_str(raw).map_err(|_| invalid())?;
        // Version 1 is active; version 2 is a deleting tombstone with the same
        // bounded enumeration fields. Older readers reject version 2 safely.
        if !value.valid() {
            return Err(invalid());
        }
        Ok(value)
    }

    fn valid(&self) -> bool {
        matches!(self.version, 1 | 2)
            && (1..=MAX_BYTES / CHUNK_BYTES).contains(&self.chunks)
            && self.sha256.len() == 64
            && self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    fn account(&self, root: &str, index: usize) -> String {
        format!("{root}.{}.{}", self.generation, index)
    }
}

impl<S: KeyringStore> ChunkedStore<S> {
    fn read_generation(
        &self,
        service: &str,
        account: &str,
        manifest: &Manifest,
    ) -> Result<String, CredentialStoreError> {
        let mut bytes = Vec::new();
        for index in 0..manifest.chunks {
            let hex = self
                .0
                .load(service, &manifest.account(account, index))?
                .ok_or_else(invalid)?;
            if hex.is_empty()
                || hex.len() > CHUNK_BYTES * 2
                || hex.len() % 2 != 0
                || !hex.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(invalid());
            }
            for pair in hex.as_bytes().chunks_exact(2) {
                let pair = std::str::from_utf8(pair).map_err(|_| invalid())?;
                bytes.push(u8::from_str_radix(pair, 16).map_err(|_| invalid())?);
            }
        }
        if format!("{:x}", Sha256::digest(&bytes)) != manifest.sha256 {
            return Err(invalid());
        }
        String::from_utf8(bytes).map_err(|_| invalid())
    }
}

impl<S: KeyringStore> KeyringStore for ChunkedStore<S> {
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialStoreError> {
        let Some(raw) = self.0.load(service, account)? else {
            return Ok(None);
        };
        let manifest = Manifest::parse(&raw)?;
        if manifest.version == 2 {
            return Ok(None);
        }
        self.read_generation(service, account, &manifest).map(Some)
    }

    fn save(&self, service: &str, account: &str, value: &str) -> Result<(), CredentialStoreError> {
        if value.is_empty() || value.len() > MAX_BYTES {
            return Err(invalid());
        }
        self.recover_generations(service, account)?;
        let mut previous = self
            .0
            .load(service, account)?
            .map(|raw| Manifest::parse(&raw))
            .transpose()?;
        if previous
            .as_ref()
            .is_some_and(|manifest| manifest.version == 2)
        {
            // Never overwrite the only enumeration record for pending cleanup.
            self.delete(service, account)?;
            previous = None;
        }
        let manifest = Manifest {
            version: 1,
            generation: Uuid::new_v4(),
            chunks: value.len().div_ceil(CHUNK_BYTES),
            sha256: format!("{:x}", Sha256::digest(value.as_bytes())),
        };
        self.journal_generations(service, account, previous.as_ref(), &manifest)?;
        for (index, chunk) in value.as_bytes().chunks(CHUNK_BYTES).enumerate() {
            let hex: String = chunk.iter().map(|byte| format!("{byte:02x}")).collect();
            self.0
                .save(service, &manifest.account(account, index), &hex)?;
        }
        if self.read_generation(service, account, &manifest)? != value {
            return Err(invalid());
        }
        let raw = serde_json::to_string(&manifest).map_err(|_| invalid())?;
        // A failed/ambiguous commit leaves both generations journaled. Recovery
        // checks the actual root, never guesses whether this write committed.
        self.0.save(service, account, &raw)?;
        if self.0.load(service, account)?.as_deref() != Some(raw.as_str()) {
            return Err(invalid());
        }
        self.recover_generations(service, account)
    }

    fn delete(&self, service: &str, account: &str) -> Result<bool, CredentialStoreError> {
        let Some(raw) = self.0.load(service, account)? else {
            self.recover_generations(service, account)?;
            return Ok(false);
        };
        // Parse before modifying anything: corrupt metadata may be the last
        // evidence available for repair, and must not be silently destroyed.
        let mut manifest = Manifest::parse(&raw)?;
        if manifest.version == 1 {
            manifest.version = 2;
            let tombstone = serde_json::to_string(&manifest).map_err(|_| invalid())?;
            self.0.save(service, account, &tombstone)?;
        }
        self.recover_generations(service, account)?;
        for index in 0..manifest.chunks {
            self.0.delete(service, &manifest.account(account, index))?;
        }
        // Keep the tombstone until every chunk deletion succeeded. Missing
        // chunks are harmless when retrying after an interrupted operation.
        self.0.delete(service, account)?;
        Ok(true)
    }
}

#[cfg(test)]
#[path = "chunks_tests.rs"]
mod tests;
