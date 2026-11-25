//! Essential-style chat: tabs, unread counts, mentions and history.
//!
//! The point of a client chat is that you never lose a message you cared about.
//! Each channel keeps its own scrollback, a mention counter and unread count, so
//! the HUD can show `Friends (2)` without the player opening anything.

use alpinoxi_core::friends::ChatMessage;

/// How many lines each tab keeps. Old lines are dropped from the front.
pub const HISTORY_LIMIT: usize = 200;

/// A tab: one channel name plus its messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    pub channel: String,
    messages: Vec<ChatMessage>,
    unread: usize,
    mentions: usize,
}

impl Tab {
    pub fn new(channel: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            messages: Vec::new(),
            unread: 0,
            mentions: 0,
        }
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn unread(&self) -> usize {
        self.unread
    }

    pub fn mentions(&self) -> usize {
        self.mentions
    }

    fn push(&mut self, message: ChatMessage, active: bool) {
        if message.mentions_me {
            self.mentions += 1;
        }
        if !active {
            self.unread += 1;
        }
        self.messages.push(message);
        if self.messages.len() > HISTORY_LIMIT {
            self.messages.drain(..self.messages.len() - HISTORY_LIMIT);
        }
    }

    fn mark_read(&mut self) {
        self.unread = 0;
        self.mentions = 0;
    }

    /// The most recent line, for a compact HUD preview.
    pub fn last(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }
}

/// All chat tabs plus which one is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTabs {
    tabs: Vec<Tab>,
    active: usize,
    pub muted: bool,
}

impl Default for ChatTabs {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new("global")],
            active: 0,
            muted: false,
        }
    }
}

impl ChatTabs {
    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active(&self) -> &Tab {
        &self.tabs[self.active]
    }

    /// Switches tab and clears its badges.
    pub fn select(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
            self.tabs[index].mark_read();
        }
    }

    pub fn select_channel(&mut self, channel: &str) -> bool {
        match self.tabs.iter().position(|tab| tab.channel == channel) {
            Some(index) => {
                self.select(index);
                true
            }
            None => false,
        }
    }

    /// Adds a channel tab if it does not exist yet, e.g. a friend's DM.
    pub fn ensure(&mut self, channel: &str) -> usize {
        match self.tabs.iter().position(|tab| tab.channel == channel) {
            Some(index) => index,
            None => {
                self.tabs.push(Tab::new(channel));
                self.tabs.len() - 1
            }
        }
    }

    /// Routes a message to its channel tab. Messages for a channel the player is
    /// looking at do not count as unread.
    pub fn push(&mut self, message: ChatMessage) {
        let channel = message.channel.clone();
        let index = self.ensure(&channel);
        let active = index == self.active;
        self.tabs[index].push(message, active);
    }

    pub fn total_unread(&self) -> usize {
        self.tabs.iter().map(Tab::unread).sum()
    }

    pub fn mentions(&self) -> usize {
        self.tabs.iter().map(Tab::mentions).sum()
    }

    /// Tabs with something waiting, for the HUD label.
    pub fn badges(&self) -> Vec<(&str, usize)> {
        self.tabs
            .iter()
            .filter(|tab| tab.unread() > 0)
            .map(|tab| (tab.channel.as_str(), tab.unread()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alpinoxi_core::account::AccountProvider;
    use alpinoxi_core::friends::Friend;

    fn other() -> Friend {
        Friend::new(AccountProvider::Microsoft, "uuid-o", "Notch")
    }

    #[test]
    fn message_for_an_open_tab_is_not_unread() {
        let mut chat = ChatTabs::default();
        chat.push(ChatMessage::new(other(), "global", "hi", 1));
        assert_eq!(chat.total_unread(), 0);
        assert_eq!(chat.active().messages().len(), 1);
    }

    #[test]
    fn background_tab_accumulates_badges() {
        let mut chat = ChatTabs::default();
        chat.push(ChatMessage::new(other(), "dm:Notch", "hi", 1));
        assert_eq!(chat.total_unread(), 1);
        assert_eq!(chat.badges(), vec![("dm:Notch", 1)]);
        chat.select_channel("dm:Notch");
        assert_eq!(chat.total_unread(), 0);
    }

    #[test]
    fn history_is_bounded() {
        let mut chat = ChatTabs::default();
        for index in 0..(HISTORY_LIMIT + 50) {
            chat.push(ChatMessage::new(other(), "global", format!("line {index}"), index as u64));
        }
        assert_eq!(chat.active().messages().len(), HISTORY_LIMIT);
        assert_eq!(
            chat.active().last().unwrap().text,
            format!("line {}", HISTORY_LIMIT + 49)
        );
    }

    #[test]
    fn mentions_survive_until_the_tab_is_opened() {
        let mut chat = ChatTabs::default();
        let mut message = ChatMessage::new(other(), "dm:Notch", "ping Alpinoxi", 1);
        message.detect_mention("Alpinoxi");
        chat.push(message);
        assert_eq!(chat.mentions(), 1);
        chat.select_channel("dm:Notch");
        assert_eq!(chat.mentions(), 0);
    }
}
