//! JNI bridge between the Alpinoxi Fabric mods and the Rust rules.
//!
//! Design rule: **Java renders, Rust decides.** The mods call these functions
//! with plain strings and numbers, and get JSON back. That keeps one source of
//! truth (this workspace) for HUD layout, keybind resolution, chat rules, friend
//! state and performance advice, and means a fix in Rust ships to every mod.
//!
//! The library is also a normal Rust crate, so every function below has a Rust
//! entry point that the tests exercise without a JVM. The `#[no_mangle]` wrappers
//! at the bottom are thin: convert, call, convert.
//!
//! JNI naming: `Java_<package>_<class>_<method>`, package dots become
//! underscores. See `mods/alpinoxi-hud/src/main/java/net/alpinoxi/bridge/Native.java`.

use alpinoxi_core::friends::{ChatMessage, Friend};
use alpinoxi_core::hud::{
    HudAnchor, HudElement, HudSettings, KeyChord, KeyCode, KeybindAction, KeybindSet,
};
use alpinoxi_core::launcher::{jvm_flags, MemoryProfile, PERFORMANCE_MODS};
use alpinoxi_core::skin::SkinModel;
use alpinoxi_ingame::chat::ChatTabs;
use alpinoxi_ingame::{FriendBook, Overlay};
use serde::{Deserialize, Serialize};

/// Everything the game asked Rust to keep between frames.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeState {
    pub hud: HudSettings,
    pub keybinds: KeybindSet,
    pub chat: ChatTabs,
    pub book: FriendBook,
    pub overlay: Overlay,
}

impl BridgeState {
    pub fn new() -> Self {
        Self {
            hud: HudSettings::default(),
            keybinds: KeybindSet::default(),
            chat: ChatTabs::default(),
            book: FriendBook::default(),
            overlay: Overlay::default(),
        }
    }
}

/// HUD layout as the renderer needs it: one row per visible element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudRow {
    pub id: String,
    pub label: String,
    pub anchor: String,
    pub x: f32,
    pub y: f32,
    pub offset_x: i16,
    pub offset_y: i16,
}

/// Resolved layout for one frame, in normalised screen coordinates.
pub fn hud_layout(hud: &HudSettings) -> Vec<HudRow> {
    hud_frame(hud)["elements"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| HudRow {
                    id: row["id"].as_str().unwrap_or_default().to_owned(),
                    label: row["label"].as_str().unwrap_or_default().to_owned(),
                    anchor: row["anchor"].as_str().unwrap_or_default().to_owned(),
                    x: row["x"].as_f64().unwrap_or_default() as f32,
                    y: row["y"].as_f64().unwrap_or_default() as f32,
                    offset_x: row["offset_x"].as_i64().unwrap_or_default() as i16,
                    offset_y: row["offset_y"].as_i64().unwrap_or_default() as i16,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The whole HUD frame as the renderer wants it in one JSON answer:
/// `{hud_enabled, hud_scale, elements: [...]}`.
pub fn hud_frame(hud: &HudSettings) -> serde_json::Value {
    let rows = layout_rows(hud);
    serde_json::json!({
        "hud_enabled": hud.enabled,
        "hud_scale": hud.scale_percent,
        "elements": rows,
    })
}

fn layout_rows(hud: &HudSettings) -> Vec<HudRow> {
    let mut rows = Vec::new();
    for element in HudElement::ALL {
        if !hud.is_visible(element) {
            continue;
        }
        let placement = hud.placement(element);
        let (x, y) = placement.anchor.origin();
        rows.push(HudRow {
            id: element_id(element).to_owned(),
            label: element.label().to_owned(),
            anchor: placement.anchor.label().to_owned(),
            x,
            y,
            offset_x: placement.offset.0,
            offset_y: placement.offset.1,
        });
    }
    rows
}

fn element_id(element: HudElement) -> &'static str {
    match element {
        HudElement::Fps => "fps",
        HudElement::Coordinates => "coords",
        HudElement::Ping => "ping",
        HudElement::Keystrokes => "keys",
        HudElement::Armor => "armor",
        HudElement::PotionEffects => "effects",
        HudElement::ServerAddress => "server",
        HudElement::Clock => "clock",
        HudElement::Friends => "friends",
        HudElement::Reach => "reach",
    }
}

/// What a pressed key should do, as a JSON-friendly string.
///
/// `mods` is a bitfield: 1 = shift, 2 = ctrl, 4 = alt. The game maps its GLFW
/// key to the same names used here.
pub fn resolve_key(key: &str, mods: u8, keybinds: &KeybindSet) -> String {
    let Some(code) = key_from_name(key) else {
        return "nothing".to_owned();
    };
    let chord = KeyChord {
        key: code,
        shift: mods & 1 != 0,
        ctrl: mods & 2 != 0,
        alt: mods & 4 != 0,
    };
    match keybinds.resolve(chord) {
        Some(KeybindAction::OpenHudEditor) => "open_hud_editor".to_owned(),
        Some(KeybindAction::ToggleHud) => "toggle_hud".to_owned(),
        Some(KeybindAction::ToggleElement(element)) => format!("toggle:{}", element_id(element)),
        Some(KeybindAction::OpenFriends) => "open_friends".to_owned(),
        Some(KeybindAction::OpenChat) => "open_chat".to_owned(),
        Some(KeybindAction::ToggleSkinPreview) => "toggle_skin_preview".to_owned(),
        None => "nothing".to_owned(),
    }
}

fn key_from_name(name: &str) -> Option<KeyCode> {
    let normalised = name.trim().to_ascii_uppercase();
    KeyCode::ALL.iter().copied().find(|code| {
        code.label().to_ascii_uppercase().replace(' ', "_") == normalised.replace(' ', "_")
    })
}

/// Incoming chat line: sanitised, routed to its tab, mention flag applied.
pub fn receive_chat(
    state: &mut BridgeState,
    from_name: &str,
    provider: &str,
    channel: &str,
    text: &str,
    timestamp: u64,
) -> Result<String, String> {
    let provider = alpinoxi_core::AccountProvider::from_id(provider)
        .ok_or_else(|| format!("unknown provider {provider}"))?;
    let mut message = ChatMessage::new(
        Friend::new(provider, from_name, from_name),
        channel,
        text,
        timestamp,
    );
    message.text = ChatMessage::sanitise(&message.text).map_err(|error| error.to_string())?;
    let my_name = state
        .book
        .me
        .as_ref()
        .map(|me| me.name.clone())
        .unwrap_or_default();
    message.detect_mention(&my_name);
    state.chat.push(message);
    Ok(serde_json::json!({
        "mentions": state.chat.mentions(),
        "unread": state.chat.total_unread(),
    })
    .to_string())
}

/// Launch advice for the in-game settings screen: what the JVM flags say and
/// which performance mods are worth installing for this heap.
pub fn performance_report(heap_mib: u32, installed: &[String]) -> String {
    let flags = jvm_flags(heap_mib);
    let missing: Vec<&str> = PERFORMANCE_MODS
        .iter()
        .filter(|entry| !installed.iter().any(|id| id == entry.id))
        .map(|entry| entry.name)
        .collect();
    serde_json::json!({
        "heap_mib": heap_mib,
        "flag_count": flags.len(),
        "gc": "G1GC",
        "missing_mods": missing,
        "advice": if heap_mib > 8 * 1024 {
            "A heap above 8 GB rarely helps a client: install Sodium and keep 4 GB."
        } else {
            "Heap is in the range where G1GC pauses stay short."
        }
    })
    .to_string()
}

/// Memory profile id -> MiB, for the mods' own settings screen.
pub fn memory_mebibytes(profile: &str) -> u32 {
    match profile {
        "performance" => u32::from(MemoryProfile::Performance.mebibytes()),
        "balanced" => u32::from(MemoryProfile::Balanced.mebibytes()),
        other => other.parse().unwrap_or(2_048),
    }
}

/// Skin model name the renderer should use for the local player.
pub fn skin_model_id(model: &str) -> &'static str {
    match model {
        "slim" => "slim",
        _ => "classic",
    }
}

/// Moves one HUD element to another anchor. Returns `false` for an unknown
/// element or anchor, so the mod can keep showing the old position.
pub fn set_anchor(state: &mut BridgeState, element_id: &str, anchor_name: &str) -> bool {
    let Some(element) = HudElement::ALL
        .iter()
        .copied()
        .find(|element| element_id(element) == element_id)
    else {
        return false;
    };
    let normalised = anchor_name.trim().to_ascii_lowercase();
    let Some(anchor) = HudAnchor::ALL.iter().copied().find(|anchor| {
        anchor.label().to_ascii_lowercase() == normalised
            || anchor.label().to_ascii_lowercase().replace(' ', "") == normalised.replace(' ', "")
    }) else {
        return false;
    };
    state.hud.set_anchor(element, anchor);
    true
}

/// Toggles one element; returns its new visibility, or `None` if the id is
/// unknown. Used by the mods' own keybind actions.
pub fn toggle_element(state: &mut BridgeState, element_id: &str) -> Option<bool> {
    let element = HudElement::ALL
        .iter()
        .copied()
        .find(|element| element_id(element) == element_id)?;
    Some(state.hud.toggle(element))
}

/// A JSON snapshot of the whole bridge, for the mod's debug screen.
pub fn snapshot(state: &BridgeState) -> String {
    serde_json::json!({
        "hud_enabled": state.hud.enabled,
        "hud_scale": state.hud.scale_percent,
        "elements": hud_layout(&state.hud),
        "friends_online": state
            .book
            .friends()
            .iter()
            .filter(|friend| state.book.presence_of(friend).is_online())
            .count(),
        "chat_unread": state.chat.total_unread(),
        "mentions": state.chat.mentions(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_layout_only_lists_visible_elements() {
        let mut hud = HudSettings::default();
        let all = hud_layout(&hud);
        hud.set_visible(HudElement::Fps, false);
        let without_fps = hud_layout(&hud);
        assert_eq!(all.len(), without_fps.len() + 1);
        assert!(!without_fps.iter().any(|row| row.id == "fps"));
    }

    #[test]
    fn key_resolution_matches_launcher_defaults() {
        let binds = KeybindSet::default();
        assert_eq!(resolve_key("H", 1, &binds), "open_hud_editor");
        assert_eq!(resolve_key("O", 0, &binds), "open_friends");
        assert_eq!(resolve_key("Y", 0, &binds), "open_chat");
        assert_eq!(resolve_key("Q", 0, &binds), "nothing");
        assert_eq!(resolve_key("not-a-key", 0, &binds), "nothing");
    }

    #[test]
    fn chat_reaches_the_right_tab_with_mentions() {
        let mut state = BridgeState::new();
        state.book.sign_in(Friend::new(
            alpinoxi_core::AccountProvider::Offline,
            "uuid-me",
            "Alpinoxi",
        ));
        let reply = receive_chat(
            &mut state,
            "Notch",
            "microsoft",
            "global",
            "@Alpinoxi hi",
            1,
        )
        .expect("chat should be accepted");
        assert!(reply.contains("\"mentions\":1"));
        assert!(receive_chat(&mut state, "Notch", "microsoft", "global", "   ", 2).is_err());
        assert!(receive_chat(&mut state, "Notch", "nonsense", "global", "hi", 3).is_err());
    }

    #[test]
    fn performance_report_names_missing_mods() {
        let report = performance_report(4_096, &["sodium".to_owned()]);
        assert!(report.contains("Lithium"));
        assert!(!report.contains("\"Sodium\""));
        assert!(report.contains("\"flag_count\""));
    }

    #[test]
    fn memory_profiles_are_readable_from_java() {
        assert_eq!(memory_mebibytes("performance"), 4_096);
        assert_eq!(memory_mebibytes("balanced"), 2_048);
        assert_eq!(memory_mebibytes("6144"), 6_144);
    }

    #[test]
    fn snapshot_is_valid_json() {
        let state = BridgeState::new();
        let parsed: serde_json::Value = serde_json::from_str(&snapshot(&state)).unwrap();
        assert_eq!(parsed["hud_enabled"], true);
    }

    #[test]
    fn anchors_can_be_moved_by_the_editor() {
        let mut state = BridgeState::new();
        assert!(set_anchor(&mut state, "fps", "Bottom right"));
        assert_eq!(
            state.hud.placement(HudElement::Fps).anchor,
            HudAnchor::BottomRight
        );
        assert!(!set_anchor(&mut state, "fps", "middle of nowhere"));
        assert!(!set_anchor(&mut state, "unknown", "Top left"));
    }

    #[test]
    fn elements_can_be_toggled_by_id() {
        let mut state = BridgeState::new();
        assert_eq!(toggle_element(&mut state, "coords"), Some(false));
        assert_eq!(toggle_element(&mut state, "coords"), Some(true));
        assert_eq!(toggle_element(&mut state, "nope"), None);
    }

    #[test]
    fn skin_model_is_normalised() {
        assert_eq!(skin_model_id("slim"), "slim");
        assert_eq!(skin_model_id("SOMETHING"), "classic");
    }
}

/// Thin JNI wrappers. Each one converts Java strings to Rust, calls the logic
/// above, and returns a JSON string the mod can parse.
#[cfg(feature = "jni-exports")]
mod jni_exports {
    use super::*;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;
    use std::cell::RefCell;

    thread_local! {
        static STATE: RefCell<BridgeState> = RefCell::new(BridgeState::new());
    }

    fn read(env: &mut JNIEnv, value: JString) -> String {
        env.get_string(&value)
            .map(|text| text.into())
            .unwrap_or_default()
    }

    fn reply(env: &mut JNIEnv, value: String) -> jstring {
        env.new_string(value)
            .map(|text| text.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_hudLayout(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let json = STATE.with(|state| hud_frame(&state.borrow().hud).to_string());
        reply(&mut env, json)
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_resolveKey(
        mut env: JNIEnv,
        _class: JClass,
        key: JString,
        mods: jni::sys::jint,
    ) -> jstring {
        let key = read(&mut env, key);
        let action = STATE.with(|state| resolve_key(&key, mods as u8, &state.borrow().keybinds));
        reply(&mut env, action)
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_receiveChat(
        mut env: JNIEnv,
        _class: JClass,
        from: JString,
        provider: JString,
        channel: JString,
        text: JString,
        timestamp: jni::sys::jlong,
    ) -> jstring {
        let (from, provider, channel, text) = (
            read(&mut env, from),
            read(&mut env, provider),
            read(&mut env, channel),
            read(&mut env, text),
        );
        let result = STATE.with(|state| {
            receive_chat(
                &mut state.borrow_mut(),
                &from,
                &provider,
                &channel,
                &text,
                timestamp as u64,
            )
        });
        match result {
            Ok(json) => reply(&mut env, json),
            Err(error) => reply(&mut env, serde_json::json!({ "error": error }).to_string()),
        }
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_performanceReport(
        mut env: JNIEnv,
        _class: JClass,
        heap_mib: jni::sys::jint,
        installed_csv: JString,
    ) -> jstring {
        let installed = read(&mut env, installed_csv);
        let installed: Vec<String> = installed
            .split(',')
            .filter(|entry| !entry.trim().is_empty())
            .map(|entry| entry.trim().to_owned())
            .collect();
        let report = performance_report(heap_mib.max(512) as u32, &installed);
        reply(&mut env, report)
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_setAnchor(
        mut env: JNIEnv,
        _class: JClass,
        element_id: JString,
        anchor: JString,
    ) -> jni::sys::jboolean {
        let element_id = read(&mut env, element_id);
        let anchor = read(&mut env, anchor);
        STATE.with(|state| set_anchor(&mut state.borrow_mut(), &element_id, &anchor))
            as jni::sys::jboolean
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_toggleElement(
        mut env: JNIEnv,
        _class: JClass,
        element_id: JString,
    ) -> jni::sys::jint {
        let element_id = read(&mut env, element_id);
        STATE.with(
            |state| match toggle_element(&mut state.borrow_mut(), &element_id) {
                Some(visible) => i32::from(visible),
                None => -1,
            },
        ) as jni::sys::jint
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_snapshot(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let json = STATE.with(|state| snapshot(&state.borrow()));
        reply(&mut env, json)
    }

    #[no_mangle]
    pub extern "system" fn Java_net_alpinoxi_bridge_Native_bridgeVersion(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        reply(&mut env, env!("CARGO_PKG_VERSION").to_owned())
    }
}
