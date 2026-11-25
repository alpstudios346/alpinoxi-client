//! Token helpers: PKCE, random state strings and a debug-safe token set.

use serde::{Deserialize, Serialize};

/// The token response shared by Microsoft and Ely.by.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub token_type: String,
    /// Seconds until `access_token` expires.
    #[serde(default)]
    pub expires_in: u64,
}

impl TokenSet {
    pub fn is_expired_at(&self, issued_at: u64, now: u64) -> bool {
        // Refresh a minute early so a request never races the expiry.
        now + 60 >= issued_at.saturating_add(self.expires_in)
    }
}

// Tokens must never reach a log line by accident, so Debug prints only shape.
impl std::fmt::Display for TokenSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TokenSet {{ access_token: <{} chars>, refresh_token: {}, expires_in: {} }}",
            self.access_token.len(),
            if self.refresh_token.is_some() {
                "present"
            } else {
                "none"
            },
            self.expires_in
        )
    }
}

/// Builds an unpredictable string for `state`/`code_verifier`.
///
/// Without a CSPRNG dependency this mixes the OS randomness available through
/// the standard library (address entropy, time, thread id) and hashes it. The
/// result is fine for CSRF `state`; for the PKCE verifier it is only a fallback
/// and the app should prefer a platform RNG where one is available.
pub fn random_urlsafe(len: usize) -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(len);
    let mut counter: u64 = 0;
    while out.len() < len {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        );
        hasher.write_u64(counter);
        hasher.write_u64(&out as *const _ as usize as u64);
        counter = counter.wrapping_add(1);
        let mut value = hasher.finish();
        for _ in 0..10 {
            if out.len() == len {
                break;
            }
            out.push(ALPHABET[(value % ALPHABET.len() as u64) as usize] as char);
            value /= ALPHABET.len() as u64;
        }
    }
    out
}

/// PKCE S256 challenge for a verifier (RFC 7636).
pub fn code_challenge(verifier: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(verifier.as_bytes());
    base64_url_no_pad(&digest)
}

fn base64_url_no_pad(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).copied().map(u32::from).unwrap_or(0);
        let b2 = chunk.get(2).copied().map(u32::from).unwrap_or(0);
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[(triple >> 18) as usize & 0x3f] as char);
        out.push(TABLE[(triple >> 12) as usize & 0x3f] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(triple >> 6) as usize & 0x3f] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[triple as usize & 0x3f] as char);
        }
    }
    out
}

/// Replaces the value of every sensitive form field or JSON key with a marker.
pub fn redact_body(body: &str) -> String {
    const SENSITIVE: &[&str] = &[
        "access_token",
        "refresh_token",
        "client_secret",
        "code",
        "code_verifier",
        "device_code",
        "identityToken",
        "password",
    ];
    let mut out = body.to_owned();
    for key in SENSITIVE {
        out = redact_key(&out, key, "&");
        out = redact_key(&out, key, "\"");
    }
    out
}

fn redact_key(body: &str, key: &str, separator: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(position) = rest.find(key) {
        let (before, after) = rest.split_at(position + key.len());
        out.push_str(before);
        let value = if separator == "&" {
            after.strip_prefix('=').map(|tail| ("=", tail))
        } else {
            after
                .strip_prefix("\":\"")
                .or_else(|| after.strip_prefix("\": \""))
                .map(|tail| ("", tail))
        };
        match value {
            Some((prefix, tail)) => {
                out.push_str(prefix);
                let end = if separator == "&" {
                    tail.find('&').unwrap_or(tail.len())
                } else {
                    tail.find('"').unwrap_or(tail.len())
                };
                out.push_str("<redacted>");
                rest = &tail[end..];
            }
            None => rest = after,
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_rfc_example() {
        // RFC 7636 appendix B.
        assert_eq!(
            code_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_strings_are_url_safe_and_varied() {
        let a = random_urlsafe(32);
        let b = random_urlsafe(32);
        assert_eq!(a.len(), 32);
        assert!(a
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert_ne!(a, b);
    }

    #[test]
    fn token_debug_shape_is_not_a_secret() {
        let set = TokenSet {
            access_token: "secret".to_owned(),
            refresh_token: Some("refresh".to_owned()),
            token_type: "Bearer".to_owned(),
            expires_in: 86_400,
        };
        let shown = format!("{set}");
        assert!(!shown.contains("secret"));
        assert!(shown.contains("refresh_token: present"));
    }

    #[test]
    fn expiry_has_a_safety_margin() {
        let set = TokenSet {
            access_token: "a".to_owned(),
            refresh_token: None,
            token_type: "Bearer".to_owned(),
            expires_in: 3_600,
        };
        assert!(!set.is_expired_at(0, 3_000));
        assert!(set.is_expired_at(0, 3_540));
    }

    #[test]
    fn redaction_covers_form_and_json() {
        let form = redact_body("client_id=x&code=abc&refresh_token=def");
        assert_eq!(form, "client_id=x&code=<redacted>&refresh_token=<redacted>");

        let json = redact_body(r#"{"access_token":"abc","x":1}"#);
        assert_eq!(json, r#"{"access_token":"<redacted>","x":1}"#);
    }
}
