// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Key-value session store with per-session namespacing and TTL support.

use std::collections::HashMap;

/// A single session containing key-value pairs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
    pub created_at: u64,
    pub expires_at: u64,
}

#[allow(dead_code)]
impl Session {
    pub fn new(id: &str, now: u64, ttl: u64) -> Self {
        Self {
            id: id.to_string(),
            data: HashMap::new(),
            created_at: now,
            expires_at: now + ttl,
        }
    }

    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.expires_at
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn entry_count(&self) -> usize {
        self.data.len()
    }
}

/// The session store.
#[allow(dead_code)]
pub struct SessionStore {
    sessions: HashMap<String, Session>,
    now: u64,
    default_ttl: u64,
    evict_count: u64,
}

#[allow(dead_code)]
impl SessionStore {
    pub fn new(default_ttl: u64) -> Self {
        Self {
            sessions: HashMap::new(),
            now: 0,
            default_ttl,
            evict_count: 0,
        }
    }

    pub fn advance_time(&mut self, dt: u64) {
        self.now += dt;
    }

    pub fn create_session(&mut self, id: &str) -> bool {
        if self.sessions.contains_key(id) {
            return false;
        }
        let s = Session::new(id, self.now, self.default_ttl);
        self.sessions.insert(id.to_string(), s);
        true
    }

    pub fn destroy_session(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn set(&mut self, session_id: &str, key: &str, value: &str) -> bool {
        if let Some(s) = self.sessions.get_mut(session_id) {
            if !s.is_expired(self.now) {
                s.set(key, value);
                return true;
            }
        }
        false
    }

    pub fn get(&self, session_id: &str, key: &str) -> Option<&str> {
        self.sessions
            .get(session_id)
            .filter(|s| !s.is_expired(self.now))
            .and_then(|s| s.get(key))
    }

    pub fn evict_expired(&mut self) -> usize {
        let before = self.sessions.len();
        let now = self.now;
        self.sessions.retain(|_, s| !s.is_expired(now));
        let evicted = before - self.sessions.len();
        self.evict_count += evicted as u64;
        evicted
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn evict_count(&self) -> u64 {
        self.evict_count
    }

    pub fn has_session(&self, id: &str) -> bool {
        self.sessions.contains_key(id)
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(3600)
    }
}

pub fn new_session_store(default_ttl: u64) -> SessionStore {
    SessionStore::new(default_ttl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_set() {
        let mut store = new_session_store(1000);
        store.create_session("sess1");
        assert!(store.set("sess1", "user", "alice"));
        assert_eq!(store.get("sess1", "user"), Some("alice"));
    }

    #[test]
    fn expired_session_invisible() {
        let mut store = new_session_store(10);
        store.create_session("s");
        store.set("s", "k", "v");
        store.advance_time(20);
        assert!(store.get("s", "k").is_none());
    }

    #[test]
    fn evict_removes_expired() {
        let mut store = new_session_store(5);
        store.create_session("a");
        store.advance_time(10);
        store.evict_expired();
        assert!(!store.has_session("a"));
    }

    #[test]
    fn duplicate_session_rejected() {
        let mut store = new_session_store(100);
        assert!(store.create_session("x"));
        assert!(!store.create_session("x"));
    }

    #[test]
    fn destroy_session() {
        let mut store = new_session_store(100);
        store.create_session("s");
        assert!(store.destroy_session("s"));
        assert!(!store.has_session("s"));
    }

    #[test]
    fn session_count() {
        let mut store = new_session_store(100);
        store.create_session("a");
        store.create_session("b");
        assert_eq!(store.session_count(), 2);
    }

    #[test]
    fn evict_count_tracked() {
        let mut store = new_session_store(1);
        store.create_session("x");
        store.advance_time(5);
        store.evict_expired();
        assert_eq!(store.evict_count(), 1);
    }

    #[test]
    fn clear_all() {
        let mut store = new_session_store(100);
        store.create_session("a");
        store.clear();
        assert_eq!(store.session_count(), 0);
    }

    #[test]
    fn get_missing_key() {
        let mut store = new_session_store(100);
        store.create_session("s");
        assert!(store.get("s", "nope").is_none());
    }

    #[test]
    fn advance_time_tracked() {
        let mut store = new_session_store(100);
        store.advance_time(50);
        assert_eq!(store.now(), 50);
    }
}
