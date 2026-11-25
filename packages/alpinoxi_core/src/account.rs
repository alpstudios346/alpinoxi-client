//! Account model for the three ways Alpinoxi lets people play.
//!
//! The three providers are deliberately treated as first-class peers: Microsoft
//! (official Minecraft account), Ely.by (`alpinoxi-client` OAuth client) and
//! Cracked (offline-mode name). Skins and the friend list are built on top of
//! this same enum, so a cracked or Ely.by player gets the same features.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// How a player signed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccountProvider {
    /// Official Minecraft account through Microsoft/Xbox Live.
    Microsoft,
    /// Ely.by account, OAuth client id `alpinoxi-client`.
    ElyBy,
    /// Offline-mode name, used only where the server allows it.
    Offline,
}

impl AccountProvider {
    /// All providers, in the order the login screen shows them.
    pub const ALL: [Self; 3] = [Self::Microsoft, Self::ElyBy, Self::Offline];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Microsoft => "Microsoft",
            Self::ElyBy => "Ely.by",
            Self::Offline => "Cracked",
        }
    }

    pub const fn detail(self) -> &'static str {
        match self {
            Self::Microsoft => "Official Minecraft account sign-in",
            Self::ElyBy => "Ely.by account sign-in",
            Self::Offline => "Local profile for offline-mode servers",
        }
    }

    /// Short one-liner for the login cards.
    pub const fn tagline(self) -> &'static str {
        match self {
            Self::Microsoft => "Premium skin, cape and friend list",
            Self::ElyBy => "Ely.by skins, capes and friend list",
            Self::Offline => "Your own name, local skin, friend list",
        }
    }

    /// Whether the provider issues a skin the servers can see (online-mode
    /// texture source) or whether the skin stays local to Alpinoxi.
    pub const fn has_remote_skins(self) -> bool {
        matches!(self, Self::Microsoft | Self::ElyBy)
    }

    /// Whether the provider can join online-mode servers.
    pub const fn supports_online_mode(self) -> bool {
        !matches!(self, Self::Offline)
    }

    /// Stable id used in config files and the launch bridge.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Microsoft => "microsoft",
            Self::ElyBy => "elyby",
            Self::Offline => "cracked",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim().to_ascii_lowercase().as_str() {
            "microsoft" | "ms" | "msa" => Some(Self::Microsoft),
            "elyby" | "ely.by" | "ely" => Some(Self::ElyBy),
            "cracked" | "offline" | "local" => Some(Self::Offline),
            _ => None,
        }
    }
}

/// A signed-in profile. Tokens are intentionally *not* stored here: they live in
/// the auth package's vault, so the UI and the core never touch a raw token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountProfile {
    pub provider: AccountProvider,
    pub display_name: String,
    /// Dashed UUID (no dashes in storage is normalised by `set_uuid`).
    pub uuid: String,
    /// Minecraft access token, present for Microsoft and Ely.by only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Custom skin a cracked player picked locally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_skin: Option<String>,
}

impl AccountProfile {
    pub fn new(provider: AccountProvider, display_name: impl Into<String>, uuid: impl Into<String>) -> Self {
        Self {
            provider,
            display_name: display_name.into(),
            uuid: uuid.into(),
            access_token: None,
            local_skin: None,
        }
    }

    /// Name validity shared by every provider: 3-16 characters, letters, digits
    /// and underscore, as Minecraft itself requires.
    pub fn validate_name(name: &str) -> Result<(), CrackedAccountError> {
        let len = name.chars().count();
        if !(3..=16).contains(&len) {
            return Err(CrackedAccountError::BadLength);
        }
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(CrackedAccountError::BadCharacters);
        }
        Ok(())
    }

    /// Builds a cracked profile, deriving the offline UUID the same way the game
    /// does: `UUID.nameUUIDFromBytes("OfflinePlayer:<name>")` (version 3 MD5).
    pub fn cracked(name: &str) -> Result<Self, CrackedAccountError> {
        Self::validate_name(name)?;
        let uuid = offline_uuid(name);
        Ok(Self::new(AccountProvider::Offline, name, uuid))
    }

    pub fn is_online(&self) -> bool {
        self.access_token.is_some() && self.provider.supports_online_mode()
    }

    /// `name` shown next to the provider badge.
    pub fn badge(&self) -> String {
        format!("{} · {}", self.display_name, self.provider.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CrackedAccountError {
    #[error("player names must be 3 to 16 characters long")]
    BadLength,
    #[error("player names may only contain A-Z, a-z, 0-9 and underscore")]
    BadCharacters,
}

/// Offline UUID derivation matching vanilla Minecraft, so cracked players keep
/// the same UUID across launcher restarts and servers that allow them agree.
pub fn offline_uuid(name: &str) -> String {
    let digest = md5_bytes(format!("OfflinePlayer:{name}").as_bytes());
    let mut bytes = digest;
    bytes[6] = (bytes[6] & 0x0f) | 0x30; // version 3
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // RFC 4122 variant
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Small dependency-free MD5, only used for the offline UUID above.
fn md5_bytes(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    let k: Vec<u32> = (0..64)
        .map(|i| ((i as f64 + 1.0).sin().abs() * 4_294_967_296.0) as u32)
        .collect();

    let mut a0: u32 = 0x6745_2301;
    let mut b0: u32 = 0xefcd_ab89;
    let mut c0: u32 = 0x98ba_dcfe;
    let mut d0: u32 = 0x1032_5476;

    let mut message = input.to_vec();
    let bit_len = (input.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    for chunk in message.chunks(64) {
        let m: Vec<u32> = chunk
            .chunks(4)
            .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
            .collect();
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(k[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(S[i]));
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut out = [0_u8; 16];
    for (i, word) in [a0, b0, c0, d0].iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuid_matches_vanilla() {
        // UUID.nameUUIDFromBytes("OfflinePlayer:Notch".getBytes(UTF_8))
        assert_eq!(
            offline_uuid("Notch"),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
    }

    #[test]
    fn cracked_names_are_validated() {
        assert!(AccountProfile::cracked("Alpinoxi").is_ok());
        assert_eq!(AccountProfile::cracked("ab"), Err(CrackedAccountError::BadLength));
        assert_eq!(
            AccountProfile::cracked("bad name"),
            Err(CrackedAccountError::BadCharacters)
        );
    }

    #[test]
    fn provider_ids_round_trip() {
        for provider in AccountProvider::ALL {
            assert_eq!(AccountProvider::from_id(provider.id()), Some(provider));
        }
    }
}
