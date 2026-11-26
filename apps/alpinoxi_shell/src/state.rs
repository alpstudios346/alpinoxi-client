//! What the page can ask Rust to do, and what Rust remembers.
//!
//! The page is a view: it renders what this struct says and sends one command
//! per click. Keeping the decisions here means the same rules (the heap cap,
//! the keybind conflict check, the skin size check) hold for the window, the
//! in-game mods and the tests.

use std::path::PathBuf;

use alpinoxi_core::hud::{HudElement, HudSettings, KeyChord, KeyCode, KeybindAction, KeybindSet};
use alpinoxi_core::launcher::{Instance, LaunchPlan, MemoryProfile};
use alpinoxi_core::skin::{Skin, SkinModel};
use serde::{Deserialize, Serialize};

/// The launcher's own settings file. Same shape as the egui launcher writes,
/// so a player moving between builds keeps their setup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub system_memory_mib: u16,
    pub memory: MemoryProfile,
    pub instance: Instance,
    pub hud: HudSettings,
    pub keybinds: KeybindSet,
    pub local_skin: Option<Skin>,
    pub skin_model: SkinModel,
    pub keep_launcher_light: bool,
    pub close_launcher_on_launch: bool,
    pub cracked_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            system_memory_mib: 8 * 1024,
            memory: MemoryProfile::Balanced,
            instance: Instance::default(),
            hud: HudSettings::default(),
            keybinds: KeybindSet::default(),
            local_skin: None,
            skin_model: SkinModel::Classic,
            keep_launcher_light: true,
            close_launcher_on_launch: false,
            cracked_name: String::new(),
        }
    }
}

/// Where settings live: `%APPDATA%/alpinoxi` on Windows, `~/.config/alpinoxi`
/// elsewhere. The mods read `bridge.json` from the same folder.
pub fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .or_else(|_| std::env::var("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|_| std::env::var("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("alpinoxi")
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub struct LauncherState {
    pub settings: Settings,
    pub notice: String,
    pub running: bool,
}

impl LauncherState {
    pub fn load() -> Self {
        let settings = std::fs::read_to_string(settings_path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        Self {
            settings,
            notice: String::new(),
            running: false,
        }
    }

    pub fn save(&mut self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(&self.settings) {
            let _ = std::fs::write(path, text);
        }
        let snapshot = crate::bridge::BridgeSnapshot::from_settings(&self.settings, "global");
        let _ = crate::bridge::write_snapshot(&snapshot);
    }

    /// The heap the game will actually get, capped by this machine.
    pub fn plan(&self) -> LaunchPlan {
        LaunchPlan::from_profile(self.settings.memory, self.settings.system_memory_mib)
    }

    /// One command from the page. Unknown commands are refused, not ignored:
    /// a page that asks for something the launcher does not do is a page
    /// talking to the wrong binary.
    pub fn handle(&mut self, body: &str) -> String {
        let request: Command = match serde_json::from_str(body) {
            Ok(request) => request,
            Err(error) => return error_json(&format!("bad command: {error}")),
        };
        match request.cmd.as_str() {
            "heap" => format!("{{\"heap\":{}}}", self.plan().mebibytes),
            "set_memory" => {
                let mib = request.mib.unwrap_or(2_048);
                self.settings.memory = MemoryProfile::Custom(mib);
                format!("{{\"heap\":{}}}", self.plan().mebibytes)
            }
            "toggle_mod" => {
                let Some(id) = request.id else {
                    return error_json("toggle_mod needs an id");
                };
                let mods = &mut self.settings.instance.enabled_mods;
                match mods.iter().position(|existing| existing == &id) {
                    Some(index) => {
                        mods.remove(index);
                    }
                    None => mods.push(id),
                }
                self.save();
                "{\"ok\":true}".to_owned()
            }
            "bind" => self.bind(request),
            "toggle_hud_element" => {
                let Some(id) = request.id else {
                    return error_json("toggle_hud_element needs an id");
                };
                let Some(element) = HudElement::ALL
                    .iter()
                    .copied()
                    .find(|element| crate::bridge::element_id(*element) == id)
                else {
                    return error_json("unknown HUD element");
                };
                let visible = self.settings.hud.toggle(element);
                self.save();
                format!("{{\"visible\":{visible}}}")
            }
            "snapshot" => match serde_json::to_string(&crate::bridge::BridgeSnapshot::from_settings(
                &self.settings,
                "global",
            )) {
                Ok(json) => json,
                Err(error) => error_json(&error.to_string()),
            },
            "save" => {
                self.save();
                "{\"ok\":true}".to_owned()
            }
            other => error_json(&format!("unknown command: {other}")),
        }
    }

    fn bind(&mut self, request: Command) -> String {
        let Some(action_name) = request.action else {
            return error_json("bind needs an action");
        };
        let Some(action) = action_by_name(&action_name) else {
            return error_json("unknown action");
        };
        let Some(chord) = request.chord.and_then(|label| parse_chord(&label)) else {
            return error_json("that chord cannot be read");
        };
        let mut binds = self.settings.keybinds.clone();
        match binds.assign(action, chord) {
            Ok(()) => {
                self.settings.keybinds = binds;
                self.save();
                "{\"ok\":true}".to_owned()
            }
            Err(taken) => format!(
                "{{\"conflict\":{{\"action\":\"{}\",\"chord\":\"{}\"}}}}",
                action_name_of(taken),
                chord.label()
            ),
        }
    }
}

/// Every action the page can bind, by the name it sends.
///
/// `KeybindAction` has no string id of its own: the name the page uses is the
/// one written here, and `action_name_of` is its inverse, so a bind the page
/// sends always comes back with the same label.
fn action_by_name(name: &str) -> Option<KeybindAction> {
    Some(match name {
        "hud_editor" => KeybindAction::OpenHudEditor,
        "toggle_hud" => KeybindAction::ToggleHud,
        "friends" => KeybindAction::OpenFriends,
        "chat" => KeybindAction::OpenChat,
        "skin_preview" => KeybindAction::ToggleSkinPreview,
        other => KeybindAction::ToggleElement(
            HudElement::ALL
                .iter()
                .copied()
                .find(|element| crate::bridge::element_id(*element) == other)?,
        ),
    })
}

fn action_name_of(action: KeybindAction) -> String {
    match action {
        KeybindAction::OpenHudEditor => "hud_editor".to_owned(),
        KeybindAction::ToggleHud => "toggle_hud".to_owned(),
        KeybindAction::OpenFriends => "friends".to_owned(),
        KeybindAction::OpenChat => "chat".to_owned(),
        KeybindAction::ToggleSkinPreview => "skin_preview".to_owned(),
        KeybindAction::ToggleElement(element) => crate::bridge::element_id(element).to_owned(),
    }
}

#[derive(Debug, Deserialize)]
struct Command {
    cmd: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    mib: Option<u16>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    chord: Option<String>,
}

/// Reads a chord the way the page writes one: `Ctrl+Shift+H`, `F1`, `Y`.
///
/// The last part is the key; the parts before it are modifiers, which is the
/// same order `KeyChord::label` prints, so a chord survives a round trip.
fn parse_chord(label: &str) -> Option<KeyChord> {
    let mut parts: Vec<&str> = label
        .split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    let key_name = parts.pop()?;
    let key = KeyCode::ALL
        .iter()
        .copied()
        .find(|code| code.label().eq_ignore_ascii_case(key_name))?;
    let mut chord = KeyChord::key(key);
    for modifier in parts {
        match modifier.to_ascii_lowercase().as_str() {
            "shift" => chord.shift = true,
            "ctrl" | "control" => chord.ctrl = true,
            "alt" => chord.alt = true,
            _ => return None,
        }
    }
    Some(chord)
}

fn error_json(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> LauncherState {
        LauncherState {
            settings: Settings {
                system_memory_mib: 8 * 1024,
                ..Settings::default()
            },
            notice: String::new(),
            running: false,
        }
    }

    #[test]
    fn the_heap_is_capped_at_half_the_machine() {
        let mut state = state();
        let reply = state.handle(r#"{"cmd":"set_memory","mib":16000}"#);
        assert_eq!(reply, "{\"heap\":4096}");
    }

    #[test]
    fn toggling_a_mod_adds_then_removes_it() {
        let mut state = state();
        state.handle(r#"{"cmd":"toggle_mod","id":"sodium"}"#);
        assert!(state
            .settings
            .instance
            .enabled_mods
            .contains(&"sodium".to_owned()));
        state.handle(r#"{"cmd":"toggle_mod","id":"sodium"}"#);
        assert!(!state
            .settings
            .instance
            .enabled_mods
            .contains(&"sodium".to_owned()));
    }

    #[test]
    fn an_unknown_command_is_refused() {
        let mut state = state();
        let reply = state.handle(r#"{"cmd":"launch_the_missiles"}"#);
        assert!(reply.contains("unknown command"));
    }

    #[test]
    fn a_hud_element_toggles_by_its_stable_id() {
        let mut state = state();
        let first = state.handle(r#"{"cmd":"toggle_hud_element","id":"fps"}"#);
        assert_eq!(first, "{\"visible\":false}");
        let second = state.handle(r#"{"cmd":"toggle_hud_element","id":"fps"}"#);
        assert_eq!(second, "{\"visible\":true}");
    }

    #[test]
    fn a_hud_element_id_must_exist() {
        let mut state = state();
        let reply = state.handle(r#"{"cmd":"toggle_hud_element","id":"nonsense"}"#);
        assert!(reply.contains("unknown HUD element"));
    }

    #[test]
    fn a_snapshot_carries_the_settings() {
        let mut state = state();
        state.handle(r#"{"cmd":"toggle_mod","id":"lithium"}"#);
        let reply = state.handle(r#"{"cmd":"snapshot"}"#);
        assert!(reply.contains("lithium"));
        assert!(reply.contains("\"heap_mib\""));
    }
}
