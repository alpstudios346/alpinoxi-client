//! Ely.by OAuth 2.0 authorization-code flow with PKCE.
//!
//! Alpinoxi registers as an Ely.by *Website* application with the client id
//! `alpinoxi-client`. A desktop binary cannot keep a client secret, so the flow
//! uses PKCE (S256) and a loopback redirect, which is the pattern OAuth 2.0 for
//! native apps recommends.
//!
//! Reference: <https://docs.ely.by/en/oauth.html>

use serde::Deserialize;

use crate::token::{code_challenge, random_urlsafe, TokenSet};
use crate::{AuthenticationError, ContentType, HttpRequest, HttpResponse, Method, ELYBY_CLIENT_ID};

pub const AUTHORIZE_URL: &str = "https://account.ely.by/oauth2/v1";
pub const TOKEN_URL: &str = "https://account.ely.by/api/oauth2/v1/token";
pub const ACCOUNT_INFO_URL: &str = "https://account.ely.by/api/account/v1/info";
pub const SKIN_URL: &str = "http://skinsystem.ely.by/skins/{name}.png";

/// Scopes Alpinoxi needs: the account, and a Minecraft session so the game can
/// join Ely.by-aware servers.
pub const SCOPE: &str = "account_info offline_access minecraft_server_session";

/// A pending authorization. Keep the verifier in memory only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElyByFlow {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub code_verifier: String,
}

impl ElyByFlow {
    /// Starts a flow for a loopback redirect such as
    /// `http://127.0.0.1:42800/callback`.
    pub fn start(redirect_uri: &str) -> Self {
        Self {
            client_id: ELYBY_CLIENT_ID.to_owned(),
            redirect_uri: redirect_uri.to_owned(),
            state: random_urlsafe(32),
            code_verifier: random_urlsafe(64),
        }
    }

    /// The URL to open in the player's browser.
    pub fn authorize_url(&self) -> String {
        format!(
            "{AUTHORIZE_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256&description=Alpinoxi+Client",
            urlencode(&self.client_id),
            urlencode(&self.redirect_uri),
            urlencode(SCOPE),
            urlencode(&self.state),
            code_challenge(&self.code_verifier),
        )
    }

    /// Handles the loopback redirect: verifies `state` and extracts the code.
    pub fn accept_redirect(&self, query: &str) -> Result<String, AuthenticationError> {
        let params = parse_query(query);
        if let Some(error) = params.iter().find(|(key, _)| key == "error") {
            return Err(AuthenticationError::Provider(error.1.clone()));
        }
        let state = params
            .iter()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.as_str());
        if state != Some(self.state.as_str()) {
            return Err(AuthenticationError::Provider(
                "the sign-in response did not match this request".to_owned(),
            ));
        }
        params
            .into_iter()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value)
            .ok_or(AuthenticationError::MalformedResponse("code"))
    }

    /// Exchanges the code for tokens. Public client: no `client_secret`.
    pub fn token_request(&self, code: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Post,
            url: TOKEN_URL.to_owned(),
            content_type: ContentType::Form,
            body: format!(
                "client_id={}&grant_type=authorization_code&redirect_uri={}&code={}&code_verifier={}",
                urlencode(&self.client_id),
                urlencode(&self.redirect_uri),
                urlencode(code),
                urlencode(&self.code_verifier),
            ),
            bearer: None,
        }
    }

    /// Refresh request, used before every launch once the access token ages out.
    pub fn refresh_request(&self, refresh_token: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Post,
            url: TOKEN_URL.to_owned(),
            content_type: ContentType::Form,
            body: format!(
                "client_id={}&grant_type=refresh_token&refresh_token={}&scope={}",
                urlencode(&self.client_id),
                urlencode(refresh_token),
                urlencode(SCOPE),
            ),
            bearer: None,
        }
    }
}

/// Request for `account_info`.
pub fn account_info_request(access_token: &str) -> HttpRequest {
    HttpRequest {
        method: Method::Get,
        url: ACCOUNT_INFO_URL.to_owned(),
        content_type: ContentType::Json,
        body: String::new(),
        bearer: Some(access_token.to_owned()),
    }
}

#[derive(Deserialize)]
struct ElyAccount {
    uuid: String,
    username: String,
}

/// Parses `/api/account/v1/info` into `(uuid, name)`.
pub fn parse_account_info(
    response: &HttpResponse,
) -> Result<(String, String), AuthenticationError> {
    if !response.ok() {
        return Err(AuthenticationError::Provider(format!(
            "Ely.by account request returned status {}",
            response.status
        )));
    }
    let parsed: ElyAccount = serde_json::from_str(&response.body)
        .map_err(|_| AuthenticationError::MalformedResponse("uuid"))?;
    Ok((parsed.uuid, parsed.username))
}

/// Parses a token response from either the code exchange or a refresh.
pub fn parse_tokens(response: &HttpResponse) -> Result<TokenSet, AuthenticationError> {
    if !response.ok() {
        return Err(AuthenticationError::Provider(format!(
            "Ely.by token request returned status {}",
            response.status
        )));
    }
    serde_json::from_str(&response.body)
        .map_err(|_| AuthenticationError::MalformedResponse("access_token"))
}

/// Skin URL for a name; Ely.by falls back to the premium texture when needed.
pub fn skin_url(name: &str) -> String {
    SKIN_URL.replace("{name}", &urlencode(name))
}

fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(key), percent_decode(value))
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    Err(_) => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_url_carries_client_id_state_and_challenge() {
        let flow = ElyByFlow::start("http://127.0.0.1:42800/callback");
        let url = flow.authorize_url();
        assert!(url.contains("client_id=alpinoxi-client"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains(&format!("state={}", flow.state)));
        assert!(url.contains("scope=account_info%20offline_access%20minecraft_server_session"));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A42800%2Fcallback"));
    }

    #[test]
    fn redirect_state_must_match() {
        let flow = ElyByFlow::start("http://127.0.0.1:42800/callback");
        assert!(flow.accept_redirect("?code=abc&state=wrong").is_err());
        assert_eq!(
            flow.accept_redirect(&format!("?code=abc&state={}", flow.state)),
            Ok("abc".to_owned())
        );
        assert!(flow.accept_redirect("?error=access_denied").is_err());
    }

    #[test]
    fn token_request_has_no_secret() {
        let flow = ElyByFlow::start("http://127.0.0.1:42800/callback");
        let request = flow.token_request("the-code");
        assert!(!request.body.contains("client_secret"));
        assert!(request.body.contains("grant_type=authorization_code"));
        assert!(request.body.contains("code_verifier="));
    }

    #[test]
    fn account_info_parses() {
        let response = HttpResponse {
            status: 200,
            body: r#"{"id":1,"uuid":"ffc8fdc9-5824-509e-8a57-c99b940fb996","username":"ErickSkrauch"}"#
                .to_owned(),
        };
        assert_eq!(
            parse_account_info(&response),
            Ok((
                "ffc8fdc9-5824-509e-8a57-c99b940fb996".to_owned(),
                "ErickSkrauch".to_owned()
            ))
        );
    }

    #[test]
    fn query_decoding_handles_percent_and_plus() {
        assert_eq!(percent_decode("a%20b+c"), "a b c");
    }
}
