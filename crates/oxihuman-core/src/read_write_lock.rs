#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// State of a read-write lock (non-thread-safe simulation).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RwLockState {
    readers: u32,
    writer: bool,
}

#[allow(dead_code)]
pub fn new_rw_lock_state() -> RwLockState {
    RwLockState {
        readers: 0,
        writer: false,
    }
}

#[allow(dead_code)]
pub fn try_read_lock(state: &mut RwLockState) -> bool {
    if state.writer {
        false
    } else {
        state.readers += 1;
        true
    }
}

#[allow(dead_code)]
pub fn try_write_lock(state: &mut RwLockState) -> bool {
    if state.writer || state.readers > 0 {
        false
    } else {
        state.writer = true;
        true
    }
}

#[allow(dead_code)]
pub fn read_unlock(state: &mut RwLockState) -> bool {
    if state.readers > 0 {
        state.readers -= 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn write_unlock(state: &mut RwLockState) -> bool {
    if state.writer {
        state.writer = false;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn reader_count(state: &RwLockState) -> u32 {
    state.readers
}

#[allow(dead_code)]
pub fn is_write_locked(state: &RwLockState) -> bool {
    state.writer
}

#[allow(dead_code)]
pub fn lock_state_to_json(state: &RwLockState) -> String {
    format!(
        "{{\"readers\":{},\"writer\":{}}}",
        state.readers, state.writer
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rw_lock_state() {
        let s = new_rw_lock_state();
        assert_eq!(reader_count(&s), 0);
        assert!(!is_write_locked(&s));
    }

    #[test]
    fn test_try_read_lock() {
        let mut s = new_rw_lock_state();
        assert!(try_read_lock(&mut s));
        assert_eq!(reader_count(&s), 1);
    }

    #[test]
    fn test_try_write_lock() {
        let mut s = new_rw_lock_state();
        assert!(try_write_lock(&mut s));
        assert!(is_write_locked(&s));
    }

    #[test]
    fn test_read_blocks_write() {
        let mut s = new_rw_lock_state();
        try_read_lock(&mut s);
        assert!(!try_write_lock(&mut s));
    }

    #[test]
    fn test_write_blocks_read() {
        let mut s = new_rw_lock_state();
        try_write_lock(&mut s);
        assert!(!try_read_lock(&mut s));
    }

    #[test]
    fn test_read_unlock() {
        let mut s = new_rw_lock_state();
        try_read_lock(&mut s);
        assert!(read_unlock(&mut s));
        assert_eq!(reader_count(&s), 0);
    }

    #[test]
    fn test_write_unlock() {
        let mut s = new_rw_lock_state();
        try_write_lock(&mut s);
        assert!(write_unlock(&mut s));
        assert!(!is_write_locked(&s));
    }

    #[test]
    fn test_lock_state_to_json() {
        let s = new_rw_lock_state();
        let json = lock_state_to_json(&s);
        assert!(json.contains("\"readers\":0"));
    }

    #[test]
    fn test_multiple_readers() {
        let mut s = new_rw_lock_state();
        try_read_lock(&mut s);
        try_read_lock(&mut s);
        assert_eq!(reader_count(&s), 2);
    }

    #[test]
    fn test_unlock_empty() {
        let mut s = new_rw_lock_state();
        assert!(!read_unlock(&mut s));
        assert!(!write_unlock(&mut s));
    }
}
