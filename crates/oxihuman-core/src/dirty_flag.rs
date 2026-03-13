// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dirty flag and change tracking utilities for incremental updates.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DirtyFlag {
    pub dirty: bool,
    pub dirty_count: u64,
    pub last_clean_ts: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DirtyMask {
    pub bits: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DirtyTracker {
    pub flags: Vec<(String, DirtyFlag)>,
}

#[allow(dead_code)]
pub fn new_dirty_flag() -> DirtyFlag {
    DirtyFlag {
        dirty: false,
        dirty_count: 0,
        last_clean_ts: 0,
    }
}

#[allow(dead_code)]
pub fn mark_dirty(flag: &mut DirtyFlag) {
    flag.dirty = true;
    flag.dirty_count += 1;
}

#[allow(dead_code)]
pub fn mark_clean(flag: &mut DirtyFlag, ts: u64) {
    flag.dirty = false;
    flag.last_clean_ts = ts;
}

#[allow(dead_code)]
pub fn is_dirty(flag: &DirtyFlag) -> bool {
    flag.dirty
}

#[allow(dead_code)]
pub fn dirty_count(flag: &DirtyFlag) -> u64 {
    flag.dirty_count
}

#[allow(dead_code)]
pub fn new_dirty_mask() -> DirtyMask {
    DirtyMask { bits: 0 }
}

#[allow(dead_code)]
pub fn mask_set_bit(mask: &mut DirtyMask, bit: u8) {
    if bit < 64 {
        mask.bits |= 1u64 << bit;
    }
}

#[allow(dead_code)]
pub fn mask_clear_bit(mask: &mut DirtyMask, bit: u8) {
    if bit < 64 {
        mask.bits &= !(1u64 << bit);
    }
}

#[allow(dead_code)]
pub fn mask_is_set(mask: &DirtyMask, bit: u8) -> bool {
    if bit < 64 {
        (mask.bits & (1u64 << bit)) != 0
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn mask_any_set(mask: &DirtyMask) -> bool {
    mask.bits != 0
}

#[allow(dead_code)]
pub fn new_dirty_tracker() -> DirtyTracker {
    DirtyTracker { flags: Vec::new() }
}

#[allow(dead_code)]
pub fn tracker_mark(tracker: &mut DirtyTracker, name: &str) {
    if let Some(entry) = tracker.flags.iter_mut().find(|(n, _)| n == name) {
        mark_dirty(&mut entry.1);
    } else {
        let mut flag = new_dirty_flag();
        mark_dirty(&mut flag);
        tracker.flags.push((name.to_string(), flag));
    }
}

#[allow(dead_code)]
pub fn tracker_clean(tracker: &mut DirtyTracker, name: &str, ts: u64) {
    if let Some(entry) = tracker.flags.iter_mut().find(|(n, _)| n == name) {
        mark_clean(&mut entry.1, ts);
    }
}

#[allow(dead_code)]
pub fn tracker_is_dirty(tracker: &DirtyTracker, name: &str) -> bool {
    tracker
        .flags
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, f)| f.dirty)
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn tracker_dirty_count(tracker: &DirtyTracker) -> usize {
    tracker.flags.iter().filter(|(_, f)| f.dirty).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dirty_flag() {
        let f = new_dirty_flag();
        assert!(!f.dirty);
        assert_eq!(f.dirty_count, 0);
        assert_eq!(f.last_clean_ts, 0);
    }

    #[test]
    fn test_mark_dirty_increments_count() {
        let mut f = new_dirty_flag();
        mark_dirty(&mut f);
        assert!(is_dirty(&f));
        assert_eq!(dirty_count(&f), 1);
        mark_dirty(&mut f);
        assert_eq!(dirty_count(&f), 2);
    }

    #[test]
    fn test_mark_clean() {
        let mut f = new_dirty_flag();
        mark_dirty(&mut f);
        mark_clean(&mut f, 42);
        assert!(!is_dirty(&f));
        assert_eq!(f.last_clean_ts, 42);
    }

    #[test]
    fn test_dirty_mask_set_clear() {
        let mut m = new_dirty_mask();
        mask_set_bit(&mut m, 5);
        assert!(mask_is_set(&m, 5));
        assert!(mask_any_set(&m));
        mask_clear_bit(&mut m, 5);
        assert!(!mask_is_set(&m, 5));
        assert!(!mask_any_set(&m));
    }

    #[test]
    fn test_dirty_mask_oob() {
        let mut m = new_dirty_mask();
        mask_set_bit(&mut m, 64); // out of range, should be no-op
        assert!(!mask_any_set(&m));
        assert!(!mask_is_set(&m, 64));
    }

    #[test]
    fn test_tracker_mark_and_clean() {
        let mut t = new_dirty_tracker();
        tracker_mark(&mut t, "mesh");
        assert!(tracker_is_dirty(&t, "mesh"));
        assert_eq!(tracker_dirty_count(&t), 1);
        tracker_clean(&mut t, "mesh", 100);
        assert!(!tracker_is_dirty(&t, "mesh"));
        assert_eq!(tracker_dirty_count(&t), 0);
    }

    #[test]
    fn test_tracker_unknown_name() {
        let t = new_dirty_tracker();
        assert!(!tracker_is_dirty(&t, "unknown"));
    }

    #[test]
    fn test_tracker_multiple_flags() {
        let mut t = new_dirty_tracker();
        tracker_mark(&mut t, "a");
        tracker_mark(&mut t, "b");
        tracker_mark(&mut t, "a");
        assert_eq!(tracker_dirty_count(&t), 2);
        tracker_clean(&mut t, "a", 1);
        assert_eq!(tracker_dirty_count(&t), 1);
    }

    #[test]
    fn test_mask_multiple_bits() {
        let mut m = new_dirty_mask();
        for bit in [0u8, 7, 15, 31, 63] {
            mask_set_bit(&mut m, bit);
        }
        for bit in [0u8, 7, 15, 31, 63] {
            assert!(mask_is_set(&m, bit));
        }
        assert!(!mask_is_set(&m, 1));
    }
}
