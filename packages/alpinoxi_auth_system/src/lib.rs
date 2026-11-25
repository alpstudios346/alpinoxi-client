//! Authentication for the three Alpinoxi login types.
//!
//! * **Microsoft** — official OAuth 2.0 device-code flow, then Xbox Live, XSTS
//!   and `login_with_xbox` for the Minecraft token.
//! * **Ely.by** — OAuth 2.0 authorization-code flow with client id
//!   `alpinoxi-client` and PKCE, since a desktop app cannot keep a client secret.
//! * **Cracked** — a local name only, with the vanilla offline UUID.
//!
//! The rules from `docs/AUTHENTICATION.md` are enforced in code, not by
//! convention: this crate never stores a password, never logs a token, and hands
//! the UI an `AccountProfile` only after a flow succeeded. Network calls are
//! described as typed requests so the flow stays testable and the HTTP layer
//! stays replaceable.

use alpinoxi_core::{AccountProfile, AccountProvider, CrackedAccountError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod elyby;
pub mod microsoft;
pub mod token;
pub mod vault;

pub use elyby::ElyByFlow;
pub use microsoft::MicrosoftFlow;
pub use token::{code_challenge, random_urlsafe, TokenSet};
pub use vault::{MemoryVault, TokenVault};

/// The two official providers plus the local one.
pub const ELYBY_CLIENT_ID: &str = "alpinoxi-client";

/// Progress of a sign-in, as the login screen renders it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationState {
    /// Nothing started yet.
    Idle,
    /// The player must finish something in the browser.
    AwaitingBrowser { url: String, user_code: Option<String> },
    /// Flow completed and a profile is available.
    Connected(AccountProfile),
    /// Flow failed; the message is safe to show (never contains a token).
    Failed(AuthenticationError),
}

impl AuthenticationState {
    pub fn is_busy(&self) -> bool {
        matches!(self, Self::AwaitingBrowser { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthenticationError {
    #[error("offline profiles do not have an OAuth flow")]
    OfflineProfile,
    #[error("authentication is not configured for {0}")]
    NotConfigured(&'static str),
    #[error("the provider returned an error: {0}")]
    Provider(String),
    #[error("the sign-in code expired, please try again")]
    CodeExpired,
    #[error("Microsoft did not grant Minecraft access for this account")]
    NoMinecraftEntitlement,
    #[error("the token response was missing a field: {0}")]
    MalformedResponse(&'static str),
    #[error("player name problem: {0}")]
    Cracked(#[from] CrackedAccountError),
}

impl AuthenticationError {
    /// Errors that only mean "not set up yet", so the UI can offer guidance
    /// instead of an alarm.
    pub fn is_setup_issue(&self) -> bool {
        matches!(self, Self::NotConfigured(_) | Self::OfflineProfile)
    }
}

/// One HTTP request the flow needs. `body` is either form-encoded
/// (`application/x-www-form-urlencoded`) or JSON, matching `content_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub content_type: ContentType,
    pub body: String,
    pub bearer: Option<String>,
}

impl HttpRequest {
    /// Debug view with every secret stripped, so a request can be logged safely.
    pub fn redacted(&self) -> Self {
        let mut clone = self.clone();
        clone.body = token::redact_body(&clone.body);
        clone.bearer = clone.bearer.map(|_| "<redacted>".to_owned());
        clone
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Json,
    Form,
}

/// What the HTTP layer must return for a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

impl HttpResponse {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Starts a sign-in. Cracked needs no network at all.
pub fn begin(provider: AccountProvider) -> AuthenticationState {
    match provider {
        AccountProvider::Offline => AuthenticationState::Failed(AuthenticationError::OfflineProfile),
        AccountProvider::Microsoft | AccountProvider::ElyBy => AuthenticationState::Idle,
    }
}

/// Signs in a cracked player: validates the name and derives the offline UUID.
pub fn begin_cracked(name: &str) -> AuthenticationState {
    match AccountProfile::cracked(name) {
        Ok(profile) => AuthenticationState::Connected(profile),
        Err(error) => AuthenticationState::Failed(AuthenticationError::Cracked(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_does_not_offer_oauth() {
        assert_eq!(
            begin(AccountProvider::Offline),
            AuthenticationState::Failed(AuthenticationError::OfflineProfile)
        );
    }

    #[test]
    fn cracked_login_is_immediate_and_offline() {
        match begin_cracked("Alpinoxi") {
            AuthenticationState::Connected(profile) => {
                assert_eq!(profile.provider, AccountProvider::Offline);
                assert!(!profile.is_online());
            }
            other => panic!("expected a connected profile, got {other:?}"),
        }
    }

    #[test]
    fn redaction_hides_secrets() {
        let request = HttpRequest {
            method: Method::Post,
            url: "https://example.invalid/token".to_owned(),
            content_type: ContentType::Form,
            body: "client_id=x&refresh_token=SUPERSECRET".to_owned(),
            bearer: Some("SUPERSECRET".to_owned()),
        };
        let redacted = request.redacted();
        assert!(!redacted.body.contains("SUPERSECRET"));
        assert!(!redacted.bearer.unwrap().contains("SUPERSECRET"));
    }
}
