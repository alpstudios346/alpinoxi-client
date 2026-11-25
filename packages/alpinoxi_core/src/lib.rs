//! Domain types shared by the Alpinoxi launcher and its future services.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountProvider {
    Microsoft,
    ElyBy,
    Offline,
}

impl AccountProvider {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Microsoft => "Microsoft",
            Self::ElyBy => "Ely.by",
            Self::Offline => "Offline",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Microsoft => "Official Minecraft account sign-in",
            Self::ElyBy => "Ely.by account sign-in",
            Self::Offline => "Local profile for offline-mode servers",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountProfile {
    pub provider: AccountProvider,
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryProfile {
    Balanced,
    Performance,
    Custom(u16),
}

impl Default for MemoryProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl MemoryProfile {
    pub const fn mebibytes(self) -> u16 {
        match self {
            Self::Balanced => 2_048,
            Self::Performance => 4_096,
            Self::Custom(value) => value,
        }
    }

    pub fn validate(self) -> Result<(), MemoryProfileError> {
        let value = self.mebibytes();
        if !(512..=16_384).contains(&value) {
            return Err(MemoryProfileError::OutOfRange(value));
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryProfileError {
    #[error("memory limit {0} MiB must be between 512 MiB and 16384 MiB")]
    OutOfRange(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_memory_limits_bounded() {
        assert!(MemoryProfile::Custom(511).validate().is_err());
        assert!(MemoryProfile::Custom(4_096).validate().is_ok());
        assert!(MemoryProfile::Custom(16_385).validate().is_err());
    }
}
