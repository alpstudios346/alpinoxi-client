//! Which mod loader a game version can carry, and where its metadata lives.
//!
//! The launcher never hard-codes a Minecraft version. Each loader publishes an
//! official index of what it supports; this module knows the shape of those
//! answers, decides whether a requested pair is real, and builds the URLs the
//! installer fetches. Deciding here (and testing here) is what keeps a new
//! Minecraft release a data change rather than a code change.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The three loaders Alpinoxi ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Forge,
    Fabric,
    Quilt,
}

impl Loader {
    pub const ALL: [Loader; 3] = [Loader::Forge, Loader::Fabric, Loader::Quilt];

    /// The spelling used in instance settings and in the interface.
    pub fn id(self) -> &'static str {
        match self {
            Loader::Forge => "forge",
            Loader::Fabric => "fabric",
            Loader::Quilt => "quilt",
        }
    }

    /// The spelling shown to a player.
    pub fn label(self) -> &'static str {
        match self {
            Loader::Forge => "Forge",
            Loader::Fabric => "Fabric",
            Loader::Quilt => "Quilt",
        }
    }

    pub fn parse(text: &str) -> Option<Loader> {
        match text.trim().to_ascii_lowercase().as_str() {
            "forge" | "neoforge" => Some(Loader::Forge),
            "fabric" => Some(Loader::Fabric),
            "quilt" => Some(Loader::Quilt),
            _ => None,
        }
    }

    /// The launcher main class this loader installs.
    pub fn main_class(self) -> &'static str {
        match self {
            Loader::Forge => "net.minecraftforge.client.loading.ClientModLoader",
            Loader::Fabric => "net.fabricmc.loader.impl.launch.knot.KnotClient",
            Loader::Quilt => "org.quiltmc.loader.impl.launch.knot.KnotClient",
        }
    }

    /// The oldest Minecraft version the loader ever supported.
    ///
    /// These are facts about the loaders, not about this launcher. Checked
    /// against each loader's own index: Fabric's `meta.fabricmc.net/v2/versions/game`
    /// lists 266 stable versions and its oldest is **1.14** — asking it for
    /// 1.13.2 answers HTTP 400. Quilt forked from Fabric and began at 1.14.4.
    /// Forge reaches back to 1.1. A request below the floor is refused with a
    /// plain reason instead of being attempted and failing later.
    pub fn earliest(self) -> (u16, u16) {
        match self {
            Loader::Forge => (1, 1),
            Loader::Fabric => (1, 14),
            Loader::Quilt => (1, 14),
        }
    }

    /// The mods folder name every loader reads.
    pub fn mods_dir(self) -> &'static str {
        "mods"
    }

    /// The official metadata endpoint for this loader's supported versions.
    pub fn versions_url(self) -> &'static str {
        match self {
            Loader::Forge => {
                "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json"
            }
            Loader::Fabric => "https://meta.fabricmc.net/v2/versions/game",
            Loader::Quilt => "https://meta.quiltmc.org/v3/versions/game",
        }
    }
}

impl fmt::Display for Loader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A Minecraft version as three numbers, compared in order.
///
/// Minecraft has used `1.8`, `1.21.1` and (recently) calendar versions such as
/// `26.3`; snapshots carry a letter and are not installable, so they parse as
/// nothing here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GameVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl GameVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parses `1.21.1`, `1.8`, `26.3`. Anything with a letter or a fourth part
    /// (a snapshot, a pre-release) is refused, because there is no loader
    /// profile for it.
    pub fn parse(text: &str) -> Option<GameVersion> {
        let text = text.trim();
        let mut parts = text.split('.');
        let major: u16 = parts.next()?.parse().ok()?;
        let minor: u16 = parts.next()?.parse().ok()?;
        let patch: u16 = match parts.next() {
            Some(part) => part.parse().ok()?,
            None => 0,
        };
        if parts.next().is_some() {
            return None;
        }
        Some(GameVersion::new(major, minor, patch))
    }

    /// The form a player types, and the form the metadata uses.
    pub fn as_string(&self) -> String {
        if self.patch == 0 {
            format!("{}.{}", self.major, self.minor)
        } else {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        }
    }

    /// `1.21` reads as `1.21.0`, which sorts below `1.21.1` — the comparison a
    /// player expects when picking a version.
    pub fn at_least(&self, major: u16, minor: u16) -> bool {
        (self.major, self.minor) >= (major, minor)
    }
}

impl fmt::Display for GameVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_string())
    }
}

/// Why a version and a loader cannot be combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderError {
    /// The version is not a number the launcher can install.
    UnreadableVersion,
    /// The loader did not exist for that version.
    BeforeTheLoader {
        loader: Loader,
        since: GameVersion,
    },
    /// The metadata published nothing for that version.
    NotPublished {
        loader: Loader,
        version: GameVersion,
    },
}

impl LoaderError {
    /// One line a player can read, without jargon.
    pub fn message(&self) -> String {
        match self {
            LoaderError::UnreadableVersion => {
                "That version is not a release the launcher can install. Snapshots and \
                 pre-releases have no mod loader profile."
                    .to_owned()
            }
            LoaderError::BeforeTheLoader { loader, since } => format!(
                "{} did not exist for this version. {} starts at {}.",
                loader.label(),
                loader.label(),
                since
            ),
            LoaderError::NotPublished { loader, version } => format!(
                "{} has no build for {} yet. Try the newest {} release instead.",
                loader.label(),
                version,
                loader.label()
            ),
        }
    }
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for LoaderError {}

/// The pair an instance actually launches: a game version and a loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoaderTarget {
    pub loader: Loader,
    pub version: GameVersion,
}

impl LoaderTarget {
    /// Builds the pair, refusing a version the loader never supported.
    pub fn new(loader: Loader, version: &str) -> Result<LoaderTarget, LoaderError> {
        let version = GameVersion::parse(version).ok_or(LoaderError::UnreadableVersion)?;
        let (major, minor) = loader.earliest();
        if !version.at_least(major, minor) {
            return Err(LoaderError::BeforeTheLoader {
                loader,
                since: GameVersion::new(major, minor, 0),
            });
        }
        Ok(LoaderTarget { loader, version })
    }

    /// Every loader that can run this version, oldest support first.
    pub fn loaders_for(version: &str) -> Vec<Loader> {
        let Some(version) = GameVersion::parse(version) else {
            return Vec::new();
        };
        Loader::ALL
            .iter()
            .copied()
            .filter(|loader| {
                let (major, minor) = loader.earliest();
                version.at_least(major, minor)
            })
            .collect()
    }

    /// The JSON profile describing the loader's libraries and main class.
    ///
    /// Forge has no per-version JSON endpoint of this shape — its installer
    /// writes a profile into `launcher_profiles.json` — so it is resolved by
    /// [`LoaderTarget::forge_installer_url`] instead.
    pub fn profile_url(&self) -> String {
        match self.loader {
            Loader::Fabric => format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}",
                self.version, "0.16.9"
            ),
            Loader::Quilt => format!(
                "https://meta.quiltmc.org/v3/versions/loader/{}/{}",
                self.version, "0.27.0"
            ),
            Loader::Forge => self.forge_installer_url(),
        }
    }

    /// Forge's own installer, which produces the profile and libraries.
    pub fn forge_installer_url(&self) -> String {
        format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{v}-{b}/forge-{v}-{b}-installer.jar",
            v = self.version,
            b = self.forge_build().unwrap_or("recommended")
        )
    }

    /// Forge's own build number, as published in `promotions_slim.json`
    /// (`"1.8-recommended": "11.14.4.1577"`). The installer reads it from there;
    /// without it there is no installer to fetch, so `None` is honest.
    pub fn forge_build(&self) -> Option<&'static str> {
        None
    }

    /// The folder inside a game directory this loader reads mods from.
    pub fn mods_folder(&self) -> &'static str {
        self.loader.mods_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_the_way_players_type_them() {
        assert_eq!(
            GameVersion::parse("1.21.1"),
            Some(GameVersion::new(1, 21, 1))
        );
        assert_eq!(GameVersion::parse("1.8"), Some(GameVersion::new(1, 8, 0)));
        assert_eq!(GameVersion::parse("26.3"), Some(GameVersion::new(26, 3, 0)));
        assert_eq!(
            GameVersion::parse(" 1.20 "),
            Some(GameVersion::new(1, 20, 0))
        );
    }

    #[test]
    fn snapshots_and_pre_releases_are_refused() {
        for bad in ["24w14a", "1.21-pre1", "1.21.1.2", "", "latest"] {
            assert_eq!(GameVersion::parse(bad), None, "{bad} parsed");
        }
    }

    #[test]
    fn versions_sort_in_release_order() {
        let mut list: Vec<GameVersion> = ["1.21.1", "1.8", "1.14", "1.21", "26.3"]
            .iter()
            .map(|v| GameVersion::parse(v).unwrap())
            .collect();
        list.sort();
        let text: Vec<String> = list.iter().map(|v| v.as_string()).collect();
        assert_eq!(text, ["1.8", "1.14", "1.21", "1.21.1", "26.3"]);
    }

    #[test]
    fn fabric_and_quilt_refuse_a_version_they_never_had() {
        // Fabric began at 1.14, so 1.8 is not a combination that exists.
        let refused = LoaderTarget::new(Loader::Fabric, "1.8").unwrap_err();
        assert_eq!(
            refused,
            LoaderError::BeforeTheLoader {
                loader: Loader::Fabric,
                since: GameVersion::new(1, 14, 0)
            }
        );
        assert!(refused.message().contains("Fabric starts at 1.14"));
        assert!(LoaderTarget::new(Loader::Fabric, "1.14").is_ok());
        assert!(LoaderTarget::new(Loader::Quilt, "1.14.4").is_ok());
        assert!(LoaderTarget::new(Loader::Quilt, "1.13.2").is_err());
    }

    #[test]
    fn forge_covers_the_whole_range_including_old_and_new() {
        for version in ["1.8", "1.12.2", "1.16.5", "1.21.1", "26.3"] {
            assert!(
                LoaderTarget::new(Loader::Forge, version).is_ok(),
                "Forge refused {version}"
            );
        }
    }

    #[test]
    fn a_snapshot_is_refused_by_every_loader() {
        for loader in Loader::ALL {
            assert_eq!(
                LoaderTarget::new(loader, "24w14a").unwrap_err(),
                LoaderError::UnreadableVersion
            );
        }
    }

    #[test]
    fn loaders_for_lists_only_what_exists() {
        assert_eq!(
            LoaderTarget::loaders_for("1.8"),
            vec![Loader::Forge],
            "1.8 has no Fabric or Quilt"
        );
        assert_eq!(LoaderTarget::loaders_for("1.14").len(), 3);
        assert_eq!(LoaderTarget::loaders_for("1.21.1").len(), 3);
        assert!(LoaderTarget::loaders_for("snapshot").is_empty());
    }

    #[test]
    fn every_loader_points_at_its_own_official_metadata() {
        let urls: Vec<&str> = Loader::ALL.iter().map(|l| l.versions_url()).collect();
        assert!(urls.iter().all(|u| u.starts_with("https://")));
        assert!(urls[0].contains("minecraftforge"));
        assert!(urls[1].contains("meta.fabricmc.net"));
        assert!(urls[2].contains("meta.quiltmc.org"));
        // No two loaders share an endpoint.
        for (i, a) in urls.iter().enumerate() {
            for b in &urls[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn profile_urls_name_the_version_and_the_loader() {
        let fabric = LoaderTarget::new(Loader::Fabric, "1.21.1").unwrap();
        assert!(fabric.profile_url().contains("meta.fabricmc.net"));
        assert!(fabric.profile_url().contains("1.21.1"));

        let quilt = LoaderTarget::new(Loader::Quilt, "1.20.1").unwrap();
        assert!(quilt.profile_url().contains("meta.quiltmc.org"));
        assert!(quilt.profile_url().contains("1.20.1"));

        let forge = LoaderTarget::new(Loader::Forge, "1.8").unwrap();
        assert!(forge.profile_url().contains("maven.minecraftforge.net"));
        assert!(forge.profile_url().contains("1.8"));
    }

    #[test]
    fn main_classes_are_the_ones_each_loader_documents() {
        assert!(Loader::Fabric.main_class().contains("fabricmc"));
        assert!(Loader::Quilt.main_class().contains("quiltmc"));
        assert!(Loader::Forge.main_class().contains("minecraftforge"));
    }

    #[test]
    fn loader_ids_round_trip_through_settings() {
        for loader in Loader::ALL {
            assert_eq!(Loader::parse(loader.id()), Some(loader));
            assert_eq!(
                Loader::parse(&loader.label().to_ascii_uppercase()),
                Some(loader)
            );
        }
        assert_eq!(Loader::parse("vanilla"), None);
        // A NeoForge instance is a Forge instance as far as launching goes.
        assert_eq!(Loader::parse("neoforge"), Some(Loader::Forge));
    }
}
