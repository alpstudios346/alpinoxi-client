//! Domain types shared by the Alpinoxi launcher and its future services.
//!
//! Everything here is pure Rust: no HTTP, no filesystem, no UI. The desktop app,
//! the auth package and (later) the in-game bridge all build on these types.

pub mod account;
pub mod friends;
pub mod hud;
pub mod import;
pub mod launcher;
pub mod skin;
pub mod slogan;

pub use account::{AccountProfile, AccountProvider, CrackedAccountError};
pub use friends::{ChatMessage, Friend, FriendRequest, FriendState, Presence};
pub use hud::{HudAnchor, HudElement, HudSettings, KeyChord, Keybind, KeybindAction, KeybindSet};
pub use import::{
    detect, detect_all, plan, DetectedSetup, ImportItem, ImportPlan, ImportSelection, LauncherKind,
    SkipReason, SkippedItem,
};
pub use launcher::{
    Instance, LaunchPlan, MemoryProfile, MemoryProfileError, PerformanceMod, PERFORMANCE_MODS,
    SLOGANS,
};
pub use skin::{Skin, SkinModel, SkinSource};
pub use slogan::{Slogan, SloganPicker};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_memory_limits_bounded() {
        assert!(MemoryProfile::Custom(511).validate().is_err());
        assert!(MemoryProfile::Custom(4_096).validate().is_ok());
        assert!(MemoryProfile::Custom(16_385).validate().is_err());
    }

    #[test]
    fn auto_profile_never_asks_for_more_than_half_of_ram() {
        for total in [4_u16, 8, 16, 32, 64] {
            let plan = LaunchPlan::auto(total);
            assert!(plan.mebibytes <= u32::from(total) * 1024 / 2);
            assert!(plan.mebibytes >= 1_024);
        }
    }

    #[test]
    fn provider_labels_are_stable() {
        assert_eq!(AccountProvider::Microsoft.label(), "Microsoft");
        assert_eq!(AccountProvider::ElyBy.label(), "Ely.by");
        assert_eq!(AccountProvider::Offline.label(), "Cracked");
    }
}
