//! The in-game half of Alpinoxi: HUD, keybinds, chat and friend presence.
//!
//! This crate holds the logic, not the renderer. It reads the same
//! `HudSettings`/`KeybindSet` the launcher writes, turns a key press into an
//! action, keeps the chat and friend list in memory, and talks to the game
//! through the `GameBridge` trait. That keeps the rules testable without a
//! Minecraft install and lets the renderer be swapped later.
//!
//! The key design point: **no commands**. Opening the HUD editor, toggling an
//! element, opening friends or chat is always a keybind, and the chat itself
//! supports `@mentions` and tabs.

use alpinoxi_core::friends::{ChatError, ChatMessage, Friend, FriendRequest, FriendState, Presence};
use alpinoxi_core::hud::{HudElement, HudSettings, KeyChord, KeybindAction, KeybindSet};
use alpinoxi_core::skin::Skin;

pub mod chat;
pub mod session;

pub use chat::ChatTabs;
pub use session::GameSession;

/// What the renderer must do after a key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionOutcome {
    /// Open the HUD editor screen.
    OpenHudEditor,
    /// HUD shown/hidden.
    HudVisible(bool),
    /// One element flipped; `visible` is its new state.
    ElementToggled(HudElement, bool),
    OpenFriends,
    OpenChat,
    SkinPreviewVisible(bool),
    /// The chord is not bound.
    Nothing,
}

/// The game side. A renderer (Fabric mod, or a future native client) implements
/// this; the logic above never calls into the game directly.
pub trait GameBridge {
    fn local_player_name(&self) -> String;
    fn current_server(&self) -> Option<String>;
    fn show_hud(&mut self, visible: bool);
    fn open_hud_editor(&mut self);
    fn open_friend_list(&mut self);
    fn open_chat(&mut self);
    fn set_skin_preview(&mut self, visible: bool);
    fn push_message(&mut self, channel: &str, text: &str);
}

/// Applies a chord to the settings and returns what the game should do.
pub fn handle_key(
    chord: KeyChord,
    hud: &mut HudSettings,
    keybinds: &KeybindSet,
    bridge: &mut dyn GameBridge,
) -> ActionOutcome {
    let Some(action) = keybinds.resolve(chord) else {
        return ActionOutcome::Nothing;
    };
    match action {
        KeybindAction::OpenHudEditor => {
            bridge.open_hud_editor();
            ActionOutcome::OpenHudEditor
        }
        KeybindAction::ToggleHud => {
            hud.enabled = !hud.enabled;
            bridge.show_hud(hud.enabled);
            ActionOutcome::HudVisible(hud.enabled)
        }
        KeybindAction::ToggleElement(element) => {
            let visible = hud.toggle(element);
            ActionOutcome::ElementToggled(element, visible)
        }
        KeybindAction::OpenFriends => {
            bridge.open_friend_list();
            ActionOutcome::OpenFriends
        }
        KeybindAction::OpenChat => {
            bridge.open_chat();
            ActionOutcome::OpenChat
        }
        KeybindAction::ToggleSkinPreview => {
            let visible = !hud.is_visible(HudElement::Friends);
            bridge.set_skin_preview(visible);
            ActionOutcome::SkinPreviewVisible(visible)
        }
    }
}

/// Friend book: requests, accepted friends and presence, keyed by
/// `provider:uuid` so one person cannot appear twice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FriendBook {
    pub me: Option<Friend>,
    friends: Vec<Friend>,
    requests: Vec<FriendRequest>,
    presence: Vec<(String, Presence)>,
}

impl FriendBook {
    pub fn sign_in(&mut self, me: Friend) {
        self.me = Some(me);
    }

    pub fn send_request(&mut self, to: Friend, now: u64) -> Result<(), FriendError> {
        let me = self.me.clone().ok_or(FriendError::NotSignedIn)?;
        if me.key() == to.key() {
            return Err(FriendError::SelfRequest);
        }
        if self.is_friend(&to) {
            return Err(FriendError::AlreadyFriends);
        }
        if self
            .requests
            .iter()
            .any(|request| request.to.key() == to.key() && request.state == FriendState::Pending)
        {
            return Err(FriendError::AlreadyPending);
        }
        self.requests.push(FriendRequest::new(me, to, now));
        Ok(())
    }

    pub fn accept(&mut self, counterpart: &Friend) -> Result<(), FriendError> {
        let request = self
            .requests
            .iter_mut()
            .find(|request| {
                request.state == FriendState::Pending
                    && request.counterpart(self.me.as_ref().ok_or(FriendError::NotSignedIn)?)
                        == counterpart
            })
            .ok_or(FriendError::NoSuchRequest)?;
        request.state = FriendState::Accepted;
        let friend = counterpart.clone();
        if !self.is_friend(&friend) {
            self.friends.push(friend);
        }
        Ok(())
    }

    pub fn decline(&mut self, counterpart: &Friend) -> Result<(), FriendError> {
        let me = self.me.clone().ok_or(FriendError::NotSignedIn)?;
        let request = self
            .requests
            .iter_mut()
            .find(|request| {
                request.state == FriendState::Pending && request.counterpart(&me) == counterpart
            })
            .ok_or(FriendError::NoSuchRequest)?;
        request.state = FriendState::Declined;
        Ok(())
    }

    pub fn remove(&mut self, friend: &Friend) {
        self.friends.retain(|existing| existing.key() != friend.key());
        self.presence.retain(|(key, _)| key != &friend.key());
    }

    pub fn is_friend(&self, friend: &Friend) -> bool {
        self.friends.iter().any(|existing| existing.key() == friend.key())
    }

    pub fn friends(&self) -> &[Friend] {
        &self.friends
    }

    pub fn pending(&self) -> Vec<&FriendRequest> {
        self.requests
            .iter()
            .filter(|request| request.state == FriendState::Pending)
            .collect()
    }

    pub fn set_presence(&mut self, friend: &Friend, presence: Presence) {
        let key = friend.key();
        match self.presence.iter_mut().find(|(existing, _)| existing == &key) {
            Some(entry) => entry.1 = presence,
            None => self.presence.push((key, presence)),
        }
    }

    pub fn presence_of(&self, friend: &Friend) -> Presence {
        self.presence
            .iter()
            .find(|(key, _)| key == &friend.key())
            .map(|(_, presence)| presence.clone())
            .unwrap_or_default()
    }

    /// Friends sorted the way the panel shows them: online first, then
    /// favourites, then name.
    pub fn sorted(&self) -> Vec<(&Friend, Presence)> {
        let mut rows: Vec<(&Friend, Presence)> = self
            .friends
            .iter()
            .map(|friend| (friend, self.presence_of(friend)))
            .collect();
        rows.sort_by(|(left, left_presence), (right, right_presence)| {
            right_presence
                .is_online()
                .cmp(&left_presence.is_online())
                .then(right.favourite.cmp(&left.favourite))
                .then_with(|| left.name.to_ascii_lowercase().cmp(&right.name.to_ascii_lowercase()))
        });
        rows
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FriendError {
    #[error("sign in before adding friends")]
    NotSignedIn,
    #[error("you cannot add yourself")]
    SelfRequest,
    #[error("this player is already on your friend list")]
    AlreadyFriends,
    #[error("a request to this player is already pending")]
    AlreadyPending,
    #[error("that friend request no longer exists")]
    NoSuchRequest,
}

/// The in-game overlay state: everything the HUD editor and the side panels need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overlay {
    pub hud: HudSettings,
    pub keybinds: KeybindSet,
    pub book: FriendBook,
    pub chat: ChatTabs,
    pub skin_preview: bool,
    pub skin: Option<Skin>,
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            hud: HudSettings::default(),
            keybinds: KeybindSet::default(),
            book: FriendBook::default(),
            chat: ChatTabs::default(),
            skin_preview: false,
            skin: None,
        }
    }
}

impl Overlay {
    /// Incoming chat, routed into the right tab and mentioning the local player.
    pub fn receive(&mut self, mut message: ChatMessage, my_name: &str) -> Result<(), ChatError> {
        message.text = ChatMessage::sanitise(&message.text)?;
        message.detect_mention(my_name);
        self.chat.push(message);
        Ok(())
    }

    /// The player typed a line in the chat box.
    pub fn send(
        &mut self,
        from: Friend,
        channel: &str,
        text: &str,
        now: u64,
        bridge: &mut dyn GameBridge,
    ) -> Result<(), ChatError> {
        let clean = ChatMessage::sanitise(text)?;
        let message = ChatMessage::new(from, channel, clean, now);
        self.chat.push(message);
        bridge.push_message(channel, text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alpinoxi_core::account::AccountProvider;

    #[derive(Default)]
    struct Recorder {
        hud_visible: bool,
        opened_editor: bool,
        opened_friends: bool,
        opened_chat: bool,
        pushed: Vec<(String, String)>,
    }

    impl GameBridge for Recorder {
        fn local_player_name(&self) -> String {
            "Alpinoxi".to_owned()
        }
        fn current_server(&self) -> Option<String> {
            None
        }
        fn show_hud(&mut self, visible: bool) {
            self.hud_visible = visible;
        }
        fn open_hud_editor(&mut self) {
            self.opened_editor = true;
        }
        fn open_friend_list(&mut self) {
            self.opened_friends = true;
        }
        fn open_chat(&mut self) {
            self.opened_chat = true;
        }
        fn set_skin_preview(&mut self, _visible: bool) {}
        fn push_message(&mut self, channel: &str, text: &str) {
            self.pushed
                .push((channel.to_owned(), text.to_owned()));
        }
    }

    fn me() -> Friend {
        Friend::new(AccountProvider::Offline, "uuid-me", "Alpinoxi")
    }

    #[test]
    fn keybind_opens_hud_editor_without_a_command() {
        let mut hud = HudSettings::default();
        let binds = KeybindSet::default();
        let mut bridge = Recorder::default();
        let chord = binds.chord_for(KeybindAction::OpenHudEditor).unwrap();
        assert_eq!(
            handle_key(chord, &mut hud, &binds, &mut bridge),
            ActionOutcome::OpenHudEditor
        );
        assert!(bridge.opened_editor);
    }

    #[test]
    fn unbound_chord_does_nothing() {
        let mut hud = HudSettings::default();
        let binds = KeybindSet::default();
        let mut bridge = Recorder::default();
        assert_eq!(
            handle_key(KeyChord::key(alpinoxi_core::hud::KeyCode::F9), &mut hud, &binds, &mut bridge),
            ActionOutcome::Nothing
        );
    }

    #[test]
    fn friend_flow_accepts_and_sorts() {
        let mut book = FriendBook::default();
        book.sign_in(me());
        let other = Friend::new(AccountProvider::Microsoft, "uuid-other", "Notch");
        book.send_request(other.clone(), 1).unwrap();
        assert_eq!(book.send_request(other.clone(), 2), Err(FriendError::AlreadyPending));
        book.accept(&other).unwrap();
        assert!(book.is_friend(&other));
        book.set_presence(&other, Presence::InGame { server: Some("play.example".to_owned()) });
        assert!(book.presence_of(&other).is_online());
        assert_eq!(book.sorted().len(), 1);
    }

    #[test]
    fn chat_routes_and_flags_mentions() {
        let mut overlay = Overlay::default();
        let other = Friend::new(AccountProvider::ElyBy, "uuid-o", "Erick");
        let message = ChatMessage::new(other, "global", "@Alpinoxi hi", 5);
        overlay.receive(message, "Alpinoxi").unwrap();
        assert_eq!(overlay.chat.mentions(), 1);
    }

    #[test]
    fn sending_strips_junk_and_reaches_the_game() {
        let mut overlay = Overlay::default();
        let mut bridge = Recorder::default();
        overlay
            .send(me(), "global", "  hello  ", 1, &mut bridge)
            .unwrap();
        assert_eq!(bridge.pushed, vec![("global".to_owned(), "  hello  ".to_owned())]);
        assert!(overlay.send(me(), "global", "   ", 2, &mut bridge).is_err());
    }
}
