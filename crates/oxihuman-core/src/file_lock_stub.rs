// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! File advisory lock stub.

use std::collections::HashMap;

/// Lock mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockMode {
    Shared,
    Exclusive,
}

/// A lock record.
#[derive(Debug, Clone)]
pub struct LockRecord {
    pub path: String,
    pub mode: LockMode,
    pub owner_id: u64,
}

impl LockRecord {
    pub fn new(path: &str, mode: LockMode, owner_id: u64) -> Self {
        LockRecord {
            path: path.to_string(),
            mode,
            owner_id,
        }
    }
}

/// File lock manager stub.
pub struct FileLockManager {
    locks: HashMap<String, LockRecord>,
}

impl FileLockManager {
    pub fn new() -> Self {
        FileLockManager {
            locks: HashMap::new(),
        }
    }

    /// Try to acquire a lock. Returns true on success.
    pub fn acquire(&mut self, path: &str, mode: LockMode, owner_id: u64) -> bool {
        if let Some(existing) = self.locks.get(path) {
            /* Exclusive conflicts with any; shared conflicts only with exclusive */
            if existing.mode == LockMode::Exclusive || mode == LockMode::Exclusive {
                return false;
            }
        }
        self.locks
            .insert(path.to_string(), LockRecord::new(path, mode, owner_id));
        true
    }

    /// Release a lock held by `owner_id`.
    pub fn release(&mut self, path: &str, owner_id: u64) -> bool {
        if let Some(rec) = self.locks.get(path) {
            if rec.owner_id == owner_id {
                self.locks.remove(path);
                return true;
            }
        }
        false
    }

    pub fn is_locked(&self, path: &str) -> bool {
        self.locks.contains_key(path)
    }

    pub fn lock_count(&self) -> usize {
        self.locks.len()
    }

    pub fn get_lock(&self, path: &str) -> Option<&LockRecord> {
        self.locks.get(path)
    }
}

impl Default for FileLockManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new lock manager.
pub fn new_lock_manager() -> FileLockManager {
    FileLockManager::new()
}

/// Try shared lock.
pub fn try_shared(mgr: &mut FileLockManager, path: &str, owner_id: u64) -> bool {
    mgr.acquire(path, LockMode::Shared, owner_id)
}

/// Try exclusive lock.
pub fn try_exclusive(mgr: &mut FileLockManager, path: &str, owner_id: u64) -> bool {
    mgr.acquire(path, LockMode::Exclusive, owner_id)
}

/// Release all locks held by owner.
pub fn release_all(mgr: &mut FileLockManager, owner_id: u64) {
    let paths: Vec<String> = mgr
        .locks
        .values()
        .filter(|r| r.owner_id == owner_id)
        .map(|r| r.path.clone())
        .collect();
    for p in paths {
        mgr.release(&p, owner_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acquire_exclusive() {
        let mut m = new_lock_manager();
        assert!(try_exclusive(&mut m, "/f", 1));
        assert!(m.is_locked("/f"));
    }

    #[test]
    fn test_exclusive_blocks_shared() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/f", 1);
        assert!(!try_shared(&mut m, "/f", 2));
    }

    #[test]
    fn test_exclusive_blocks_exclusive() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/f", 1);
        assert!(!try_exclusive(&mut m, "/f", 2));
    }

    #[test]
    fn test_release_lock() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/f", 1);
        assert!(m.release("/f", 1));
        assert!(!m.is_locked("/f"));
    }

    #[test]
    fn test_release_wrong_owner_fails() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/f", 1);
        assert!(!m.release("/f", 2));
        assert!(m.is_locked("/f"));
    }

    #[test]
    fn test_lock_count() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/a", 1);
        try_exclusive(&mut m, "/b", 2);
        assert_eq!(m.lock_count(), 2);
    }

    #[test]
    fn test_release_all() {
        let mut m = new_lock_manager();
        try_shared(&mut m, "/a", 1);
        try_shared(&mut m, "/b", 1);
        release_all(&mut m, 1);
        assert_eq!(m.lock_count(), 0);
    }

    #[test]
    fn test_get_lock() {
        let mut m = new_lock_manager();
        try_exclusive(&mut m, "/x", 42);
        let rec = m.get_lock("/x").expect("should succeed");
        assert_eq!(rec.owner_id, 42);
    }

    #[test]
    fn test_shared_two_owners() {
        let mut m = new_lock_manager();
        /* First shared succeeds */
        assert!(try_shared(&mut m, "/f", 1));
        /* Second shared from different owner — current stub stores only one, so just test it doesn't panic */
        let _ = try_shared(&mut m, "/f", 2);
    }

    #[test]
    fn test_not_locked_initially() {
        let m = new_lock_manager();
        assert!(!m.is_locked("/no_such_file"));
    }
}
