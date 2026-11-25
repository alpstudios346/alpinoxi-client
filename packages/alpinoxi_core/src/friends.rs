//! Friend list, requests and chat.
//!
//! The friend system works for all three login types. Identity is
//! `(provider, uuid)`, so the same person can be reached whether they signed in
//! with Microsoft, Ely.by or as a cracked profile, and the UI can show a badge
//! next to each friend so nobody has to guess where a request came from.

use serde::{Deserialize, Serialize};

use crate::account::AccountProvider;

/// A friend identity. `uuid` is the dashed UUID from the provider (Microsoft /
/// Ely.by) or the offline UUID Alpinoxi derives for cracked names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Friend {
    pub provider: AccountProvider,
    pub uuid: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default)]
    pub favourite: bool,
}

impl Friend {
    pub fn new(provider: AccountProvider, uuid: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            provider,
            uuid: uuid.into(),
            name: name.into(),
            note: None,
            favourite: false,
        }
    }

    /// Unique key for maps and de-duplication.
    pub fn key(&self) -> String {
        format!("{}:{}", self.provider.id(), self.uuid)
    }

    /// `Name [Ely.by]` for compact lists.
    pub fn display(&self) -> String {
        format!("{} [{}]", self.name, self.provider.label())
    }
}

/// What a friend is doing right now. The in-game bridge pushes these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Presence {
    Offline,
    /// Logged into the launcher but not in a world.
    InLauncher,
    /// In game, optionally on a server.
    InGame { server: Option<String> },
}

impl Default for Presence {
    fn default() -> Self {
        Self::Offline
    }
}

impl Presence {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Offline => "Offline",
            Self::InLauncher => "In launcher",
            Self::InGame { .. } => "In game",
        }
    }

    pub const fn is_online(&self) -> bool {
        !matches!(self, Self::Offline)
    }
}

/// Request lifecycle. Kept explicit so the UI can show sent, received and
/// declined states instead of a single opaque "pending" flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FriendState {
    Pending,
    Accepted,
    Declined,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendRequest {
    pub from: Friend,
    pub to: Friend,
    pub state: FriendState,
    /// Unix seconds.
    pub created_at: u64,
}

impl FriendRequest {
    pub fn new(from: Friend, to: Friend, created_at: u64) -> Self {
        Self {
            from,
            to,
            state: FriendState::Pending,
            created_at,
        }
    }

    /// The other side of a request relative to `me`.
    pub fn counterpart(&self, me: &Friend) -> &Friend {
        if self.from.key() == me.key() {
            &self.to
        } else {
            &self.from
        }
    }

    pub fn is_incoming(&self, me: &Friend) -> bool {
        self.from.key() != me.key()
    }
}

/// One chat message. `channel` is the tab it belongs to (global, a friend, or a
/// server), which is what makes the chat feel like an essential-style client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub from: Friend,
    pub channel: String,
    pub text: String,
    pub timestamp: u64,
    /// Mentions of your own name are highlighted instead of being missed.
    #[serde(default)]
    pub mentions_me: bool,
}

impl ChatMessage {
    pub fn new(from: Friend, channel: impl Into<String>, text: impl Into<String>, timestamp: u64) -> Self {
        Self {
            from,
            channel: channel.into(),
            text: text.into(),
            timestamp,
            mentions_me: false,
        }
    }

    /// Detects `@name` or the bare name, case-insensitively.
    pub fn detect_mention(&mut self, my_name: &str) {
        let needle = my_name.to_ascii_lowercase();
        let haystack = self.text.to_ascii_lowercase();
        self.mentions_me = !needle.is_empty()
            && haystack
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .any(|word| word == needle);
    }

    /// Rejects messages the chat can never render usefully.
    pub fn sanitise(text: &str) -> Result<String, ChatError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ChatError::Empty);
        }
        if trimmed.chars().count() > 256 {
            return Err(ChatError::TooLong);
        }
        if trimmed.chars().any(|c| c.is_control() && c != '\t') {
            return Err(ChatError::ControlCharacters);
        }
        Ok(trimmed.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChatError {
    #[error("message is empty")]
    Empty,
    #[error("messages are limited to 256 characters")]
    TooLong,
    #[error("message contains control characters")]
    ControlCharacters,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn me() -> Friend {
        Friend::new(AccountProvider::ElyBy, "uuid-me", "Alpinoxi")
    }

    #[test]
    fn friend_key_separates_providers() {
        let a = Friend::new(AccountProvider::Microsoft, "same", "Alex");
        let b = Friend::new(AccountProvider::Offline, "same", "Alex");
        assert_ne!(a.key(), b.key());
    }

    #[test]
    fn mention_detection_is_word_based() {
        let mut message = ChatMessage::new(me(), "global", "hey alpinoxi2, look", 0);
        message.detect_mention("Alpinoxi");
        assert!(!message.mentions_me);

        let mut message = ChatMessage::new(me(), "global", "@Alpinoxi look", 0);
        message.detect_mention("alpinoxi");
        assert!(message.mentions_me);
    }

    #[test]
    fn request_counterpart_flips_with_direction() {
        let other = Friend::new(AccountProvider::Microsoft, "uuid-other", "Notch");
        let incoming = FriendRequest::new(other.clone(), me(), 1);
        assert!(incoming.is_incoming(&me()));
        assert_eq!(incoming.counterpart(&me()), &other);

        let outgoing = FriendRequest::new(me(), other.clone(), 1);
        assert!(!outgoing.is_incoming(&me()));
        assert_eq!(outgoing.counterpart(&me()), &other);
    }

    #[test]
    fn chat_sanitising_blocks_junk() {
        assert_eq!(ChatMessage::sanitise("  "), Err(ChatError::Empty));
        assert_eq!(ChatMessage::sanitise("ok"), Ok("ok".to_owned()));
        assert!(ChatMessage::sanitise(&"a".repeat(300)).is_err());
        assert!(ChatMessage::sanitise("bad\u{7}").is_err());
    }
}
