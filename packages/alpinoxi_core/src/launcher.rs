//! Launch policy: RAM profiles, JVM flags, instances and the performance set.
//!
//! Two rules drive this module:
//!
//! 1. **The launcher stays small.** Alpinoxi is native Rust and does not sit on
//!    top of a Java runtime, so its own footprint is a few megabytes. Everything
//!    RAM-related below is about the *Minecraft* process, never the launcher.
//! 2. **Minecraft gets what it needs, not more.** A larger heap is not a faster
//!    game; past roughly 4-8 GB a vanilla or Sodium client mostly pays for longer
//!    GC pauses. `LaunchPlan::auto` therefore caps at half of system RAM and
//!    never above 8 GB unless the player asks for it explicitly.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// RAM profiles offered in the launcher UI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryProfile {
    /// 2 GB: vanilla and light Sodium setups on low-RAM machines.
    #[default]
    Balanced,
    /// 4 GB: modded gameplay.
    Performance,
    /// Anything the player types, validated before launch.
    Custom(u16),
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

    pub const fn label(self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::Performance => "Performance",
            Self::Custom(_) => "Custom",
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MemoryProfileError {
    #[error("memory limit {0} MiB must be between 512 MiB and 16384 MiB")]
    OutOfRange(u16),
}

/// The resolved heap + flags handed to the Java process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchPlan {
    /// Heap size in MiB, applied to both `-Xms` and `-Xmx`.
    pub mebibytes: u32,
    /// JVM arguments, already ordered the way Java expects them.
    pub jvm_args: Vec<String>,
}

impl LaunchPlan {
    /// Picks a heap from system RAM and the chosen profile, then derives flags.
    pub fn auto(total_system_mebibytes: u16) -> Self {
        // `total_system_mebibytes` is mebibytes, not gigabytes: half of
        // 8 192 MiB is 4 096 MiB, and the floor keeps a tiny machine usable.
        let half = u32::from(total_system_mebibytes) / 2;
        let heap = half.clamp(1_024, 8_192);
        Self::with_heap(heap)
    }

    pub fn from_profile(profile: MemoryProfile, total_system_mebibytes: u16) -> Self {
        // The cap applies to every profile, custom included: a player who
        // types 16 GB into a 8 GB laptop gets the safe heap, not a crash on
        // the first world load. `auto` already clamps to half the machine
        // and to 8 GiB, so it is both the floor for a profile's own request
        // and the ceiling for everything else.
        let auto = Self::auto(total_system_mebibytes);
        let wanted = u32::from(profile.mebibytes());
        // A profile may ask for less than the cap, never for more.
        Self::with_heap(wanted.min(auto.mebibytes))
    }

    pub fn with_heap(mebibytes: u32) -> Self {
        Self {
            mebibytes,
            jvm_args: jvm_flags(mebibytes),
        }
    }

    /// The complete `-Xms/-Xmx` pair plus tuning flags, ready to append to a
    /// command line.
    pub fn arguments(&self) -> Vec<String> {
        let mut args = vec![
            format!("-Xms{}M", self.mebibytes),
            format!("-Xmx{}M", self.mebibytes),
        ];
        args.extend(self.jvm_args.iter().cloned());
        args
    }
}

/// G1GC tuning for a *client* (not a server). Based on Aikar's flags, trimmed to
/// what a game client actually benefits from and scaled by heap size.
///
/// Reference: <https://aikar.co/2018/07/02/tuning-the-jvm-g1gc-garbage-collector-flags-for-minecraft/>
pub fn jvm_flags(mebibytes: u32) -> Vec<String> {
    let mut flags: Vec<String> = [
        "-XX:+UseG1GC",
        "-XX:+ParallelRefProcEnabled",
        "-XX:MaxGCPauseMillis=200",
        "-XX:+UnlockExperimentalVMOptions",
        "-XX:+DisableExplicitGC",
        "-XX:G1NewSizePercent=30",
        "-XX:G1MaxNewSizePercent=40",
        "-XX:G1ReservePercent=20",
        "-XX:G1HeapWastePercent=5",
        "-XX:G1MixedGCCountTarget=4",
        "-XX:InitiatingHeapOccupancyPercent=15",
        "-XX:G1MixedGCLiveThresholdPercent=90",
        "-XX:G1RSetUpdatingPauseTimePercent=5",
        "-XX:SurvivorRatio=32",
        "-XX:MaxTenuringThreshold=1",
        "-XX:+PerfDisableSharedMem",
        "-Dfile.encoding=UTF-8",
    ]
    .iter()
    .map(|flag| (*flag).to_owned())
    .collect();

    // Region size must follow the heap: 8M under 12 GB, 16M above (Aikar).
    flags.push(if mebibytes >= 12 * 1_024 {
        "-XX:G1HeapRegionSize=16M".to_owned()
    } else {
        "-XX:G1HeapRegionSize=8M".to_owned()
    });

    // `AlwaysPreTouch` makes start-up allocate everything up front. It removes
    // later stalls on servers, but on a client it just doubles start-up time and
    // touches RAM the player may need for the browser. Only worth it on big heaps.
    if mebibytes >= 6 * 1_024 {
        flags.push("-XX:+AlwaysPreTouch".to_owned());
    }

    flags.push("-Daikars.new.flags=true".to_owned());
    flags.push("-Dusing.aikars.flags=https://mcflags.emc.gs".to_owned());
    flags
}

/// A named, versioned game instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub memory: MemoryProfile,
    #[serde(default)]
    pub mods: Vec<String>,
    #[serde(default)]
    pub resource_packs: Vec<String>,
    #[serde(default)]
    pub enabled_mods: Vec<String>,
    #[serde(default)]
    pub hud: crate::hud::HudSettings,
    #[serde(default)]
    pub keybinds: crate::hud::KeybindSet,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            name: "Vanilla".to_owned(),
            game_version: "1.21.1".to_owned(),
            loader: "fabric".to_owned(),
            memory: MemoryProfile::Balanced,
            mods: Vec::new(),
            resource_packs: Vec::new(),
            enabled_mods: Vec::new(),
            hud: crate::hud::HudSettings::default(),
            keybinds: crate::hud::KeybindSet::default(),
        }
    }
}

/// One client-side performance mod Alpinoxi recommends. We link to the official
/// release instead of redistributing files: licences stay with their authors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerformanceMod {
    pub id: &'static str,
    pub name: &'static str,
    pub what_it_fixes: &'static str,
    pub project: &'static str,
    /// Modrinth project slug, used to resolve the latest compatible file.
    pub slug: &'static str,
}

/// The curated client-side performance set. Everything here is a rendering,
/// memory or allocation fix, so it is safe for any world and any server.
pub const PERFORMANCE_MODS: &[PerformanceMod] = &[
    PerformanceMod {
        id: "sodium",
        name: "Sodium",
        what_it_fixes: "Rewrites chunk rendering: the single biggest FPS gain",
        project: "Modrinth",
        slug: "sodium",
    },
    PerformanceMod {
        id: "lithium",
        name: "Lithium",
        what_it_fixes: "Game-logic and tick optimisations without changing behaviour",
        project: "Modrinth",
        slug: "lithium",
    },
    PerformanceMod {
        id: "ferritecore",
        name: "FerriteCore",
        what_it_fixes: "Cuts model and blockstate memory use, often by hundreds of MB",
        project: "Modrinth",
        slug: "ferrite-core",
    },
    PerformanceMod {
        id: "krypton",
        name: "Krypton",
        what_it_fixes: "Optimises the network stack for lower ping and less CPU",
        project: "Modrinth",
        slug: "krypton",
    },
    PerformanceMod {
        id: "modernfix",
        name: "ModernFix",
        what_it_fixes: "Faster start-up and lower memory during world load",
        project: "Modrinth",
        slug: "modernfix",
    },
    PerformanceMod {
        id: "entityculling",
        name: "Entity Culling",
        what_it_fixes: "Skips rendering entities you cannot see",
        project: "Modrinth",
        slug: "entityculling",
    },
    PerformanceMod {
        id: "indium",
        name: "Indium",
        what_it_fixes: "Sodium compatibility layer for mods that use the Fabric renderer",
        project: "Modrinth",
        slug: "indium",
    },
    PerformanceMod {
        id: "immediatelyfast",
        name: "ImmediatelyFast",
        what_it_fixes: "Batches immediate-mode GUI and text rendering",
        project: "Modrinth",
        slug: "immediatelyfast",
    },
    PerformanceMod {
        id: "memoryleakfix",
        name: "Memory Leak Fix",
        what_it_fixes: "Patches known vanilla and mod memory leaks",
        project: "Modrinth",
        slug: "memoryleakfix",
    },
];

/// Short lines the launcher rotates through while it works, in the spirit of the
/// "OneClient is best!" line the project started from.
pub const SLOGANS: &[&str] = &[
    "OneClient is best!",
    "Alpinoxi is best!",
    "Low RAM, high frames.",
    "Native Rust. No Electron.",
    "Your skin, your name, your friends.",
    "Press the keybind, not a command.",
    "Same game. Less waiting.",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_scale_with_heap() {
        let small = jvm_flags(2_048);
        assert!(small.contains(&"-XX:G1HeapRegionSize=8M".to_owned()));
        assert!(!small.iter().any(|flag| flag.contains("AlwaysPreTouch")));

        let large = jvm_flags(12_288);
        assert!(large.contains(&"-XX:G1HeapRegionSize=16M".to_owned()));
        assert!(large.iter().any(|flag| flag.contains("AlwaysPreTouch")));
    }

    #[test]
    fn arguments_start_with_heap_pair() {
        let plan = LaunchPlan::with_heap(3_072);
        let args = plan.arguments();
        assert_eq!(args[0], "-Xms3072M");
        assert_eq!(args[1], "-Xmx3072M");
    }

    #[test]
    fn performance_mods_have_unique_ids() {
        let mut ids: Vec<&str> = PERFORMANCE_MODS.iter().map(|m| m.id).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn profile_never_exceeds_auto_budget() {
        // 4 GB machine: Balanced (2 GB) is above the 2 GB half-budget cap? No,
        // exactly at it, so it is kept; Performance (4 GB) falls back to auto.
        let plan = LaunchPlan::from_profile(MemoryProfile::Performance, 4_096);
        assert_eq!(plan.mebibytes, 2_048);
    }
}
