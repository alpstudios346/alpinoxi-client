//! Skin model shared by Microsoft, Ely.by and cracked profiles.
//!
//! A skin always has a *source*, so the wardrobe can tell a player exactly why a
//! skin is visible to other people (or not):
//!
//! * Microsoft and Ely.by skins are served by the account provider, so everyone
//!   on an online-mode server sees them.
//! * A cracked skin is a local PNG. Alpinoxi renders it for the player and can
//!   send it to Alpinoxi-aware servers, but plain offline servers keep showing
//!   the default skin. The UI says so instead of pretending otherwise.

use serde::{Deserialize, Serialize};

use crate::account::AccountProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinModel {
    /// Classic: 4-pixel arms.
    Classic,
    /// Slim: 3-pixel arms (Alex).
    Slim,
}

impl SkinModel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Slim => "Slim",
        }
    }

    /// Value written into the texture metadata `model` field.
    pub const fn metadata(self) -> Option<&'static str> {
        match self {
            Self::Classic => None,
            Self::Slim => Some("slim"),
        }
    }

    pub fn from_metadata(value: Option<&str>) -> Self {
        match value {
            Some("slim") => Self::Slim,
            _ => Self::Classic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinSource {
    /// `textures.minecraft.net`, from the Microsoft account.
    Microsoft,
    /// `skinsystem.ely.by`, from the Ely.by account.
    ElyBy,
    /// A PNG the player picked in the launcher.
    Local,
}

impl SkinSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Microsoft => "Microsoft account",
            Self::ElyBy => "Ely.by account",
            Self::Local => "Local file",
        }
    }

    /// Whether other players on a normal online-mode server can see this skin.
    pub const fn is_public(self) -> bool {
        !matches!(self, Self::Local)
    }

    /// The source a provider must use for the uploaded skin.
    pub const fn for_provider(provider: AccountProvider) -> Self {
        match provider {
            AccountProvider::Microsoft => Self::Microsoft,
            AccountProvider::ElyBy => Self::ElyBy,
            AccountProvider::Offline => Self::Local,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skin {
    /// Texture URL, or an absolute path for `SkinSource::Local`.
    pub location: String,
    pub model: SkinModel,
    pub source: SkinSource,
    /// Optional cape, same rules as the skin itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cape: Option<String>,
}

impl Skin {
    pub fn new(location: impl Into<String>, model: SkinModel, source: SkinSource) -> Self {
        Self {
            location: location.into(),
            model,
            source,
            cape: None,
        }
    }

    /// Where a player's skin is fetched from for each provider, without any
    /// network call: the wardrobe uses this to show a preview immediately.
    pub fn default_location(provider: AccountProvider, name: &str) -> String {
        match provider {
            AccountProvider::ElyBy => format!("http://skinsystem.ely.by/skins/{name}.png"),
            AccountProvider::Microsoft => {
                format!("https://mc-heads.net/skin/{name}")
            }
            AccountProvider::Offline => String::new(),
        }
    }

    /// One line of honesty for the wardrobe footer.
    pub fn visibility_note(&self) -> &'static str {
        match self.source {
            SkinSource::Microsoft => "Visible to everyone on online-mode servers.",
            SkinSource::ElyBy => "Visible to everyone on servers that accept Ely.by skins.",
            SkinSource::Local => {
                "Local skin: only Alpinoxi shows it. Offline servers keep the default skin."
            }
        }
    }

    /// Checks the two things that actually break a skin upload: the PNG
    /// signature and the pixel size.
    pub fn validate_png(bytes: &[u8]) -> Result<(u32, u32), SkinValidationError> {
        const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        if bytes.len() < 24 || bytes[..8] != PNG_SIGNATURE {
            return Err(SkinValidationError::NotPng);
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        match (width, height) {
            (64, 64) | (64, 32) => Ok((width, height)),
            _ => Err(SkinValidationError::BadDimensions(width, height)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SkinValidationError {
    #[error("the file is not a PNG image")]
    NotPng,
    #[error("skins must be 64x64 or 64x32 pixels, this one is {0}x{1}")]
    BadDimensions(u32, u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        bytes
    }

    #[test]
    fn accepts_both_skin_sizes() {
        assert_eq!(Skin::validate_png(&png_header(64, 64)), Ok((64, 64)));
        assert_eq!(Skin::validate_png(&png_header(64, 32)), Ok((64, 32)));
    }

    #[test]
    fn rejects_wrong_size_and_non_png() {
        assert_eq!(
            Skin::validate_png(&png_header(128, 128)),
            Err(SkinValidationError::BadDimensions(128, 128))
        );
        assert_eq!(Skin::validate_png(b"not an image"), Err(SkinValidationError::NotPng));
    }

    #[test]
    fn provider_picks_matching_source() {
        assert_eq!(
            SkinSource::for_provider(AccountProvider::ElyBy),
            SkinSource::ElyBy
        );
        assert_eq!(
            SkinSource::for_provider(AccountProvider::Offline),
            SkinSource::Local
        );
    }
}
