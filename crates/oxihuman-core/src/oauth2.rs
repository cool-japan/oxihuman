// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OAuth2 token stub — PKCE flow and token exchange helpers.

use sha2::{Digest, Sha256};

/// PKCE code verifier and challenge pair.
#[derive(Clone, Debug)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// An OAuth2 access token with optional refresh token.
#[derive(Clone, Debug)]
pub struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in_secs: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Configuration for an OAuth2 client.
#[derive(Clone, Debug)]
pub struct OAuth2Config {
    pub client_id: String,
    pub redirect_uri: String,
    pub auth_endpoint: String,
    pub token_endpoint: String,
}

/// An OAuth2 stub client that manages the PKCE flow.
pub struct OAuth2Client {
    pub config: OAuth2Config,
    pending_verifier: Option<String>,
}

/// Encodes bytes as base64url without padding (RFC 4648 §5 / RFC 7636 §4.2).
fn base64url_no_pad(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity((bytes.len() * 4).div_ceil(3));
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let v = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(v >> 18) as usize & 63] as char);
        out.push(ALPHABET[(v >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(v >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[v as usize & 63] as char);
        }
    }
    out
}

/// Generates a real RFC 7636 PKCE challenge from a verifier string.
/// The code_challenge is SHA-256(verifier) encoded as base64url without padding.
pub fn generate_pkce_challenge(verifier: &str) -> PkceChallenge {
    let hash = Sha256::digest(verifier.as_bytes());
    let code_challenge = base64url_no_pad(&hash);
    PkceChallenge {
        code_verifier: verifier.to_owned(),
        code_challenge,
    }
}

/// Builds the authorization URL with PKCE parameters.
pub fn build_authorization_url(
    cfg: &OAuth2Config,
    challenge: &PkceChallenge,
    state: &str,
) -> String {
    format!(
        "{}?client_id={}&redirect_uri={}&code_challenge={}&state={}",
        cfg.auth_endpoint, cfg.client_id, cfg.redirect_uri, challenge.code_challenge, state
    )
}

/// Stub: exchanges an authorization code for an OAuth2 token.
/// The access token is the base64url-encoded SHA-256 of "client_id|code".
pub fn exchange_code_for_token(
    cfg: &OAuth2Config,
    code: &str,
    verifier: &str,
) -> Result<OAuth2Token, String> {
    if code.is_empty() {
        return Err("empty authorization code".into());
    }
    if verifier.is_empty() {
        return Err("empty code verifier".into());
    }
    let raw = format!("{}|{}", cfg.client_id, code);
    let token_bytes = Sha256::digest(raw.as_bytes());
    let access_token = base64url_no_pad(&token_bytes);
    Ok(OAuth2Token {
        access_token,
        token_type: "Bearer".into(),
        expires_in_secs: 3600,
        refresh_token: Some(format!("rt_{}", cfg.client_id)),
        scope: Some("openid profile".into()),
    })
}

/// Stub: refreshes an OAuth2 token using a refresh token.
pub fn refresh_token(cfg: &OAuth2Config, refresh: &str) -> Result<OAuth2Token, String> {
    if refresh.is_empty() {
        return Err("empty refresh token".into());
    }
    Ok(OAuth2Token {
        access_token: format!("at_refreshed_{}", cfg.client_id),
        token_type: "Bearer".into(),
        expires_in_secs: 3600,
        refresh_token: Some(refresh.to_owned()),
        scope: None,
    })
}

impl OAuth2Client {
    /// Creates a new OAuth2 client from config.
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            pending_verifier: None,
        }
    }

    /// Starts the PKCE flow, returning the authorization URL.
    pub fn start_pkce_flow(&mut self, verifier: &str, state: &str) -> String {
        let challenge = generate_pkce_challenge(verifier);
        self.pending_verifier = Some(verifier.to_owned());
        build_authorization_url(&self.config, &challenge, state)
    }

    /// Completes the PKCE flow by exchanging the auth code.
    pub fn complete_flow(&mut self, code: &str) -> Result<OAuth2Token, String> {
        let verifier = self.pending_verifier.take().ok_or("no pending flow")?;
        exchange_code_for_token(&self.config, code, &verifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- updated pre-existing tests ---

    #[test]
    fn test_pkce_challenge_is_base64url() {
        // Previously checked for "sha256:" prefix; now checks the real RFC 7636 contract:
        // 43-char base64url (SHA-256 = 32 bytes), no padding, URL-safe charset.
        let ch = generate_pkce_challenge("my-verifier-string");
        assert_eq!(ch.code_challenge.len(), 43);
        assert!(!ch.code_challenge.contains('='));
        assert!(!ch.code_challenge.contains('+'));
        assert!(!ch.code_challenge.contains('/'));
    }

    #[test]
    fn test_pkce_verifier_preserved() {
        let verifier = "verifier-abc";
        let ch = generate_pkce_challenge(verifier);
        assert_eq!(ch.code_verifier, verifier);
    }

    #[test]
    fn test_build_authorization_url_contains_client_id() {
        let cfg = OAuth2Config {
            client_id: "cid".into(),
            redirect_uri: "http://localhost".into(),
            auth_endpoint: "https://auth.example.com/auth".into(),
            token_endpoint: "https://auth.example.com/token".into(),
        };
        let ch = generate_pkce_challenge("ver");
        let url = build_authorization_url(&cfg, &ch, "state1");
        assert!(url.contains("cid"));
    }

    #[test]
    fn test_exchange_empty_code_returns_error() {
        let cfg = OAuth2Config {
            client_id: "cid".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        assert!(exchange_code_for_token(&cfg, "", "verifier").is_err());
    }

    #[test]
    fn test_exchange_valid_code_returns_token() {
        // Previously checked access_token.contains("code123"); now it's a base64url hash.
        // Check non-empty, URL-safe, and deterministic.
        let cfg = OAuth2Config {
            client_id: "cid".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        let tok1 = exchange_code_for_token(&cfg, "code123", "verifier").expect("should succeed");
        let tok2 = exchange_code_for_token(&cfg, "code123", "verifier").expect("should succeed");
        assert!(!tok1.access_token.is_empty());
        assert_eq!(tok1.access_token, tok2.access_token);
        assert!(!tok1.access_token.contains('+'));
        assert!(!tok1.access_token.contains('/'));
        assert!(!tok1.access_token.contains('='));
    }

    #[test]
    fn test_refresh_empty_token_returns_error() {
        let cfg = OAuth2Config {
            client_id: "cid".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        assert!(refresh_token(&cfg, "").is_err());
    }

    #[test]
    fn test_refresh_valid_token_returns_new_token() {
        let cfg = OAuth2Config {
            client_id: "myclient".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        let tok = refresh_token(&cfg, "rt_old").expect("should succeed");
        assert!(tok.access_token.contains("myclient"));
    }

    #[test]
    fn test_client_start_and_complete_flow() {
        let cfg = OAuth2Config {
            client_id: "c1".into(),
            redirect_uri: "http://localhost".into(),
            auth_endpoint: "https://auth.test/auth".into(),
            token_endpoint: "https://auth.test/token".into(),
        };
        let mut client = OAuth2Client::new(cfg);
        let url = client.start_pkce_flow("verifier-xyz", "s1");
        assert!(url.contains("c1"));
        let tok = client.complete_flow("authcode").expect("should succeed");
        assert!(!tok.access_token.is_empty());
    }

    #[test]
    fn test_complete_flow_without_start_errors() {
        let cfg = OAuth2Config {
            client_id: "c2".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        let mut client = OAuth2Client::new(cfg);
        assert!(client.complete_flow("code").is_err());
    }

    // --- new tests ---

    #[test]
    fn test_pkce_known_vector() {
        // RFC 7636 Appendix B test vector
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge_obj = generate_pkce_challenge(verifier);
        assert_eq!(
            challenge_obj.code_challenge,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn test_pkce_deterministic() {
        let a = generate_pkce_challenge("test-verifier-abc123");
        let b = generate_pkce_challenge("test-verifier-abc123");
        assert_eq!(a.code_challenge, b.code_challenge);
    }

    #[test]
    fn test_base64url_no_pad_basic() {
        // 0xFF → "_w" (two chars, no padding)
        assert_eq!(base64url_no_pad(&[0xffu8]), "_w");
        // Empty input → empty output
        assert_eq!(base64url_no_pad(&[]), "");
        // SHA-256 produces 32 bytes → ceil(32*4/3) = 43 base64url chars
        let hash = Sha256::digest(b"abc");
        let encoded = base64url_no_pad(&hash);
        assert_eq!(encoded.len(), 43);
        assert!(!encoded.contains('='));
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
    }

    #[test]
    fn test_token_exchange_deterministic() {
        let cfg = OAuth2Config {
            client_id: "client-x".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        let tok1 = exchange_code_for_token(&cfg, "auth-code-42", "v").expect("ok");
        let tok2 = exchange_code_for_token(&cfg, "auth-code-42", "v").expect("ok");
        assert_eq!(tok1.access_token, tok2.access_token);
        // Different code → different token
        let tok3 = exchange_code_for_token(&cfg, "auth-code-99", "v").expect("ok");
        assert_ne!(tok1.access_token, tok3.access_token);
    }
}
