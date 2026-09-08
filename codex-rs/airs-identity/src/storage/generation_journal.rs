//! Bounded OS-store metadata for interrupted generation writes and retirement.
use super::ChunkedStore;
use super::Manifest;
use super::invalid;
use codex_keyring_store::CredentialStoreError;
use codex_keyring_store::KeyringStore;
use serde::Deserialize;
use serde::Serialize;

const MAX_JOURNAL_BYTES: usize = 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    generations: Vec<Manifest>,
}

impl<S: KeyringStore> ChunkedStore<S> {
    pub(super) fn journal_generations(
        &self,
        service: &str,
        account: &str,
        previous: Option<&Manifest>,
        next: &Manifest,
    ) -> Result<(), CredentialStoreError> {
        let generations = previous
            .into_iter()
            .cloned()
            .chain(std::iter::once(next.clone()))
            .collect();
        let journal = Journal {
            version: 1,
            generations,
        };
        if journal.generations.len() == 2
            && journal.generations[0].generation == journal.generations[1].generation
        {
            return Err(invalid());
        }
        let raw = serde_json::to_string(&journal).map_err(|_| invalid())?;
        if raw.len() > MAX_JOURNAL_BYTES {
            return Err(invalid());
        }
        // The fixed suffix cannot collide with UUID/index chunk account names.
        // Metadata contains no credential bytes and fits Windows' UTF-16 limit.
        let journal_account = format!("{account}.pending-generations");
        self.0.save(service, &journal_account, &raw)?;
        if self.0.load(service, &journal_account)?.as_deref() != Some(raw.as_str()) {
            return Err(invalid());
        }
        Ok(())
    }

    pub(super) fn recover_generations(
        &self,
        service: &str,
        account: &str,
    ) -> Result<(), CredentialStoreError> {
        let journal_account = format!("{account}.pending-generations");
        let Some(raw) = self.0.load(service, &journal_account)? else {
            return Ok(());
        };
        if raw.len() > MAX_JOURNAL_BYTES {
            return Err(invalid());
        }
        let journal: Journal = serde_json::from_str(&raw).map_err(|_| invalid())?;
        if journal.version != 1
            || !(1..=2).contains(&journal.generations.len())
            || journal
                .generations
                .iter()
                .any(|generation| !generation.valid() || generation.version != 1)
            || (journal.generations.len() == 2
                && journal.generations[0].generation == journal.generations[1].generation)
        {
            return Err(invalid());
        }
        let current = self
            .0
            .load(service, account)?
            .map(|raw| Manifest::parse(&raw))
            .transpose()?;
        if let Some(current) = &current
            && current.version == 1
        {
            // Confirm the currently committed credential before retiring any
            // predecessor. Corruption never triggers fallback to an older token.
            self.read_generation(service, account, current)?;
        }
        for generation in journal.generations {
            // An older client can publish a different v1 generation while a
            // journal is pending. Its current generation must remain untouched.
            if current
                .as_ref()
                .is_some_and(|current| current.generation == generation.generation)
            {
                continue;
            }
            for index in 0..generation.chunks {
                self.0
                    .delete(service, &generation.account(account, index))?;
            }
        }
        self.0.delete(service, &journal_account)?;
        Ok(())
    }
}
