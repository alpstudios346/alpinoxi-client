//! HUD layout and keybinds: the settings that must work **without a command**.
//!
//! Everything in game is reachable from a keybind: opening the HUD editor,
//! toggling each element, opening the friend list and opening the chat. The
//! launcher edits the same struct the in-game bridge reads, so a player can
//! arrange the HUD from the launcher or in game and both agree.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudElement {
    Fps,
    Coordinates,
    Ping,
    Keystrokes,
    Armor,
    PotionEffects,
    ServerAddress,
    Clock,
    Friends,
    Reach,
}

impl HudElement {
    pub const ALL: [Self; 10] = [
        Self::Fps,
        Self::Coordinates,
        Self::Ping,
        Self::Keystrokes,
        Self::Armor,
        Self::PotionEffects,
        Self::ServerAddress,
        Self::Clock,
        Self::Friends,
        Self::Reach,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Fps => "FPS",
            Self::Coordinates => "Coordinates",
            Self::Ping => "Ping",
            Self::Keystrokes => "Keystrokes",
            Self::Armor => "Armor",
            Self::PotionEffects => "Potion effects",
            Self::ServerAddress => "Server address",
            Self::Clock => "Clock",
            Self::Friends => "Friends online",
            Self::Reach => "Reach",
        }
    }

    pub const fn default_anchor(self) -> HudAnchor {
        match self {
            Self::Fps | Self::Coordinates | Self::ServerAddress | Self::Friends => {
                HudAnchor::TopLeft
            }
            Self::Ping | Self::Clock => HudAnchor::TopRight,
            Self::Keystrokes => HudAnchor::BottomLeft,
            Self::Armor | Self::PotionEffects => HudAnchor::BottomRight,
            Self::Reach => HudAnchor::BottomCentre,
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|e| *e == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    BottomCentre,
}

impl HudAnchor {
    pub const ALL: [Self; 5] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
        Self::BottomCentre,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::TopRight => "Top right",
            Self::BottomLeft => "Bottom left",
            Self::BottomRight => "Bottom right",
            Self::BottomCentre => "Bottom centre",
        }
    }

    /// Normalised screen position for the anchor, as (x, y) in 0..1.
    pub const fn origin(self) -> (f32, f32) {
        match self {
            Self::TopLeft => (0.02, 0.02),
            Self::TopRight => (0.98, 0.02),
            Self::BottomLeft => (0.02, 0.98),
            Self::BottomRight => (0.98, 0.98),
            Self::BottomCentre => (0.5, 0.98),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudPlacement {
    pub anchor: HudAnchor,
    /// Pixel offset from the anchor, so players can nudge elements.
    pub offset: (i16, i16),
}

impl HudPlacement {
    pub const fn new(anchor: HudAnchor) -> Self {
        Self {
            anchor,
            offset: (0, 0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudSettings {
    pub enabled: bool,
    pub scale_percent: u8,
    pub background: bool,
    pub placements: [HudPlacement; 10],
    pub visible: [bool; 10],
}

impl Default for HudSettings {
    fn default() -> Self {
        let mut placements = [HudPlacement::new(HudAnchor::TopLeft); 10];
        let mut visible = [true; 10];
        for element in HudElement::ALL {
            placements[element.index()] = HudPlacement::new(element.default_anchor());
        }
        // Reach is informational and off by default; most players never need it.
        visible[HudElement::Reach.index()] = false;
        Self {
            enabled: true,
            scale_percent: 100,
            background: true,
            placements,
            visible,
        }
    }
}

impl HudSettings {
    pub fn is_visible(&self, element: HudElement) -> bool {
        self.visible[element.index()]
    }

    pub fn set_visible(&mut self, element: HudElement, value: bool) {
        self.visible[element.index()] = value;
    }

    pub fn placement(&self, element: HudElement) -> HudPlacement {
        self.placements[element.index()]
    }

    pub fn set_anchor(&mut self, element: HudElement, anchor: HudAnchor) {
        self.placements[element.index()].anchor = anchor;
    }

    /// Toggles one element and returns its new state, for the keybind action.
    pub fn toggle(&mut self, element: HudElement) -> bool {
        let index = element.index();
        self.visible[index] = !self.visible[index];
        self.visible[index]
    }

    /// Keeps scale inside what the renderer supports.
    pub fn normalise(&mut self) {
        self.scale_percent = self.scale_percent.clamp(50, 200);
        for placement in &mut self.placements {
            placement.offset.0 = placement.offset.0.clamp(-2_000, 2_000);
            placement.offset.1 = placement.offset.1.clamp(-2_000, 2_000);
        }
    }
}

/// A key combination: one key plus modifiers, e.g. `RightShift` + `H`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub key: KeyCode,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyChord {
    pub const fn key(key: KeyCode) -> Self {
        Self {
            key,
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    pub const fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn label(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        parts.push(self.key.label());
        parts.join("+")
    }

    /// Refuses a chord that would fight with vanilla controls.
    pub fn conflicts_with_vanilla(&self) -> bool {
        matches!(
            self.key,
            KeyCode::Escape | KeyCode::F3 | KeyCode::F5 | KeyCode::Tab | KeyCode::Enter
        ) && !self.shift
            && !self.ctrl
            && !self.alt
    }
}

/// The subset of keys the launcher offers. Kept small on purpose: a keybind
/// picker with 120 entries is worse than one with 40 useful ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    RightShift,
    LeftShift,
    LeftAlt,
    RightAlt,
    LeftCtrl,
    RightCtrl,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Tab,
    Enter,
    Escape,
    Space,
    Backspace,
    Grave,
}

impl KeyCode {
    pub const ALL: [Self; 50] = [
        Self::RightShift,
        Self::LeftShift,
        Self::LeftAlt,
        Self::RightAlt,
        Self::LeftCtrl,
        Self::RightCtrl,
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
        Self::I,
        Self::J,
        Self::K,
        Self::L,
        Self::M,
        Self::N,
        Self::O,
        Self::P,
        Self::Q,
        Self::R,
        Self::S,
        Self::T,
        Self::U,
        Self::V,
        Self::W,
        Self::X,
        Self::Y,
        Self::Z,
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::F5,
        Self::F6,
        Self::F7,
        Self::F8,
        Self::F9,
        Self::F10,
        Self::F11,
        Self::F12,
        Self::Tab,
        Self::Enter,
        Self::Escape,
        Self::Space,
        Self::Backspace,
        Self::Grave,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::RightShift => "Right Shift",
            Self::LeftShift => "Left Shift",
            Self::LeftAlt => "Left Alt",
            Self::RightAlt => "Right Alt",
            Self::LeftCtrl => "Left Ctrl",
            Self::RightCtrl => "Right Ctrl",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::Tab => "Tab",
            Self::Enter => "Enter",
            Self::Escape => "Escape",
            Self::Space => "Space",
            Self::Backspace => "Backspace",
            Self::Grave => "`",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeybindAction {
    /// Opens the HUD editor, the one screen that replaces every `/hud` command.
    OpenHudEditor,
    ToggleHud,
    ToggleElement(HudElement),
    /// Friend list, with requests and the chat tab.
    OpenFriends,
    /// Essential-style chat: tabs, mentions, message history.
    OpenChat,
    ToggleSkinPreview,
}

impl KeybindAction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenHudEditor => "Open HUD editor",
            Self::ToggleHud => "Toggle HUD",
            Self::ToggleElement(element) => element.label(),
            Self::OpenFriends => "Open friend list",
            Self::OpenChat => "Open chat",
            Self::ToggleSkinPreview => "Toggle skin preview",
        }
    }
}

/// An action bound to a chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keybind {
    pub action: KeybindAction,
    pub chord: KeyChord,
}

/// The complete keybind set, with defaults that never collide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeybindSet {
    pub binds: Vec<Keybind>,
}

impl Default for KeybindSet {
    fn default() -> Self {
        Self {
            binds: vec![
                Keybind {
                    action: KeybindAction::OpenHudEditor,
                    chord: KeyChord::key(KeyCode::H).with_shift(),
                },
                Keybind {
                    action: KeybindAction::ToggleHud,
                    chord: KeyChord::key(KeyCode::F1),
                },
                Keybind {
                    action: KeybindAction::OpenFriends,
                    chord: KeyChord::key(KeyCode::O),
                },
                Keybind {
                    action: KeybindAction::OpenChat,
                    chord: KeyChord::key(KeyCode::Y),
                },
                Keybind {
                    action: KeybindAction::ToggleSkinPreview,
                    chord: KeyChord::key(KeyCode::U),
                },
                Keybind {
                    action: KeybindAction::ToggleElement(HudElement::Keystrokes),
                    chord: KeyChord::key(KeyCode::K),
                },
                Keybind {
                    action: KeybindAction::ToggleElement(HudElement::Coordinates),
                    chord: KeyChord::key(KeyCode::C),
                },
            ],
        }
    }
}

impl KeybindSet {
    /// Assigns a chord. Returns `Err` with the action already holding it, so the
    /// UI can ask before stealing a bind instead of silently breaking one.
    pub fn assign(&mut self, action: KeybindAction, chord: KeyChord) -> Result<(), KeybindAction> {
        if let Some(existing) = self
            .binds
            .iter()
            .find(|bind| bind.chord == chord && bind.action != action)
        {
            return Err(existing.action);
        }
        match self.binds.iter_mut().find(|bind| bind.action == action) {
            Some(bind) => bind.chord = chord,
            None => self.binds.push(Keybind { action, chord }),
        }
        Ok(())
    }

    pub fn chord_for(&self, action: KeybindAction) -> Option<KeyChord> {
        self.binds
            .iter()
            .find(|bind| bind.action == action)
            .map(|bind| bind.chord)
    }

    /// What a pressed chord should do, if anything.
    pub fn resolve(&self, chord: KeyChord) -> Option<KeybindAction> {
        self.binds
            .iter()
            .find(|bind| bind.chord == chord)
            .map(|bind| bind.action)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Detects a config file that was hand-edited into a collision.
    pub fn find_conflict(&self) -> Option<(KeybindAction, KeybindAction)> {
        for (index, bind) in self.binds.iter().enumerate() {
            if let Some(other) = self.binds[index + 1..]
                .iter()
                .find(|other| other.chord == bind.chord)
            {
                return Some((bind.action, other.action));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_no_conflicts() {
        assert_eq!(KeybindSet::default().find_conflict(), None);
    }

    #[test]
    fn assigning_a_taken_chord_is_refused() {
        let mut set = KeybindSet::default();
        let err = set.assign(KeybindAction::OpenChat, KeyChord::key(KeyCode::O));
        assert_eq!(err, Err(KeybindAction::OpenFriends));
    }

    #[test]
    fn chord_round_trips_through_resolve() {
        let set = KeybindSet::default();
        let chord = set.chord_for(KeybindAction::OpenHudEditor).unwrap();
        assert_eq!(chord.label(), "Shift+H");
        assert_eq!(set.resolve(chord), Some(KeybindAction::OpenHudEditor));
    }

    #[test]
    fn hud_toggle_flips_state() {
        let mut hud = HudSettings::default();
        assert!(hud.is_visible(HudElement::Fps));
        assert!(!hud.toggle(HudElement::Fps));
        assert!(!hud.is_visible(HudElement::Fps));
    }

    #[test]
    fn normalise_clamps_scale() {
        let mut hud = HudSettings::default();
        // u8 tops out at 255, so the highest a scale can be set to is 255.
        hud.scale_percent = 255;
        hud.normalise();
        assert_eq!(hud.scale_percent, 200);
    }
}
