//! What the launcher remembers between runs.
//!
//! Stored as one JSON file in the platform config folder. It holds *settings*,
//! never tokens: those belong to the auth package's vault.

use std::path::PathBuf;

use alpinoxi_core::hud::{HudSettings, KeybindSet};
use alpinoxi_core::launcher::{Instance, MemoryProfile};
use alpinoxi_core::skin::{Skin, SkinModel, SkinSource};
use serde::{Deserialize, Serialize};

use alpinoxi_core::account::AccountProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Total system RAM in MiB, detected once so the launch plan can cap the heap.
    pub system_memory_mib: u16,
    pub memory: MemoryProfile,
    pub instance: Instance,
    pub hud: HudSettings,
    pub keybinds: KeybindSet,
    pub last_provider: Option<AccountProvider>,
    pub cracked_name: String,
    pub local_skin: Option<Skin>,
    pub skin_model: SkinModel,
    pub keep_launcher_light: bool,
    pub close_launcher_on_launch: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            system_memory_mib: detect_system_memory(),
            memory: MemoryProfile::Balanced,
            instance: Instance::default(),
            hud: HudSettings::default(),
            keybinds: KeybindSet::default(),
            last_provider: None,
            cracked_name: String::new(),
            local_skin: None,
            skin_model: SkinModel::Classic,
            keep_launcher_light: true,
            close_launcher_on_launch: false,
        }
    }
}

impl Settings {
    pub fn local_skin_for(&self, provider: AccountProvider) -> Option<Skin> {
        self.local_skin.clone().filter(|skin| {
            skin.source == SkinSource::for_provider(provider)
                || skin.source == SkinSource::Local
        })
    }
}

/// Reads total physical memory, with a conservative fallback.
///
/// Linux and Windows are read from the OS directly; anywhere else (and on any
/// failure) the launcher assumes 8 GB, which yields a safe 4 GB heap ceiling.
pub fn detect_system_memory() -> u16 {
    if let Some(mib) = detect_linux() {
        return mib;
    }
    if let Some(mib) = detect_windows() {
        return mib;
    }
    8 * 1024
}

#[cfg(target_os = "linux")]
fn detect_linux() -> Option<u16> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
    let line = meminfo.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kib: u32 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some((kib / 1024).min(u32::from(u16::MAX)) as u16)
}

#[cfg(not(target_os = "linux"))]
fn detect_linux() -> Option<u16> {
    None
}

#[cfg(target_os = "windows")]
fn detect_windows() -> Option<u16> {
    // `wmic` is gone from newer Windows, so ask PowerShell through the CIM
    // interface. One short call at start-up, cached in the settings file.
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])
        .output()
        .ok()?;
    let bytes: u64 = String::from_utf8_lossy(&output.stdout).trim().parse().ok()?;
    Some((bytes / 1024 / 1024).min(u64::from(u16::MAX)) as u16)
}

#[cfg(not(target_os = "windows"))]
fn detect_windows() -> Option<u16> {
    None
}

/// Path of the settings file, created on demand.
pub fn settings_path() -> PathBuf {
    let base = config_dir();
    base.join("alpinoxi").join("settings.json")
}

fn config_dir() -> PathBuf {
    if let Ok(value) = std::env::var("APPDATA") {
        return PathBuf::from(value);
    }
    if let Ok(value) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(value);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from(".")
}

pub fn load() -> Settings {
    let path = settings_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Settings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(settings: &Settings) -> std::io::Result<()> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(settings).unwrap_or_else(|_| "{}".to_owned());
    std::fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_json() {
        let settings = Settings::default();
        let text = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.memory, settings.memory);
        assert_eq!(back.hud, settings.hud);
        assert_eq!(back.keybinds, settings.keybinds);
    }

    #[test]
    fn detected_memory_is_plausible() {
        let mib = detect_system_memory();
        assert!(mib >= 1_024, "detected {mib} MiB, expected at least 1 GiB");
    }
}
