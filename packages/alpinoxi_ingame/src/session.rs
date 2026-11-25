//! Ties the overlay to a running instance: who is signed in, what the game
//! reports, and what must be written back when the player leaves.

use alpinoxi_core::account::AccountProfile;
use alpinoxi_core::hud::{HudSettings, KeybindSet};
use alpinoxi_core::skin::Skin;

use crate::chat::ChatTabs;
use crate::FriendBook;

/// Everything one game session needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSession {
    pub account: AccountProfile,
    pub skin: Option<Skin>,
    pub hud: HudSettings,
    pub keybinds: KeybindSet,
    pub book: FriendBook,
    pub chat: ChatTabs,
}

impl GameSession {
    pub fn start(account: AccountProfile) -> Self {
        Self {
            account,
            skin: None,
            hud: HudSettings::default(),
            keybinds: KeybindSet::default(),
            book: FriendBook::default(),
            chat: ChatTabs::default(),
        }
    }

    /// Called on quit: the settings the player changed in game are handed back
    /// so the launcher persists them.
    pub fn finish(self) -> (HudSettings, KeybindSet) {
        (self.hud, self.keybinds)
    }

    /// A one-line summary for the launcher's status bar.
    pub fn summary(&self) -> String {
        format!(
            "{} · {} friends online · {} unread",
            self.account.badge(),
            self.book
                .friends()
                .iter()
                .filter(|friend| self.book.presence_of(friend).is_online())
                .count(),
            self.chat.total_unread()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alpinoxi_core::account::AccountProvider;

    #[test]
    fn session_summary_counts_online_friends() {
        let account = AccountProfile::new(AccountProvider::Offline, "Alpinoxi", "uuid");
        let session = GameSession::start(account);
        assert!(session.summary().contains("Alpinoxi · Cracked"));
        assert!(session.summary().contains("0 friends online"));
    }
}
