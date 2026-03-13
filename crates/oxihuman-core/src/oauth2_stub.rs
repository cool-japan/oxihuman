// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OAuth2 token stub — PKCE flow and token exchange helpers.

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

/// Generates a stub PKCE challenge from a verifier string.
pub fn generate_pkce_challenge(verifier: &str) -> PkceChallenge {
    let challenge = format!("sha256:{}", &verifier[..verifier.len().min(32)]);
    PkceChallenge {
        code_verifier: verifier.to_owned(),
        code_challenge: challenge,
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
    Ok(OAuth2Token {
        access_token: format!("at_{}", code),
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

    #[test]
    fn test_pkce_challenge_contains_verifier_prefix() {
        let ch = generate_pkce_challenge("my-verifier-string");
        assert!(ch.code_challenge.contains("sha256:"));
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
        let cfg = OAuth2Config {
            client_id: "cid".into(),
            redirect_uri: "".into(),
            auth_endpoint: "".into(),
            token_endpoint: "".into(),
        };
        let tok = exchange_code_for_token(&cfg, "code123", "verifier").expect("should succeed");
        assert!(tok.access_token.contains("code123"));
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
}
