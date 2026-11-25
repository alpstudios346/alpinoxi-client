//! Authentication flow state. Provider HTTP implementations belong in later modules.

use alpinoxi_core::AccountProvider;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationState {
    Ready(AccountProvider),
    AwaitingBrowser(AccountProvider),
    Connected(AccountProvider),
    Failed(AuthenticationError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuthenticationError {
    #[error("offline profiles do not have an OAuth flow")]
    OfflineProfile,
    #[error("authentication is not configured for {0}")]
    NotConfigured(&'static str),
}

pub fn begin(provider: AccountProvider) -> AuthenticationState {
    match provider {
        AccountProvider::Offline => AuthenticationState::Failed(AuthenticationError::OfflineProfile),
        AccountProvider::Microsoft | AccountProvider::ElyBy => {
            AuthenticationState::AwaitingBrowser(provider)
        }
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
}
