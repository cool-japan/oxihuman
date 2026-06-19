// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OAuth2 token management — PKCE flow (RFC 7636), authorization code exchange
//! (RFC 6749 §4.1.3), and token refresh (RFC 6749 §6).
//!
//! HTTP is injected via [`OAuth2Transport`] so this crate remains dependency-free
//! from any HTTP client library.

use sha2::{Digest, Sha256};

// ─── Transport abstraction ────────────────────────────────────────────────────

/// Caller-supplied HTTP transport. Implementors send a POST request with an
/// `application/x-www-form-urlencoded` body and return the full response body.
pub trait OAuth2Transport {
    /// Send an HTTP POST to `url` with the given `body` (already url-encoded).
    /// Returns the raw response body on success, or an error string on failure.
    fn post(&self, url: &str, body: &str) -> Result<String, String>;
}

/// Mock transport for tests — returns a canned response regardless of the request.
pub struct MockTransport {
    pub response: String,
}

impl OAuth2Transport for MockTransport {
    fn post(&self, _url: &str, _body: &str) -> Result<String, String> {
        Ok(self.response.clone())
    }
}

// ─── Core data types ──────────────────────────────────────────────────────────

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

/// An OAuth2 client that manages the PKCE flow.
pub struct OAuth2Client {
    pub config: OAuth2Config,
    pending_verifier: Option<String>,
}

// ─── Low-level encoding helpers ───────────────────────────────────────────────

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

/// Percent-encodes a string for use as a URI query parameter value.
///
/// Only unreserved characters as defined in RFC 3986 §2.3
/// (`A-Z a-z 0-9 - _ . ~`) are left unencoded; every other byte is
/// replaced with its `%XX` uppercase hexadecimal form.
pub fn url_encode(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(s.len() * 3);
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(HEX[(byte >> 4) as usize] as char);
                out.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    out
}

// ─── JSON response parsing ────────────────────────────────────────────────────

/// Parses an RFC 6749 §5.1 token response JSON body into an [`OAuth2Token`].
///
/// On error response (§5.2), returns `Err(error_description)` if present,
/// otherwise `Err(error)`.  On missing `access_token`, returns an error.
fn parse_token_response(body: &str) -> Result<OAuth2Token, String> {
    let v: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("invalid JSON: {e}"))?;

    // RFC 6749 §5.2 — error response
    if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
        let description = v
            .get("error_description")
            .and_then(|d| d.as_str())
            .unwrap_or(err);
        return Err(description.to_owned());
    }

    let access_token = v
        .get("access_token")
        .and_then(|t| t.as_str())
        .ok_or("missing access_token in response")?
        .to_owned();

    let token_type = v
        .get("token_type")
        .and_then(|t| t.as_str())
        .unwrap_or("Bearer")
        .to_owned();

    let expires_in_secs = v.get("expires_in").and_then(|e| e.as_u64()).unwrap_or(3600);

    let refresh_token = v
        .get("refresh_token")
        .and_then(|r| r.as_str())
        .map(|s| s.to_owned());

    let scope = v
        .get("scope")
        .and_then(|s| s.as_str())
        .map(|s| s.to_owned());

    Ok(OAuth2Token {
        access_token,
        token_type,
        expires_in_secs,
        refresh_token,
        scope,
    })
}

// ─── PKCE ────────────────────────────────────────────────────────────────────

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

// ─── Authorization URL ────────────────────────────────────────────────────────

/// Builds the RFC 6749 / RFC 7636 authorization URL with percent-encoded
/// parameters and `code_challenge_method=S256`.
///
/// Parameters that may contain special characters (`client_id`, `redirect_uri`,
/// `code_challenge`, `state`) are passed through [`url_encode`] so the
/// resulting URL is always well-formed.
pub fn build_authorization_url(
    cfg: &OAuth2Config,
    challenge: &PkceChallenge,
    state: &str,
) -> String {
    format!(
        "{}?response_type=code&code_challenge_method=S256\
         &client_id={}\
         &redirect_uri={}\
         &code_challenge={}\
         &state={}",
        cfg.auth_endpoint,
        url_encode(&cfg.client_id),
        url_encode(&cfg.redirect_uri),
        url_encode(&challenge.code_challenge),
        url_encode(state),
    )
}

// ─── Token exchange (RFC 6749 §4.1.3) ────────────────────────────────────────

/// Exchanges an authorization code for an [`OAuth2Token`] via the real
/// RFC 6749 §4.1.3 token endpoint.
///
/// The `application/x-www-form-urlencoded` request body contains:
/// `grant_type`, `code`, `redirect_uri`, `client_id`, and `code_verifier`.
/// Each value is percent-encoded.  The response is parsed per RFC 6749 §5.1.
pub fn exchange_code_for_token_with_transport(
    cfg: &OAuth2Config,
    code: &str,
    verifier: &str,
    transport: &dyn OAuth2Transport,
) -> Result<OAuth2Token, String> {
    if code.is_empty() {
        return Err("empty authorization code".into());
    }
    if verifier.is_empty() {
        return Err("empty code verifier".into());
    }

    let body = format!(
        "grant_type=authorization_code\
         &code={}\
         &redirect_uri={}\
         &client_id={}\
         &code_verifier={}",
        url_encode(code),
        url_encode(&cfg.redirect_uri),
        url_encode(&cfg.client_id),
        url_encode(verifier),
    );

    let response = transport.post(&cfg.token_endpoint, &body)?;
    parse_token_response(&response)
}

/// Compatibility shim: validates inputs only; callers that need a real token
/// must migrate to [`exchange_code_for_token_with_transport`].
///
/// Returns `Err` on empty inputs, otherwise returns `Err` explaining that a
/// transport is required.  This preserves the old call-site signature for
/// tests that only exercise the validation path.
pub fn exchange_code_for_token(
    _cfg: &OAuth2Config,
    code: &str,
    verifier: &str,
) -> Result<OAuth2Token, String> {
    if code.is_empty() {
        return Err("empty authorization code".into());
    }
    if verifier.is_empty() {
        return Err("empty code verifier".into());
    }
    Err("a transport is required; use exchange_code_for_token_with_transport".into())
}

// ─── Token refresh (RFC 6749 §6) ─────────────────────────────────────────────

/// Refreshes an OAuth2 token via the real RFC 6749 §6 token endpoint.
///
/// Builds `grant_type=refresh_token&refresh_token=TOKEN&client_id=ID` and
/// parses the JSON response per §5.1.
pub fn refresh_token_with_transport(
    cfg: &OAuth2Config,
    refresh: &str,
    transport: &dyn OAuth2Transport,
) -> Result<OAuth2Token, String> {
    if refresh.is_empty() {
        return Err("empty refresh token".into());
    }

    let body = format!(
        "grant_type=refresh_token\
         &refresh_token={}\
         &client_id={}",
        url_encode(refresh),
        url_encode(&cfg.client_id),
    );

    let response = transport.post(&cfg.token_endpoint, &body)?;
    parse_token_response(&response)
}

/// Compatibility shim: validates the refresh token is non-empty; callers that
/// need a real token must migrate to [`refresh_token_with_transport`].
pub fn refresh_token(cfg: &OAuth2Config, refresh: &str) -> Result<OAuth2Token, String> {
    if refresh.is_empty() {
        return Err("empty refresh token".into());
    }
    Err(format!(
        "a transport is required for client '{}'; use refresh_token_with_transport",
        cfg.client_id
    ))
}

// ─── OAuth2Client ─────────────────────────────────────────────────────────────

impl OAuth2Client {
    /// Creates a new OAuth2 client from config.
    pub fn new(config: OAuth2Config) -> Self {
        Self {
            config,
            pending_verifier: None,
        }
    }

    /// Starts the PKCE flow: derives the code challenge from `verifier`,
    /// stores the verifier for later, and returns the full authorization URL.
    pub fn start_pkce_flow(&mut self, verifier: &str, state: &str) -> String {
        let challenge = generate_pkce_challenge(verifier);
        self.pending_verifier = Some(verifier.to_owned());
        build_authorization_url(&self.config, &challenge, state)
    }

    /// Completes the PKCE flow by exchanging `code` for a token using `transport`.
    ///
    /// The stored verifier (set by [`Self::start_pkce_flow`]) is consumed; a
    /// subsequent call without a new `start_pkce_flow` will return an error.
    pub fn complete_flow<T: OAuth2Transport>(
        &mut self,
        code: &str,
        transport: &T,
    ) -> Result<OAuth2Token, String> {
        let verifier = self.pending_verifier.take().ok_or("no pending flow")?;
        exchange_code_for_token_with_transport(&self.config, code, &verifier, transport)
    }

    /// Refreshes `token` via the token endpoint, returning a new [`OAuth2Token`].
    pub fn refresh<T: OAuth2Transport>(
        &self,
        token: &OAuth2Token,
        transport: &T,
    ) -> Result<OAuth2Token, String> {
        let refresh = token
            .refresh_token
            .as_deref()
            .ok_or("token has no refresh_token")?;
        refresh_token_with_transport(&self.config, refresh, transport)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn valid_token_json(access: &str, refresh: &str) -> String {
        format!(
            r#"{{"access_token":"{access}","token_type":"Bearer","expires_in":3600,"refresh_token":"{refresh}","scope":"openid profile"}}"#
        )
    }

    fn cfg() -> OAuth2Config {
        OAuth2Config {
            client_id: "test-client".into(),
            redirect_uri: "http://localhost:8080/callback".into(),
            auth_endpoint: "https://auth.example.com/authorize".into(),
            token_endpoint: "https://auth.example.com/token".into(),
        }
    }

    // ── Preserved: PKCE ──────────────────────────────────────────────────────

    #[test]
    fn test_pkce_challenge_is_base64url() {
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

    // ── Preserved: base64url_no_pad ──────────────────────────────────────────

    #[test]
    fn test_base64url_no_pad_basic() {
        assert_eq!(base64url_no_pad(&[0xffu8]), "_w");
        assert_eq!(base64url_no_pad(&[]), "");
        let hash = Sha256::digest(b"abc");
        let encoded = base64url_no_pad(&hash);
        assert_eq!(encoded.len(), 43);
        assert!(!encoded.contains('='));
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
    }

    // ── Preserved: validation-only stubs ────────────────────────────────────

    #[test]
    fn test_exchange_empty_code_returns_error() {
        assert!(exchange_code_for_token(&cfg(), "", "verifier").is_err());
    }

    #[test]
    fn test_refresh_empty_token_returns_error() {
        assert!(refresh_token(&cfg(), "").is_err());
    }

    // ── Preserved: build_authorization_url basic ─────────────────────────────

    #[test]
    fn test_build_authorization_url_contains_client_id() {
        let ch = generate_pkce_challenge("ver");
        let url = build_authorization_url(&cfg(), &ch, "state1");
        assert!(url.contains("test-client"));
    }

    // ── Preserved: OAuth2Client without-start error ──────────────────────────

    #[test]
    fn test_complete_flow_without_start_errors() {
        let mut client = OAuth2Client::new(cfg());
        let transport = MockTransport {
            response: valid_token_json("at_x", "rt_x"),
        };
        assert!(client.complete_flow("code", &transport).is_err());
    }

    // ── New: url_encode ───────────────────────────────────────────────────────

    #[test]
    fn test_url_encode_unreserved_unchanged() {
        let s = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(url_encode(s), s);
    }

    #[test]
    fn test_url_encode_space_and_special() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("a+b=c&d"), "a%2Bb%3Dc%26d");
        assert_eq!(
            url_encode("http://x.com/cb?q=1"),
            "http%3A%2F%2Fx.com%2Fcb%3Fq%3D1"
        );
    }

    #[test]
    fn test_url_encode_empty() {
        assert_eq!(url_encode(""), "");
    }

    // ── New: build_authorization_url encodes special chars ───────────────────

    #[test]
    fn test_build_authorization_url_encodes_params() {
        let cfg_special = OAuth2Config {
            client_id: "client id+special".into(),
            redirect_uri: "http://localhost:8080/call back?x=1".into(),
            auth_endpoint: "https://auth.example.com/authorize".into(),
            token_endpoint: "https://auth.example.com/token".into(),
        };
        let ch = generate_pkce_challenge("ver");
        let url = build_authorization_url(&cfg_special, &ch, "state with spaces");

        // client_id space → %20, + → %2B
        assert!(
            url.contains("client%20id%2Bspecial"),
            "client_id not encoded: {url}"
        );
        // redirect_uri space → %20, : → %3A, / → %2F, ? → %3F, = → %3D
        assert!(
            url.contains("http%3A%2F%2Flocalhost%3A8080%2Fcall%20back%3Fx%3D1"),
            "redirect_uri not encoded: {url}"
        );
        // state space → %20
        assert!(
            url.contains("state%20with%20spaces"),
            "state not encoded: {url}"
        );
        // Must include response_type=code and S256
        assert!(
            url.contains("response_type=code"),
            "missing response_type: {url}"
        );
        assert!(
            url.contains("code_challenge_method=S256"),
            "missing S256: {url}"
        );
    }

    #[test]
    fn test_build_authorization_url_includes_required_params() {
        let ch = generate_pkce_challenge("verifier-xyz");
        let url = build_authorization_url(&cfg(), &ch, "my-state");
        assert!(url.starts_with("https://auth.example.com/authorize?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("state=my-state"));
        assert!(url.contains("redirect_uri="));
    }

    // ── New: exchange_code_for_token_with_transport ──────────────────────────

    #[test]
    fn test_exchange_code_parses_valid_response() {
        let transport = MockTransport {
            response: valid_token_json("access_tok_abc", "refresh_tok_xyz"),
        };
        let tok =
            exchange_code_for_token_with_transport(&cfg(), "auth-code-1", "verifier-1", &transport)
                .expect("should parse valid response");

        assert_eq!(tok.access_token, "access_tok_abc");
        assert_eq!(tok.token_type, "Bearer");
        assert_eq!(tok.expires_in_secs, 3600);
        assert_eq!(tok.refresh_token.as_deref(), Some("refresh_tok_xyz"));
        assert_eq!(tok.scope.as_deref(), Some("openid profile"));
    }

    #[test]
    fn test_exchange_code_handles_error_response() {
        let transport = MockTransport {
            response: r#"{"error":"invalid_grant","error_description":"Code expired"}"#.into(),
        };
        let result =
            exchange_code_for_token_with_transport(&cfg(), "bad-code", "verifier", &transport);
        assert!(result.is_err(), "expected error");
        assert!(
            result.unwrap_err().contains("Code expired"),
            "error should mention Code expired"
        );
    }

    #[test]
    fn test_exchange_code_error_without_description_uses_error_field() {
        let transport = MockTransport {
            response: r#"{"error":"unauthorized_client"}"#.into(),
        };
        let result = exchange_code_for_token_with_transport(&cfg(), "code", "verifier", &transport);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unauthorized_client"));
    }

    #[test]
    fn test_exchange_code_empty_code_returns_error() {
        let transport = MockTransport {
            response: "{}".into(),
        };
        let result = exchange_code_for_token_with_transport(&cfg(), "", "verifier", &transport);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty authorization code"));
    }

    #[test]
    fn test_exchange_code_empty_verifier_returns_error() {
        let transport = MockTransport {
            response: "{}".into(),
        };
        let result = exchange_code_for_token_with_transport(&cfg(), "code", "", &transport);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty code verifier"));
    }

    // ── New: refresh_token_with_transport ────────────────────────────────────

    #[test]
    fn test_refresh_token_parses_response() {
        let transport = MockTransport {
            response: valid_token_json("new_access_token", "new_refresh_token"),
        };
        let tok = refresh_token_with_transport(&cfg(), "old-refresh-token", &transport)
            .expect("should parse valid response");

        assert_eq!(tok.access_token, "new_access_token");
        assert_eq!(tok.token_type, "Bearer");
        assert_eq!(tok.expires_in_secs, 3600);
        assert_eq!(tok.refresh_token.as_deref(), Some("new_refresh_token"));
    }

    #[test]
    fn test_refresh_token_handles_error_response() {
        let transport = MockTransport {
            response: r#"{"error":"invalid_grant","error_description":"Refresh token expired"}"#
                .into(),
        };
        let result = refresh_token_with_transport(&cfg(), "stale-rt", &transport);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Refresh token expired"));
    }

    #[test]
    fn test_refresh_token_empty_refresh_returns_error() {
        let transport = MockTransport {
            response: "{}".into(),
        };
        let result = refresh_token_with_transport(&cfg(), "", &transport);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty refresh token"));
    }

    // ── New: OAuth2Client full flow ──────────────────────────────────────────

    #[test]
    fn test_pkce_flow_roundtrip() {
        let transport = MockTransport {
            response: valid_token_json("full_flow_token", "full_flow_rt"),
        };
        let mut client = OAuth2Client::new(cfg());

        let url = client.start_pkce_flow("secure-verifier-1234", "state-xyz");
        assert!(url.starts_with("https://auth.example.com/authorize?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("test-client"));
        // state encoded (no special chars here, so literal match)
        assert!(url.contains("state-xyz"));

        let tok = client
            .complete_flow("returned-auth-code", &transport)
            .expect("complete_flow should succeed");
        assert_eq!(tok.access_token, "full_flow_token");
        assert_eq!(tok.refresh_token.as_deref(), Some("full_flow_rt"));
    }

    #[test]
    fn test_client_refresh() {
        let transport = MockTransport {
            response: valid_token_json("refreshed_at", "new_rt"),
        };
        let client = OAuth2Client::new(cfg());
        let initial_token = OAuth2Token {
            access_token: "old_at".into(),
            token_type: "Bearer".into(),
            expires_in_secs: 0,
            refresh_token: Some("old_rt".into()),
            scope: None,
        };
        let tok = client
            .refresh(&initial_token, &transport)
            .expect("refresh should succeed");
        assert_eq!(tok.access_token, "refreshed_at");
    }

    #[test]
    fn test_client_refresh_no_refresh_token_errors() {
        let transport = MockTransport {
            response: "{}".into(),
        };
        let client = OAuth2Client::new(cfg());
        let no_refresh_token = OAuth2Token {
            access_token: "at".into(),
            token_type: "Bearer".into(),
            expires_in_secs: 3600,
            refresh_token: None,
            scope: None,
        };
        assert!(client.refresh(&no_refresh_token, &transport).is_err());
    }

    #[test]
    fn test_client_start_pkce_flow_consumes_verifier_once() {
        let transport = MockTransport {
            response: valid_token_json("tok_a", "rt_a"),
        };
        let mut client = OAuth2Client::new(cfg());
        client.start_pkce_flow("ver-111", "st1");
        // First call consumes the verifier
        let _ = client
            .complete_flow("code1", &transport)
            .expect("first complete_flow should succeed");
        // Second call must fail — no pending verifier
        let transport2 = MockTransport {
            response: valid_token_json("tok_b", "rt_b"),
        };
        assert!(client.complete_flow("code2", &transport2).is_err());
    }

    #[test]
    fn test_parse_token_response_missing_access_token() {
        let result = parse_token_response(r#"{"token_type":"Bearer"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing access_token"));
    }

    #[test]
    fn test_parse_token_response_invalid_json() {
        let result = parse_token_response("not-json{{{");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid JSON"));
    }

    #[test]
    fn test_parse_token_response_minimal_valid() {
        // Only access_token required; other fields have defaults
        let result = parse_token_response(r#"{"access_token":"minimal_tok"}"#);
        assert!(result.is_ok());
        let tok = result.unwrap();
        assert_eq!(tok.access_token, "minimal_tok");
        assert_eq!(tok.token_type, "Bearer");
        assert_eq!(tok.expires_in_secs, 3600);
        assert!(tok.refresh_token.is_none());
        assert!(tok.scope.is_none());
    }
}
