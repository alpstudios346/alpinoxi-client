//! Importing an existing setup from another launcher.
//!
//! Players do not start from zero: they arrive with a `.minecraft` folder, a
//! Prism instance, a MultiMC instance or a CurseForge profile. Asking them to
//! rebuild it by hand is how a launcher loses its first user.
//!
//! Two rules keep this module honest:
//!
//! 1. **Import never moves anything.** The source stays exactly where it is, so a
//!    half-finished import cannot break the launcher the player still uses.
//! 2. **We only claim what we can read.** Each launcher is detected by files that
//!    really exist in it; a folder that merely has the right name is not enough.
//!    Anything we cannot parse is reported as skipped, with the reason, instead
//!    of being silently dropped.
//!
//! The module is pure: it reads paths and returns a plan. Copying is the caller's
//! job, so the same plan can be shown to the player, edited, and only then
//! executed.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::launcher::{Instance, MemoryProfile};

/// A launcher whose data we know how to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LauncherKind {
    /// Vanilla or Fabric `.minecraft` folder.
    Vanilla,
    /// Prism Launcher (`instances/<name>/.minecraft`).
    Prism,
    /// MultiMC (`instances/<name>/.minecraft`, same layout as Prism).
    MultiMc,
    /// CurseForge app profile.
    CurseForge,
    /// Modrinth app profile.
    Modrinth,
    /// Another Alpinoxi install, so a player can move to a new computer.
    Alpinoxi,
}

impl LauncherKind {
    pub const ALL: [Self; 6] = [
        Self::Vanilla,
        Self::Prism,
        Self::MultiMc,
        Self::CurseForge,
        Self::Modrinth,
        Self::Alpinoxi,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla / Fabric",
            Self::Prism => "Prism Launcher",
            Self::MultiMc => "MultiMC",
            Self::CurseForge => "CurseForge",
            Self::Modrinth => "Modrinth App",
            Self::Alpinoxi => "Alpinoxi",
        }
    }

    /// The folder that holds the game files, relative to the folder the player
    /// picked. Prism and MultiMC keep each instance one level down.
    pub const fn game_subdir(self) -> &'static str {
        match self {
            Self::Vanilla | Self::CurseForge | Self::Modrinth | Self::Alpinoxi => "",
            Self::Prism | Self::MultiMc => "instances",
        }
    }
}

/// What one candidate folder turned out to be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedSetup {
    pub kind: LauncherKind,
    pub root: PathBuf,
    /// The folder that contains `mods/`, `saves/`, `options.txt`.
    pub game_dir: PathBuf,
    /// Version name as the other launcher recorded it, e.g. `1.21.1-fabric`.
    pub version: Option<String>,
    pub loader: Option<String>,
    pub mod_count: usize,
    pub resource_pack_count: usize,
    pub shader_pack_count: usize,
    pub world_count: usize,
    pub has_options: bool,
}

impl DetectedSetup {
    /// One line for the import list: what the player is about to bring over.
    pub fn summary(&self) -> String {
        format!(
            "{} · {} mods · {} resource packs · {} worlds",
            self.version.as_deref().unwrap_or("unknown version"),
            self.mod_count,
            self.resource_pack_count,
            self.world_count
        )
    }

    /// Rejects a folder that looks like a game but has nothing worth importing.
    pub fn is_worth_importing(&self) -> bool {
        self.mod_count > 0
            || self.resource_pack_count > 0
            || self.world_count > 0
            || self.has_options
    }
}

/// Which parts of the source the player wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportSelection {
    pub mods: bool,
    pub resource_packs: bool,
    pub shader_packs: bool,
    pub worlds: bool,
    pub options: bool,
    /// Keybinds and HUD layout. Off by default: another launcher's keybinds are
    /// not Alpinoxi's, and silently remapping a player's controls is hostile.
    pub settings: bool,
    /// Accounts. **Always false and not settable.** Another launcher's token is
    /// encrypted with another launcher's key; copying it would either fail or, far
    /// worse, hand us a token we cannot vouch for. The player signs in here.
    pub accounts: bool,
}

impl Default for ImportSelection {
    fn default() -> Self {
        Self {
            mods: true,
            resource_packs: true,
            shader_packs: true,
            worlds: true,
            options: true,
            settings: false,
            accounts: false,
        }
    }
}

impl ImportSelection {
    /// The selection as it will actually be used: accounts are never imported.
    pub fn sanitised(mut self) -> Self {
        self.accounts = false;
        self
    }

    pub fn is_empty(&self) -> bool {
        let clean = self.sanitised();
        !(clean.mods
            || clean.resource_packs
            || clean.shader_packs
            || clean.worlds
            || clean.options
            || clean.settings)
    }
}

/// Why a file or folder was left behind. Reported, never hidden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkipReason {
    /// Not in the selection.
    NotSelected,
    /// A jar for a different Minecraft version.
    VersionMismatch { wanted: String, found: String },
    /// A `.jar.disabled`: the player turned it off, so it stays off.
    Disabled,
    /// Already present in the target with the same name.
    AlreadyPresent,
    /// A name we refuse to write, e.g. one containing a path separator or `..`.
    UnsafeName(String),
    /// The target instance has a `mods` folder but this entry is a folder, not a
    /// jar: Alpinoxi does not follow nested mod folders.
    Unsupported(String),
}

impl SkipReason {
    pub fn message(&self) -> String {
        match self {
            Self::NotSelected => "not selected".to_owned(),
            Self::VersionMismatch { wanted, found } => {
                format!("for {found}, this instance is {wanted}")
            }
            Self::Disabled => "disabled in the other launcher".to_owned(),
            Self::AlreadyPresent => "already present".to_owned(),
            Self::UnsafeName(name) => format!("unsafe name: {name}"),
            Self::Unsupported(name) => format!("not a plain file: {name}"),
        }
    }
}

/// One item that will be copied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportItem {
    /// Path inside the source game dir, e.g. `mods/sodium.jar`.
    pub relative: String,
    /// Destination inside the target instance.
    pub destination: String,
    pub bytes: u64,
}

/// An item that will not be copied, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedItem {
    pub relative: String,
    pub reason: SkipReason,
}

/// Everything an import will do. Shown to the player before anything is written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPlan {
    pub source: DetectedSetup,
    pub instance: Instance,
    pub items: Vec<ImportItem>,
    pub skipped: Vec<SkippedItem>,
    /// A warning the player must read before pressing Import.
    pub warning: Option<String>,
}

impl ImportPlan {
    pub fn total_bytes(&self) -> u64 {
        self.items.iter().map(|item| item.bytes).sum()
    }

    pub fn copy_count(&self) -> usize {
        self.items.len()
    }

    /// A human summary for the confirm button.
    pub fn headline(&self) -> String {
        format!(
            "{} files, {}",
            self.items.len(),
            human_bytes(self.total_bytes())
        )
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Files that must never be copied, whatever the selection says.
///
/// `launcher_profiles.json` and the auth files hold the *other* launcher's
/// credentials. `logs`, `crash-reports` and `screenshots` are noise that would
/// make an import slow for no benefit.
const NEVER_COPY: &[&str] = &[
    "launcher_profiles.json",
    "launcher_accounts.json",
    "launcher_accounts_microsoft_store.json",
    "usercache.json",
    "logs",
    "crash-reports",
    "screenshots",
    "versions",
    "libraries",
    "assets",
    "runtime",
    ".mixin.out",
    "natives",
];

/// Rejects a name that would escape the destination folder.
fn is_safe_relative(relative: &str) -> bool {
    if relative.is_empty() || relative.starts_with('/') || relative.contains('\\') {
        return false;
    }
    relative
        .split('/')
        .all(|part| !part.is_empty() && part != "." && part != ".." && !part.contains(':'))
}

/// Reads a launcher folder and works out what it is.
///
/// `root` is the folder the player picked. For Prism and MultiMC that is the
/// launcher folder and the game dir is `instances/<name>/.minecraft`.
pub fn detect(root: &Path) -> Option<DetectedSetup> {
    // A picked `.minecraft` folder is the most common case, so try it first.
    if let Some(setup) = detect_game_dir(root, LauncherKind::Vanilla, root) {
        return Some(setup);
    }
    for (kind, marker) in [
        (LauncherKind::Prism, "prismlauncher.cfg"),
        (LauncherKind::MultiMc, "multimc.cfg"),
    ] {
        if root.join(marker).exists() {
            if let Some(setup) = detect_first_instance(root, kind) {
                return Some(setup);
            }
        }
    }
    // Alpinoxi keeps its instances under `instances/` too.
    if root.join("instances").is_dir() {
        if let Some(setup) = detect_first_instance(root, LauncherKind::Alpinoxi) {
            return Some(setup);
        }
    }
    None
}

/// Every setup under a launcher folder, one per instance.
///
/// A Prism install with twenty instances should offer twenty rows, not one, or
/// the player has to run the import twenty times.
pub fn detect_all(root: &Path) -> Vec<DetectedSetup> {
    if let Some(setup) = detect_game_dir(root, LauncherKind::Vanilla, root) {
        return vec![setup];
    }
    let mut found = Vec::new();
    let kind = if root.join("prismlauncher.cfg").exists() {
        LauncherKind::Prism
    } else if root.join("multimc.cfg").exists() {
        LauncherKind::MultiMc
    } else {
        LauncherKind::Alpinoxi
    };
    let Ok(entries) = std::fs::read_dir(root.join("instances")) else {
        return found;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let game_dir = entry.path().join(".minecraft");
        let game_dir = if game_dir.is_dir() {
            game_dir
        } else {
            entry.path()
        };
        if let Some(setup) = detect_game_dir(&game_dir, kind, root) {
            found.push(setup);
        }
    }
    found.sort_by(|left, right| left.game_dir.cmp(&right.game_dir));
    found
}

fn detect_first_instance(root: &Path, kind: LauncherKind) -> Option<DetectedSetup> {
    detect_all(root)
        .into_iter()
        .find(|setup| setup.kind == kind && setup.is_worth_importing())
        .or_else(|| {
            let entries = std::fs::read_dir(root.join("instances")).ok()?;
            for entry in entries.flatten() {
                if !entry.path().is_dir() {
                    continue;
                }
                let game_dir = entry.path().join(".minecraft");
                let game_dir = if game_dir.is_dir() {
                    game_dir
                } else {
                    entry.path()
                };
                if let Some(setup) = detect_game_dir(&game_dir, kind, root) {
                    return Some(setup);
                }
            }
            None
        })
}

fn detect_game_dir(game_dir: &Path, kind: LauncherKind, root: &Path) -> Option<DetectedSetup> {
    if !game_dir.is_dir() {
        return None;
    }
    let has_game_marker = game_dir.join("options.txt").exists()
        || game_dir.join("mods").is_dir()
        || game_dir.join("saves").is_dir()
        || game_dir.join("versions").is_dir();
    if !has_game_marker {
        return None;
    }

    let version = read_version(game_dir);
    let loader = read_loader(game_dir, version.as_deref());

    Some(DetectedSetup {
        kind,
        root: root.to_path_buf(),
        game_dir: game_dir.to_path_buf(),
        version,
        loader,
        mod_count: count_files(&game_dir.join("mods"), "jar"),
        resource_pack_count: count_dirs(&game_dir.join("resourcepacks")),
        shader_pack_count: count_dirs(&game_dir.join("shaderpacks")),
        world_count: count_worlds(&game_dir.join("saves")),
        has_options: game_dir.join("options.txt").exists(),
    })
}

fn read_version(game_dir: &Path) -> Option<String> {
    // Fabric keeps the version in the launcher profile; Prism in `mmc-pack.json`.
    let profiles = game_dir.join("launcher_profiles.json");
    if let Ok(text) = std::fs::read_to_string(&profiles) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(name) = value
                .get("profiles")
                .and_then(|profiles| profiles.as_object())
                .and_then(|profiles| {
                    profiles
                        .values()
                        .find_map(|profile| profile.get("lastVersionId"))
                })
                .and_then(|version| version.as_str())
            {
                return Some(name.to_owned());
            }
        }
    }
    let pack = game_dir.join("mmc-pack.json");
    if let Ok(text) = std::fs::read_to_string(&pack) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(components) = value.get("components").and_then(|c| c.as_array()) {
                for component in components {
                    let uid = component.get("uid").and_then(|uid| uid.as_str());
                    if uid == Some("net.minecraft") {
                        return component
                            .get("version")
                            .and_then(|version| version.as_str())
                            .map(str::to_owned);
                    }
                }
            }
        }
    }
    None
}

fn read_loader(game_dir: &Path, version: Option<&str>) -> Option<String> {
    if game_dir.join("fabric.mod.json").exists()
        || std::fs::read_dir(game_dir.join("mods"))
            .map(|entries| {
                entries.flatten().any(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .contains("fabric")
                })
            })
            .unwrap_or(false)
    {
        return Some("fabric".to_owned());
    }
    if version.is_some_and(|version| version.contains("fabric")) {
        return Some("fabric".to_owned());
    }
    if version.is_some_and(|version| version.contains("forge")) {
        return Some("forge".to_owned());
    }
    if version.is_some_and(|version| version.contains("neoforge")) {
        return Some("neoforge".to_owned());
    }
    Some("vanilla".to_owned())
}

fn count_files(dir: &Path, extension: &str) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry.path().is_file()
                        && entry
                            .path()
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
                })
                .count()
        })
        .unwrap_or(0)
}

fn count_dirs(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry.path().is_dir()
                        || entry.path().extension().is_some_and(|ext| ext == "zip")
                })
                .count()
        })
        .unwrap_or(0)
}

fn count_worlds(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().join("level.dat").is_file())
                .count()
        })
        .unwrap_or(0)
}

/// Builds the plan for one detected setup.
///
/// `existing` is what the target instance already contains, so a second import
/// does not overwrite a jar the player already has. Nothing is copied here.
pub fn plan(
    source: DetectedSetup,
    selection: ImportSelection,
    target_name: &str,
    existing: &BTreeSet<String>,
) -> ImportPlan {
    let selection = selection.sanitised();
    let wanted_version = source.version.clone().unwrap_or_default();
    let mut items = Vec::new();
    let mut skipped = Vec::new();

    let mut push_files = |folder: &str, wanted: bool, destination: &str| {
        let Ok(entries) = std::fs::read_dir(source.game_dir.join(folder)) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let relative = format!("{folder}/{name}");
            if !is_safe_relative(&relative) {
                skipped.push(SkippedItem {
                    relative,
                    reason: SkipReason::UnsafeName(name),
                });
                continue;
            }
            if !wanted {
                skipped.push(SkippedItem {
                    relative,
                    reason: SkipReason::NotSelected,
                });
                continue;
            }
            if !entry.path().is_file() {
                skipped.push(SkippedItem {
                    relative,
                    reason: SkipReason::Unsupported(name),
                });
                continue;
            }
            if name.to_ascii_lowercase().ends_with(".disabled") {
                skipped.push(SkippedItem {
                    relative,
                    reason: SkipReason::Disabled,
                });
                continue;
            }
            if existing.contains(&name) {
                skipped.push(SkippedItem {
                    relative,
                    reason: SkipReason::AlreadyPresent,
                });
                continue;
            }
            // A mod built for another game version crashes the instance on start,
            // which looks like Alpinoxi's fault. Keep it out, and say so.
            if let Some(mod_version) = jar_game_version(&name) {
                if !wanted_version.is_empty() && !versions_compatible(&wanted_version, &mod_version)
                {
                    skipped.push(SkippedItem {
                        relative,
                        reason: SkipReason::VersionMismatch {
                            wanted: wanted_version.clone(),
                            found: mod_version,
                        },
                    });
                    continue;
                }
            }
            let bytes = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
            items.push(ImportItem {
                relative,
                destination: format!("{destination}/{name}"),
                bytes,
            });
        }
    };

    push_files("mods", selection.mods, "mods");
    push_files("resourcepacks", selection.resource_packs, "resourcepacks");
    push_files("shaderpacks", selection.shader_packs, "shaderpacks");

    if selection.options && source.has_options && !existing.contains("options.txt") {
        if let Ok(meta) = std::fs::metadata(source.game_dir.join("options.txt")) {
            items.push(ImportItem {
                relative: "options.txt".to_owned(),
                destination: "options.txt".to_owned(),
                bytes: meta.len(),
            });
        }
    }

    if selection.worlds {
        if let Ok(entries) = std::fs::read_dir(source.game_dir.join("saves")) {
            for entry in entries.flatten() {
                if !entry.path().join("level.dat").is_file() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                let relative = format!("saves/{name}");
                if !is_safe_relative(&relative) {
                    skipped.push(SkippedItem {
                        relative,
                        reason: SkipReason::UnsafeName(name),
                    });
                    continue;
                }
                if existing.contains(&name) {
                    skipped.push(SkippedItem {
                        relative,
                        reason: SkipReason::AlreadyPresent,
                    });
                    continue;
                }
                items.push(ImportItem {
                    relative,
                    destination: format!("saves/{name}"),
                    bytes: folder_bytes(&entry.path()),
                });
            }
        }
    }

    let warning = if selection.worlds && source.world_count > 0 {
        Some(
            "Worlds are copied, not moved: the other launcher keeps its copy. Do not play the \
             same world in both, or you will get two versions of it."
                .to_owned(),
        )
    } else {
        None
    };

    let mut instance = Instance {
        name: target_name.to_owned(),
        game_version: wanted_version,
        loader: source.loader.clone().unwrap_or_else(|| "fabric".to_owned()),
        memory: MemoryProfile::Balanced,
        ..Instance::default()
    };
    instance.mods = items
        .iter()
        .filter(|item| item.destination.starts_with("mods/"))
        .map(|item| item.destination.clone())
        .collect();
    instance.resource_packs = items
        .iter()
        .filter(|item| item.destination.starts_with("resourcepacks/"))
        .map(|item| item.destination.clone())
        .collect();

    ImportPlan {
        source,
        instance,
        items,
        skipped,
        warning,
    }
}

/// Sums a folder, for the size estimate shown before copying a world.
fn folder_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                folder_bytes(&path)
            } else {
                entry.metadata().map(|meta| meta.len()).unwrap_or(0)
            }
        })
        .sum()
}

/// Reads a game version out of a mod file name, when it is written there.
///
/// Mods are inconsistent: `sodium-fabric-0.6.0+mc1.21.1.jar` is easy, a jar with
/// no version at all is not. `None` means "unknown, let it through": refusing a
/// mod we simply could not parse would drop half of a player's folder.
pub fn jar_game_version(name: &str) -> Option<String> {
    let lowered = name.to_ascii_lowercase();
    for marker in ["mc1.", "minecraft-1.", "minecraft1."] {
        if let Some(index) = lowered.find(marker) {
            let rest = &lowered[index + marker.len() - 2..];
            let version: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if version.starts_with("1.") && version.len() >= 3 {
                return Some(version.trim_end_matches('.').to_owned());
            }
        }
    }
    None
}

/// Compares `1.21.1` with `1.21`: a mod built for 1.21 is fine on 1.21.1, the
/// other way round is not.
pub fn versions_compatible(instance: &str, mod_version: &str) -> bool {
    let instance = instance.split(['-', '+']).next().unwrap_or(instance);
    let instance_parts: Vec<&str> = instance.split('.').collect();
    let mod_parts: Vec<&str> = mod_version.split('.').collect();
    if instance_parts.len() < 2 || mod_parts.len() < 2 {
        return true;
    }
    instance_parts[0] == mod_parts[0] && instance_parts[1] == mod_parts[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("alpinoxi-import-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn detects_a_plain_minecraft_folder() {
        let dir = temp_dir("vanilla");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("mods/sodium-fabric-0.6.0+mc1.21.1.jar"), "jar");
        write(&dir.join("resourcepacks/pack.zip"), "zip");
        write(&dir.join("saves/world/level.dat"), "dat");

        let setup = detect(&dir).expect("should detect");
        assert_eq!(setup.kind, LauncherKind::Vanilla);
        assert_eq!(setup.mod_count, 1);
        assert_eq!(setup.world_count, 1);
        assert!(setup.is_worth_importing());
        assert!(setup.summary().contains("1 mods"));
    }

    #[test]
    fn ignores_a_folder_that_is_not_a_game() {
        let dir = temp_dir("empty");
        write(&dir.join("notes.txt"), "hello");
        assert!(detect(&dir).is_none());
    }

    #[test]
    fn detects_every_prism_instance() {
        let dir = temp_dir("prism");
        write(&dir.join("prismlauncher.cfg"), "[General]\n");
        write(
            &dir.join("instances/Fabric 1.21.1/.minecraft/mods/lithium.jar"),
            "jar",
        );
        write(
            &dir.join("instances/Fabric 1.21.1/mmc-pack.json"),
            "{\"components\":[{\"uid\":\"net.minecraft\",\"version\":\"1.21.1\"}]}",
        );
        write(
            &dir.join("instances/Vanilla/.minecraft/options.txt"),
            "fov:0.5\n",
        );

        let all = detect_all(&dir);
        assert_eq!(all.len(), 2);
        let fabric = all
            .iter()
            .find(|setup| setup.game_dir.to_string_lossy().contains("Fabric"))
            .expect("fabric instance");
        assert_eq!(fabric.version.as_deref(), Some("1.21.1"));
        assert_eq!(fabric.mod_count, 1);
    }

    #[test]
    fn plan_never_copies_accounts() {
        let dir = temp_dir("accounts");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("launcher_accounts.json"), "{\"accounts\":{}}");
        write(&dir.join("mods/sodium.jar"), "jar");

        let setup = detect(&dir).unwrap();
        let plan = plan(
            setup,
            ImportSelection::default(),
            "Imported",
            &BTreeSet::new(),
        );
        assert!(plan
            .items
            .iter()
            .all(|item| !item.relative.contains("launcher_accounts")));
        assert!(
            !ImportSelection {
                accounts: true,
                ..ImportSelection::default()
            }
            .sanitised()
            .accounts
        );
    }

    #[test]
    fn plan_drops_mods_for_another_version() {
        let dir = temp_dir("versions");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("mods/sodium-fabric-0.5.0+mc1.20.1.jar"), "jar");
        write(&dir.join("mods/lithium-fabric-0.14+mc1.21.1.jar"), "jar");
        write(&dir.join("mods/mystery.jar"), "jar");
        write(
            &dir.join("launcher_profiles.json"),
            "{\"profiles\":{\"a\":{\"lastVersionId\":\"1.21.1\"}}}",
        );

        let setup = detect(&dir).unwrap();
        let plan = plan(
            setup,
            ImportSelection::default(),
            "Imported",
            &BTreeSet::new(),
        );
        let copied: Vec<&str> = plan.items.iter().map(|i| i.relative.as_str()).collect();
        assert!(copied.contains(&"mods/lithium-fabric-0.14+mc1.21.1.jar"));
        assert!(copied.contains(&"mods/mystery.jar"));
        assert!(!copied.contains(&"mods/sodium-fabric-0.5.0+mc1.20.1.jar"));
        assert!(plan
            .skipped
            .iter()
            .any(|item| matches!(item.reason, SkipReason::VersionMismatch { .. })));
    }

    #[test]
    fn plan_skips_disabled_and_existing_mods() {
        let dir = temp_dir("existing");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("mods/off.jar.disabled"), "jar");
        write(&dir.join("mods/here.jar"), "jar");

        let setup = detect(&dir).unwrap();
        let mut existing = BTreeSet::new();
        existing.insert("here.jar".to_owned());
        let plan = plan(setup, ImportSelection::default(), "Imported", &existing);
        assert!(plan.items.is_empty());
        assert!(plan
            .skipped
            .iter()
            .any(|item| item.reason == SkipReason::Disabled));
        assert!(plan
            .skipped
            .iter()
            .any(|item| item.reason == SkipReason::AlreadyPresent));
    }

    #[test]
    fn deselected_folders_are_reported_not_silently_dropped() {
        let dir = temp_dir("selection");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("mods/keep.jar"), "jar");
        write(&dir.join("resourcepacks/pack.zip"), "zip");

        let setup = detect(&dir).unwrap();
        let plan = plan(
            setup,
            ImportSelection {
                resource_packs: false,
                ..ImportSelection::default()
            },
            "Imported",
            &BTreeSet::new(),
        );
        assert_eq!(plan.items.len(), 1);
        assert!(plan
            .skipped
            .iter()
            .any(|item| item.relative.starts_with("resourcepacks/")
                && item.reason == SkipReason::NotSelected));
    }

    #[test]
    fn worlds_are_copied_with_a_warning() {
        let dir = temp_dir("worlds");
        write(&dir.join("options.txt"), "fov:0.5\n");
        write(&dir.join("saves/survival/level.dat"), "dat");
        write(&dir.join("saves/survival/region/r.0.0.mca"), "mca");

        let setup = detect(&dir).unwrap();
        let plan = plan(
            setup,
            ImportSelection::default(),
            "Imported",
            &BTreeSet::new(),
        );
        assert!(plan
            .items
            .iter()
            .any(|item| item.destination == "saves/survival"));
        assert!(plan.warning.is_some());
        assert!(plan.total_bytes() >= 3);
    }

    #[test]
    fn unsafe_names_are_refused() {
        assert!(!is_safe_relative("../evil.jar"));
        assert!(!is_safe_relative("mods/../../evil.jar"));
        assert!(!is_safe_relative("C:/evil.jar"));
        assert!(!is_safe_relative("mods\\evil.jar"));
        assert!(is_safe_relative("mods/evil.jar"));
    }

    #[test]
    fn version_compatibility_is_one_way() {
        assert!(versions_compatible("1.21.1", "1.21"));
        assert!(!versions_compatible("1.20.1", "1.21"));
        assert!(versions_compatible("1.21.1-fabric", "1.21.1"));
    }

    #[test]
    fn jar_versions_are_read_from_names() {
        assert_eq!(
            jar_game_version("sodium-fabric-0.6.0+mc1.21.1.jar").as_deref(),
            Some("1.21.1")
        );
        assert_eq!(jar_game_version("mystery.jar"), None);
    }

    #[test]
    fn headline_is_readable() {
        let plan = ImportPlan {
            source: DetectedSetup {
                kind: LauncherKind::Vanilla,
                root: PathBuf::from("/a"),
                game_dir: PathBuf::from("/a"),
                version: None,
                loader: None,
                mod_count: 0,
                resource_pack_count: 0,
                shader_pack_count: 0,
                world_count: 0,
                has_options: false,
            },
            instance: Instance::default(),
            items: vec![ImportItem {
                relative: "mods/a.jar".to_owned(),
                destination: "mods/a.jar".to_owned(),
                bytes: 2 * 1024 * 1024,
            }],
            skipped: Vec::new(),
            warning: None,
        };
        assert!(plan.headline().contains("1 files"));
        assert!(plan.headline().contains("2.0 MB"));
    }
}
