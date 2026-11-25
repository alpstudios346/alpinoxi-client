//! Where long-lived tokens live.
//!
//! The rule from `docs/AUTHENTICATION.md`: long-lived secrets go to the OS
//! credential store, never to a plain JSON file in the launcher folder. This
//! module defines that boundary. `MemoryVault` is the in-process implementation
//! used by tests and by the UI before a platform backend is wired up.

use std::collections::BTreeMap;

/// Platform secret storage.
pub trait TokenVault {
    fn store(&mut self, key: &str, secret: &str) -> Result<(), VaultError>;
    fn load(&self, key: &str) -> Result<Option<String>, VaultError>;
    fn forget(&mut self, key: &str) -> Result<(), VaultError>;
    /// Lists the keys held, never the values.
    fn keys(&self) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VaultError {
    #[error("the operating system refused access to the credential store")]
    Unavailable,
    #[error("the credential store is locked")]
    Locked,
}

/// Vault key for a provider's refresh token, e.g. `alpinoxi.elyby.refresh`.
pub fn refresh_key(provider: &str) -> String {
    format!("alpinoxi.{provider}.refresh")
}

/// In-memory vault. Everything is lost when the process exits, which is exactly
/// what you want for tests and for a session that must not persist tokens.
#[derive(Debug, Clone, Default)]
pub struct MemoryVault {
    entries: BTreeMap<String, String>,
}

impl MemoryVault {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenVault for MemoryVault {
    fn store(&mut self, key: &str, secret: &str) -> Result<(), VaultError> {
        self.entries.insert(key.to_owned(), secret.to_owned());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<String>, VaultError> {
        Ok(self.entries.get(key).cloned())
    }

    fn forget(&mut self, key: &str) -> Result<(), VaultError> {
        self.entries.remove(key);
        Ok(())
    }

    fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

impl std::fmt::Debug for MemoryVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the secrets, only how many accounts are remembered.
        write!(f, "MemoryVault {{ {} entries }}", self.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_forgets() {
        let mut vault = MemoryVault::new();
        let key = refresh_key("elyby");
        vault.store(&key, "refresh-token").unwrap();
        assert_eq!(vault.load(&key).unwrap().as_deref(), Some("refresh-token"));
        assert_eq!(vault.keys(), vec![key.clone()]);
        vault.forget(&key).unwrap();
        assert_eq!(vault.load(&key).unwrap(), None);
    }

    #[test]
    fn debug_does_not_leak_secrets() {
        let mut vault = MemoryVault::new();
        vault.store("k", "SUPERSECRET").unwrap();
        assert!(!format!("{vault:?}").contains("SUPERSECRET"));
    }
}
