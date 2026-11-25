//! Microsoft → Xbox Live → XSTS → Minecraft sign-in.
//!
//! Uses the OAuth 2.0 **device code** flow, which is the right fit for a desktop
//! launcher: the player opens a short URL, types a code, and no local redirect
//! server, embedded browser or password entry is needed.
//!
//! Endpoints are the documented Minecraft authentication chain:
//! <https://minecraft.wiki/w/Microsoft_authentication>

use serde::Deserialize;

use crate::token::TokenSet;
use crate::{
    AuthenticationError, ContentType, HttpRequest, HttpResponse, Method,
};

pub const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
pub const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
pub const XBL_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
pub const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
pub const MINECRAFT_LOGIN_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
pub const ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";
pub const PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Scope Minecraft needs from Microsoft.
pub const SCOPE: &str = "XboxLive.signin offline_access";

/// What the player sees while the device-code flow waits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval_seconds: u64,
    pub expires_in: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicrosoftFlow {
    /// Azure application (client) id of the Alpinoxi registration.
    pub client_id: String,
    pub device: DeviceCode,
}

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default = "default_interval")]
    interval: u64,
    expires_in: u64,
}

fn default_interval() -> u64 {
    5
}

impl MicrosoftFlow {
    /// Request 1: start the device flow.
    pub fn start_request(client_id: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Post,
            url: DEVICE_CODE_URL.to_owned(),
            content_type: ContentType::Form,
            body: format!("client_id={client_id}&scope={}", SCOPE.replace(' ', "+")),
            bearer: None,
        }
    }

    /// Parses the device-code response into a flow ready for polling.
    pub fn from_device_response(
        client_id: &str,
        response: &HttpResponse,
    ) -> Result<Self, AuthenticationError> {
        if !response.ok() {
            return Err(AuthenticationError::Provider(format!(
                "Microsoft device code request failed with status {}",
                response.status
            )));
        }
        let parsed: DeviceCodeResponse = serde_json::from_str(&response.body)
            .map_err(|_| AuthenticationError::MalformedResponse("device_code"))?;
        Ok(Self {
            client_id: client_id.to_owned(),
            device: DeviceCode {
                device_code: parsed.device_code,
                user_code: parsed.user_code,
                verification_uri: parsed.verification_uri,
                interval_seconds: parsed.interval.max(1),
                expires_in: parsed.expires_in,
            },
        })
    }

    /// Request 2: the poll. `authorization_pending` is not an error; the UI just
    /// waits `interval_seconds` and calls this again.
    pub fn poll_request(&self) -> HttpRequest {
        HttpRequest {
            method: Method::Post,
            url: TOKEN_URL.to_owned(),
            content_type: ContentType::Form,
            body: format!(
                "client_id={}&grant_type=urn:ietf:params:oauth:grant-type:device_code&device_code={}",
                self.client_id, self.device.device_code
            ),
            bearer: None,
        }
    }

    /// Interprets a poll response.
    pub fn poll_outcome(response: &HttpResponse) -> Result<PollOutcome, AuthenticationError> {
        if response.ok() {
            let tokens: TokenSet = serde_json::from_str(&response.body)
                .map_err(|_| AuthenticationError::MalformedResponse("access_token"))?;
            return Ok(PollOutcome::Authorized(tokens));
        }
        let code = error_code(&response.body);
        match code.as_deref() {
            Some("authorization_pending") | Some("slow_down") => Ok(PollOutcome::Pending),
            Some("expired_token") | Some("code_expired") => Err(AuthenticationError::CodeExpired),
            Some("authorization_declined") => Err(AuthenticationError::Provider(
                "sign-in was declined in the browser".to_owned(),
            )),
            _ => Err(AuthenticationError::Provider(format!(
                "Microsoft returned status {}",
                response.status
            ))),
        }
    }

    /// Request 3: Xbox Live user authentication (RPS ticket `d=` for MSA).
    pub fn xbl_request(ms_access_token: &str) -> HttpRequest {
        let body = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_access_token}")
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        })
        .to_string();
        HttpRequest {
            method: Method::Post,
            url: XBL_URL.to_owned(),
            content_type: ContentType::Json,
            body,
            bearer: None,
        }
    }

    /// Request 4: XSTS token for `rp://api.minecraftservices.com/`.
    pub fn xsts_request(xbl_token: &str) -> HttpRequest {
        let body = serde_json::json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        })
        .to_string();
        HttpRequest {
            method: Method::Post,
            url: XSTS_URL.to_owned(),
            content_type: ContentType::Json,
            body,
            bearer: None,
        }
    }

    /// Request 5: exchange the XSTS token for a Minecraft bearer token.
    pub fn minecraft_login_request(user_hash: &str, xsts_token: &str) -> HttpRequest {
        let body = serde_json::json!({
            "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
        })
        .to_string();
        HttpRequest {
            method: Method::Post,
            url: MINECRAFT_LOGIN_URL.to_owned(),
            content_type: ContentType::Json,
            body,
            bearer: None,
        }
    }

    /// Request 6: the profile, which carries the real name and UUID.
    pub fn profile_request(minecraft_token: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Get,
            url: PROFILE_URL.to_owned(),
            content_type: ContentType::Json,
            body: String::new(),
            bearer: Some(minecraft_token.to_owned()),
        }
    }

    /// Request 7: entitlement check, so "no Minecraft on this account" is caught
    /// before launch instead of as a mysterious disconnect.
    pub fn entitlements_request(minecraft_token: &str) -> HttpRequest {
        HttpRequest {
            method: Method::Get,
            url: ENTITLEMENTS_URL.to_owned(),
            content_type: ContentType::Json,
            body: String::new(),
            bearer: Some(minecraft_token.to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollOutcome {
    Pending,
    Authorized(TokenSet),
}

#[derive(Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Deserialize)]
struct Xui {
    uhs: String,
}

/// Pulls `Token` and `uhs` out of an XBL/XSTS response.
pub fn parse_xbl(response: &HttpResponse) -> Result<(String, String), AuthenticationError> {
    if !response.ok() {
        return Err(AuthenticationError::Provider(format!(
            "Xbox Live returned status {}",
            response.status
        )));
    }
    let parsed: XblResponse = serde_json::from_str(&response.body)
        .map_err(|_| AuthenticationError::MalformedResponse("Token"))?;
    let uhs = parsed
        .display_claims
        .xui
        .first()
        .map(|xui| xui.uhs.clone())
        .ok_or(AuthenticationError::MalformedResponse("uhs"))?;
    Ok((parsed.token, uhs))
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

/// Parses `api.minecraftservices.com/minecraft/profile`.
pub fn parse_profile(
    response: &HttpResponse,
) -> Result<(String, String), AuthenticationError> {
    if response.status == 404 {
        return Err(AuthenticationError::NoMinecraftEntitlement);
    }
    if !response.ok() {
        return Err(AuthenticationError::Provider(format!(
            "Minecraft profile returned status {}",
            response.status
        )));
    }
    let parsed: MinecraftProfile = serde_json::from_str(&response.body)
        .map_err(|_| AuthenticationError::MalformedResponse("id"))?;
    Ok((parsed.id, parsed.name))
}

/// Whether an entitlements response contains a game licence.
pub fn has_entitlement(response: &HttpResponse) -> bool {
    response.ok()
        && response
            .body
            .to_ascii_lowercase()
            .contains("product_minecraft")
}

fn error_code(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value
        .get("error")
        .and_then(|error| error.as_str())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: u16, body: &str) -> HttpResponse {
        HttpResponse {
            status,
            body: body.to_owned(),
        }
    }

    #[test]
    fn device_flow_parses_and_polls() {
        let start = MicrosoftFlow::start_request("0000-client");
        assert!(start.body.contains("scope=XboxLive.signin+offline_access"));

        let flow = MicrosoftFlow::from_device_response(
            "0000-client",
            &response(
                200,
                r#"{"device_code":"dc","user_code":"AB12","verification_uri":"https://microsoft.com/link","interval":5,"expires_in":900}"#,
            ),
        )
        .unwrap();
        assert_eq!(flow.device.user_code, "AB12");
        assert!(flow.poll_request().body.contains("device_code=dc"));
    }

    #[test]
    fn pending_is_not_an_error() {
        let outcome = MicrosoftFlow::poll_outcome(&response(
            400,
            r#"{"error":"authorization_pending"}"#,
        ));
        assert_eq!(outcome, Ok(PollOutcome::Pending));

        let expired =
            MicrosoftFlow::poll_outcome(&response(400, r#"{"error":"expired_token"}"#));
        assert_eq!(expired, Err(AuthenticationError::CodeExpired));
    }

    #[test]
    fn xsts_then_minecraft_token_shape() {
        let xbl = parse_xbl(&response(
            200,
            r#"{"Token":"xbl-token","DisplayClaims":{"xui":[{"uhs":"user-hash"}]}}"#,
        ))
        .unwrap();
        assert_eq!(xbl, ("xbl-token".to_owned(), "user-hash".to_owned()));

        let request = MicrosoftFlow::minecraft_login_request("user-hash", "xsts-token");
        assert!(request.body.contains("XBL3.0 x=user-hash;xsts-token"));
    }

    #[test]
    fn missing_entitlement_is_reported_clearly() {
        assert_eq!(
            parse_profile(&response(404, r#"{"error":"NOT_FOUND"}"#)),
            Err(AuthenticationError::NoMinecraftEntitlement)
        );
        assert!(has_entitlement(&response(
            200,
            r#"{"items":[{"name":"product_minecraft"}]}"#
        )));
    }

    #[test]
    fn tokens_are_never_in_debug_output() {
        let request = MicrosoftFlow::xbl_request("ms-secret");
        let shown = format!("{:?}", request.redacted());
        assert!(!shown.contains("ms-secret"));
    }
}
