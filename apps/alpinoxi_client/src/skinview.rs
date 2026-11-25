//! Skin rendering and the wardrobe's model.
//!
//! Two jobs:
//!
//! 1. Turn a skin PNG into a flat pixel buffer (`SkinImage`), which the UI then
//!    uploads as one texture. One upload per skin instead of one per face keeps
//!    the launcher's memory flat.
//! 2. Describe what the wardrobe shows: the chosen skin, its model, and the
//!    honesty note about who can actually see it.

use alpinoxi_core::account::AccountProvider;
use alpinoxi_core::skin::{Skin, SkinModel, SkinSource, SkinValidationError};

/// A decoded skin, ready for `Context::load_texture`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkinImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl SkinImage {
    pub fn decode(bytes: &[u8]) -> Result<Self, SkinValidationError> {
        let (width, height) = Skin::validate_png(bytes)?;
        let image = image::load_from_memory(bytes).map_err(|_| SkinValidationError::NotPng)?;
        let rgba = image.to_rgba8();
        if rgba.width() != width || rgba.height() != height {
            return Err(SkinValidationError::BadDimensions(
                rgba.width(),
                rgba.height(),
            ));
        }
        Ok(Self {
            width,
            height,
            rgba: rgba.into_raw(),
        })
    }

    fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);
        let index = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[index],
            self.rgba[index + 1],
            self.rgba[index + 2],
            self.rgba[index + 3],
        ]
    }

    /// Builds a front-facing portrait from the standard skin layout.
    ///
    /// Reads the face, hat overlay, body and arms at their fixed UVs, so the
    /// preview matches what the game shows instead of a generic avatar.
    pub fn front_portrait(&self, model: SkinModel) -> SkinImage {
        // Output grid: 16 wide, 32 tall in skin pixels.
        let mut out = vec![0_u8; 16 * 32 * 4];
        let mut put = |x: u32, y: u32, colour: [u8; 4]| {
            if x < 16 && y < 32 {
                let index = ((y * 16 + x) * 4) as usize;
                out[index..index + 4].copy_from_slice(&colour);
            }
        };

        let blend = |base: [u8; 4], overlay: [u8; 4]| -> [u8; 4] {
            if overlay[3] == 0 {
                return base;
            }
            let alpha = u16::from(overlay[3]);
            let mix = |a: u8, b: u8| -> u8 {
                (((u16::from(a) * (255 - alpha)) + (u16::from(b) * alpha)) / 255) as u8
            };
            [
                mix(base[0], overlay[0]),
                mix(base[1], overlay[1]),
                mix(base[2], overlay[2]),
                255,
            ]
        };

        // Head: base at (8,8), hat overlay at (40,8), 8x8 each.
        for y in 0..8 {
            for x in 0..8 {
                let base = self.pixel(8 + x, 8 + y);
                let hat = self.pixel(40 + x, 8 + y);
                put(x + 4, y, blend(base, hat));
            }
        }

        // Torso: base (20,20), overlay (20,36), 8x12.
        for y in 0..12 {
            for x in 0..8 {
                let base = self.pixel(20 + x, 20 + y);
                let jacket = if self.height == 64 {
                    self.pixel(20 + x, 36 + y)
                } else {
                    [0, 0, 0, 0]
                };
                put(x + 4, y + 8, blend(base, jacket));
            }
        }

        // Arms: 4 wide for classic, 3 for slim; right arm base at (44,20).
        let arm_width = match model {
            SkinModel::Classic => 4,
            SkinModel::Slim => 3,
        };
        for y in 0..12 {
            for x in 0..arm_width {
                let base = self.pixel(44 + x, 20 + y);
                let sleeve = if self.height == 64 {
                    self.pixel(44 + x, 36 + y)
                } else {
                    [0, 0, 0, 0]
                };
                let colour = blend(base, sleeve);
                put(3 - arm_width + x, y + 8, colour);
                put(12 + x, y + 8, colour);
            }
        }

        // Legs: base (4,20), 4x12.
        for y in 0..12 {
            for x in 0..4 {
                let base = self.pixel(4 + x, 20 + y);
                let trouser = if self.height == 64 {
                    self.pixel(4 + x, 36 + y)
                } else {
                    [0, 0, 0, 0]
                };
                let colour = blend(base, trouser);
                put(x + 4, y + 20, colour);
            }
        }

        SkinImage {
            width: 16,
            height: 32,
            rgba: out,
        }
    }
}

/// What the wardrobe currently shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wardrobe {
    pub provider: AccountProvider,
    pub name: String,
    pub model: SkinModel,
    pub skin: Option<Skin>,
    pub status: String,
}

impl Wardrobe {
    pub fn new(provider: AccountProvider, name: &str, model: SkinModel) -> Self {
        Self {
            provider,
            name: name.to_owned(),
            model,
            skin: None,
            status: String::new(),
        }
    }

    /// Uses the provider's own skin when there is one, otherwise the local file.
    pub fn resolve(&mut self, local: Option<Skin>) {
        let source = SkinSource::for_provider(self.provider);
        self.skin = match self.provider {
            AccountProvider::Microsoft | AccountProvider::ElyBy => {
                let location = Skin::default_location(self.provider, &self.name);
                Some(Skin::new(location, self.model, source))
            }
            AccountProvider::Offline => local.filter(|skin| skin.source == SkinSource::Local),
        };
        self.status = match (&self.skin, self.provider) {
            (Some(skin), _) => skin.visibility_note().to_owned(),
            (None, AccountProvider::Offline) => {
                "Pick a 64x64 PNG to use your own skin locally.".to_owned()
            }
            (None, _) => "Sign in to load your skin.".to_owned(),
        };
    }

    /// Applies a local file after checking it really is a skin.
    pub fn apply_local(&mut self, bytes: &[u8], path: &str) -> Result<(), SkinValidationError> {
        SkinImage::decode(bytes)?;
        let mut skin = Skin::new(path, self.model, SkinSource::Local);
        skin.cape = None;
        self.status = skin.visibility_note().to_owned();
        self.skin = Some(skin);
        Ok(())
    }

    /// The upload target for this provider: Ely.by needs its own endpoint,
    /// Microsoft goes through the Minecraft services API.
    pub fn upload_endpoint(&self) -> &'static str {
        match self.provider {
            AccountProvider::ElyBy => "https://account.ely.by/api/account/v1/skin",
            AccountProvider::Microsoft => "https://api.minecraftservices.com/minecraft/profile/skins",
            AccountProvider::Offline => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid_skin(width: u32, height: u32, colour: [u8; 4]) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0]);
        let mut image = image::RgbaImage::new(width, height);
        for pixel in image.pixels_mut() {
            *pixel = image::Rgba(colour);
        }
        let mut cursor = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        bytes.extend_from_slice(&cursor.into_inner()[8..]);
        bytes
    }

    #[test]
    fn portrait_is_sixteen_by_thirty_two() {
        let bytes = solid_skin(64, 64, [200, 40, 40, 255]);
        let skin = SkinImage::decode(&bytes).unwrap();
        let portrait = skin.front_portrait(SkinModel::Classic);
        assert_eq!((portrait.width, portrait.height), (16, 32));
        assert_eq!(portrait.rgba.len(), 16 * 32 * 4);
        // Head pixel should carry the skin colour.
        assert_eq!(&portrait.rgba[(4 * 4)..(4 * 4) + 3], &[200, 40, 40]);
    }

    #[test]
    fn slim_arms_are_narrower_than_classic() {
        let bytes = solid_skin(64, 64, [20, 200, 40, 255]);
        let skin = SkinImage::decode(&bytes).unwrap();
        let classic = skin.front_portrait(SkinModel::Classic);
        let slim = skin.front_portrait(SkinModel::Slim);
        // Leftmost arm column: filled for classic, empty for slim.
        assert_ne!(classic.rgba[3], 0);
        assert_eq!(slim.rgba[3], 0);
    }

    #[test]
    fn wardrobe_explains_local_skins() {
        let mut wardrobe = Wardrobe::new(AccountProvider::Offline, "Alpinoxi", SkinModel::Classic);
        wardrobe.resolve(None);
        assert!(wardrobe.status.contains("Pick a 64x64 PNG"));
        let bytes = solid_skin(64, 64, [1, 2, 3, 255]);
        wardrobe.apply_local(&bytes, "C:/skins/me.png").unwrap();
        assert!(wardrobe.status.contains("Local skin"));
    }

    #[test]
    fn upload_endpoint_follows_provider() {
        let ely = Wardrobe::new(AccountProvider::ElyBy, "Erick", SkinModel::Slim);
        assert!(ely.upload_endpoint().contains("ely.by"));
        let ms = Wardrobe::new(AccountProvider::Microsoft, "Notch", SkinModel::Classic);
        assert!(ms.upload_endpoint().contains("minecraftservices"));
    }
}
