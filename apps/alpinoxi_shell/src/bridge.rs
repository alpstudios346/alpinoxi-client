//! The launcher's link to the Alpinoxi mods.
//!
//! The mods ask Rust for HUD layout, keybinds, chat and performance advice over
//! `alpinoxi_bridge.dll` (JNI). The launcher answers those same questions on
//! the UI side, so the HUD preview here and the HUD in game cannot drift: both
//! read one `BridgeSnapshot`, built from one `Settings`.
//!
//! Nothing in this file touches the JVM. It is the shared shape, the writer
//! that hands it to the mods, and the reader the screens use.

use std::path::{Path, PathBuf};

use alpinoxi_core::hud::{HudElement, HudSettings};
use alpinoxi_core::launcher::MemoryProfile;
use serde::{Deserialize, Serialize};

use crate::state::{data_dir, Settings};

/// Where the bridge file lives: next to the settings, one folder up from
/// nothing, so a mod can find it by the same rule on every platform.
pub fn bridge_path() -> PathBuf {
    data_dir().join("bridge.json")
}

/// Everything the mods and the launcher's previews need, in one JSON document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeSnapshot {
    pub version: u32,
    pub hud: HudSnapshot,
    pub chat_channel: String,
    pub performance: PerfSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudSnapshot {
    pub enabled: bool,
    pub scale_percent: u8,
    pub background: bool,
    pub elements: Vec<HudElementSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudElementSnapshot {
    pub id: String,
    pub label: String,
    pub visible: bool,
    pub anchor: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfSnapshot {
    /// Heap the launcher will hand the JVM, in MiB.
    pub heap_mib: u16,
    pub performance_mods: Vec<String>,
    /// The advice line shown on the performance card.
    pub advice: String,
}

/// The document the mods read. Bumping it means an old mod jar sees a shape it
/// understands or refuses it; it never guesses.
pub const SNAPSHOT_VERSION: u32 = 1;

impl BridgeSnapshot {
    /// Builds the snapshot from the settings the launcher is editing right now.
    pub fn from_settings(settings: &Settings, chat_channel: &str) -> Self {
        let heap_mib = alpinoxi_core::launcher::LaunchPlan::from_profile(
            settings.memory,
            settings.system_memory_mib,
        )
        .mebibytes;

        Self {
            version: SNAPSHOT_VERSION,
            hud: hud_snapshot(&settings.hud),
            chat_channel: chat_channel.to_owned(),
            performance: PerfSnapshot {
                heap_mib,
                performance_mods: settings.instance.enabled_mods.clone(),
                advice: advice_for(settings.memory, heap_mib),
            },
        }
    }
}

pub fn hud_snapshot(hud: &HudSettings) -> HudSnapshot {
    HudSnapshot {
        enabled: hud.enabled,
        scale_percent: hud.scale_percent,
        background: hud.background,
        elements: HudElement::ALL
            .iter()
            .map(|element| HudElementSnapshot {
                id: element_id(*element).to_owned(),
                label: element.label().to_owned(),
                visible: hud.is_visible(*element),
                anchor: hud.placements[element.index()].anchor.label().to_owned(),
            })
            .collect(),
    }
}

/// The stable id the Java side uses. It matches `element_id` in
/// `packages/alpinoxi_jni`, which is the one place both sides agree on.
pub fn element_id(element: HudElement) -> &'static str {
    match element {
        HudElement::Fps => "fps",
        HudElement::Coordinates => "coordinates",
        HudElement::Ping => "ping",
        HudElement::Keystrokes => "keystrokes",
        HudElement::Armor => "armor",
        HudElement::PotionEffects => "potion_effects",
        HudElement::ServerAddress => "server_address",
        HudElement::Clock => "clock",
        HudElement::Friends => "friends",
        HudElement::Reach => "reach",
    }
}

/// One honest sentence about the chosen heap, not a sales pitch.
pub fn advice_for(profile: MemoryProfile, heap_mib: u16) -> String {
    match profile {
        MemoryProfile::Balanced => {
            "2 GB suits vanilla and a light Sodium setup. Extra heap mostly buys longer pauses."
                .to_owned()
        }
        MemoryProfile::Performance => format!(
            "{heap_mib} MiB for modded play. Past 8 GB a client mostly pays in longer GC pauses."
        ),
        MemoryProfile::Custom(_) => format!(
            "{heap_mib} MiB, capped at half of this machine's RAM so the desktop keeps breathing."
        ),
    }
}

/// Writes the snapshot where the mods look for it.
///
/// Failures are returned, never swallowed: a launcher that quietly stops
/// writing the file is a launcher whose HUD stops updating for no visible
/// reason.
pub fn write_snapshot(snapshot: &BridgeSnapshot) -> std::io::Result<()> {
    write_snapshot_to(&bridge_path(), snapshot)
}

pub fn write_snapshot_to(path: &Path, snapshot: &BridgeSnapshot) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(snapshot).unwrap_or_else(|_| "{}".to_owned());
    std::fs::write(path, text)
}

/// Reads the snapshot back, or `None` when there is none yet.
pub fn read_snapshot() -> Option<BridgeSnapshot> {
    let text = std::fs::read_to_string(bridge_path()).ok()?;
    serde_json::from_str(&text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_ids_are_unique_and_match_the_bridge() {
        let mut ids: Vec<&str> = HudElement::ALL.iter().map(|e| element_id(*e)).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "two elements share an id");
        assert_eq!(ids.len(), HudElement::ALL.len());
    }

    #[test]
    fn every_element_is_in_the_snapshot() {
        let settings = Settings::default();
        let snapshot = BridgeSnapshot::from_settings(&settings, "global");
        assert_eq!(snapshot.hud.elements.len(), HudElement::ALL.len());
        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
    }

    #[test]
    fn heap_is_capped_by_the_machine() {
        let mut settings = Settings::default();
        settings.system_memory_mib = 8 * 1024;
        settings.memory = MemoryProfile::Custom(16_000);
        let snapshot = BridgeSnapshot::from_settings(&settings, "global");
        assert!(snapshot.performance.heap_mib <= 4 * 1024);
    }

    #[test]
    fn a_snapshot_survives_a_round_trip() {
        let settings = Settings::default();
        let snapshot = BridgeSnapshot::from_settings(&settings, "dm:Alpinoxi");
        let text = serde_json::to_string(&snapshot).unwrap();
        let back: BridgeSnapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(back, snapshot);
    }

    #[test]
    fn hidden_elements_stay_hidden_in_the_snapshot() {
        let mut settings = Settings::default();
        settings.hud.toggle(HudElement::Ping);
        let snapshot = BridgeSnapshot::from_settings(&settings, "global");
        let ping = snapshot
            .hud
            .elements
            .iter()
            .find(|element| element.id == "ping")
            .unwrap();
        assert!(!ping.visible);
    }
}
