// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Session token generation and validation stub.

/// Represents a session with a token and expiry.
#[derive(Clone, Debug)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub expires_at: u64,
    pub created_at: u64,
}

/// Configuration for session management.
#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub token_length: usize,
    pub ttl_secs: u64,
    pub prefix: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            token_length: 32,
            ttl_secs: 3600,
            prefix: "sess_".into(),
        }
    }
}

/// A simple in-memory session store.
pub struct SessionStore {
    pub config: SessionConfig,
    sessions: Vec<Session>,
}

/// Generates a pseudo-random session token (stub — not cryptographic).
pub fn generate_token(config: &SessionConfig, seed: u64) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut result = config.prefix.clone();
    let mut s = seed.wrapping_add(0xdeadbeef);
    for _ in 0..config.token_length {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        result.push(chars[(s >> 33) as usize % chars.len()] as char);
    }
    result
}

/// Creates a new session for a user at the given timestamp.
pub fn create_session(store: &mut SessionStore, user_id: &str, now: u64) -> Session {
    let seed = now.wrapping_add(user_id.len() as u64);
    let token = generate_token(&store.config, seed);
    let sess = Session {
        token: token.clone(),
        user_id: user_id.to_owned(),
        created_at: now,
        expires_at: now + store.config.ttl_secs,
    };
    store.sessions.push(sess.clone());
    sess
}

/// Validates a session token at the given timestamp.
pub fn validate_session<'a>(store: &'a SessionStore, token: &str, now: u64) -> Option<&'a Session> {
    store
        .sessions
        .iter()
        .find(|s| s.token == token && s.expires_at > now)
}

/// Revokes a session token, removing it from the store.
pub fn revoke_session(store: &mut SessionStore, token: &str) -> bool {
    let before = store.sessions.len();
    store.sessions.retain(|s| s.token != token);
    store.sessions.len() < before
}

/// Removes all expired sessions from the store.
pub fn purge_expired(store: &mut SessionStore, now: u64) -> usize {
    let before = store.sessions.len();
    store.sessions.retain(|s| s.expires_at > now);
    before.saturating_sub(store.sessions.len())
}

impl SessionStore {
    /// Creates a new session store with the given config.
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Vec::new(),
        }
    }

    /// Returns the number of active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> SessionStore {
        SessionStore::new(SessionConfig::default())
    }

    #[test]
    fn test_generate_token_has_prefix() {
        let cfg = SessionConfig::default();
        let tok = generate_token(&cfg, 42);
        assert!(tok.starts_with("sess_"));
    }

    #[test]
    fn test_generate_token_length() {
        let cfg = SessionConfig::default();
        let tok = generate_token(&cfg, 1);
        /* token = prefix + token_length chars */
        assert_eq!(tok.len(), cfg.prefix.len() + cfg.token_length);
    }

    #[test]
    fn test_create_session_stored() {
        let mut store = make_store();
        let sess = create_session(&mut store, "alice", 1000);
        assert_eq!(store.active_count(), 1);
        assert_eq!(sess.user_id, "alice");
    }

    #[test]
    fn test_validate_session_success() {
        let mut store = make_store();
        let sess = create_session(&mut store, "bob", 1000);
        let found = validate_session(&store, &sess.token, 1500);
        assert!(found.is_some());
    }

    #[test]
    fn test_validate_session_expired() {
        let mut store = make_store();
        let sess = create_session(&mut store, "carol", 0);
        /* expire_at = ttl_secs = 3600; now = 5000 => expired */
        let found = validate_session(&store, &sess.token, 5000);
        assert!(found.is_none());
    }

    #[test]
    fn test_revoke_session_removes_it() {
        let mut store = make_store();
        let sess = create_session(&mut store, "dave", 1000);
        let removed = revoke_session(&mut store, &sess.token);
        assert!(removed);
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn test_revoke_nonexistent_returns_false() {
        let mut store = make_store();
        assert!(!revoke_session(&mut store, "nonexistent"));
    }

    #[test]
    fn test_purge_expired_removes_old_sessions() {
        let mut store = make_store();
        create_session(&mut store, "e1", 0); /* expires at 3600 */
        create_session(&mut store, "e2", 100); /* expires at 3700 */
        create_session(&mut store, "e3", 5000); /* expires at 8600 */
        let removed = purge_expired(&mut store, 4000);
        assert_eq!(removed, 2);
        assert_eq!(store.active_count(), 1);
    }

    #[test]
    fn test_different_seeds_produce_different_tokens() {
        let cfg = SessionConfig::default();
        let t1 = generate_token(&cfg, 1);
        let t2 = generate_token(&cfg, 2);
        assert_ne!(t1, t2);
    }
}
