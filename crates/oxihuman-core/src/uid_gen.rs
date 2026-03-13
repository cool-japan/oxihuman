// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UID generator: produces unique u64 identifiers with optional prefix encoding.

/// A UID generator with monotone counter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UidGen {
    counter: u64,
    namespace: u16,
    recycled: Vec<u64>,
}

/// Create a new `UidGen` with the given namespace (0–65535).
#[allow(dead_code)]
pub fn new_uid_gen(namespace: u16) -> UidGen {
    UidGen {
        counter: 1,
        namespace,
        recycled: Vec::new(),
    }
}

/// Allocate a new UID (uses recycled pool first).
#[allow(dead_code)]
pub fn ug_alloc(gen: &mut UidGen) -> u64 {
    if let Some(uid) = gen.recycled.pop() {
        return uid;
    }
    let uid = ((gen.namespace as u64) << 48) | gen.counter;
    gen.counter += 1;
    uid
}

/// Recycle a UID for future reuse.
#[allow(dead_code)]
pub fn ug_recycle(gen: &mut UidGen, uid: u64) {
    if !gen.recycled.contains(&uid) {
        gen.recycled.push(uid);
    }
}

/// Number of UIDs allocated so far (not counting recycled).
#[allow(dead_code)]
pub fn ug_allocated_count(gen: &UidGen) -> u64 {
    gen.counter - 1
}

/// Number of UIDs in the recycled pool.
#[allow(dead_code)]
pub fn ug_recycled_count(gen: &UidGen) -> usize {
    gen.recycled.len()
}

/// Extract the namespace from a UID.
#[allow(dead_code)]
pub fn ug_namespace(uid: u64) -> u16 {
    (uid >> 48) as u16
}

/// Extract the local counter from a UID.
#[allow(dead_code)]
pub fn ug_local_id(uid: u64) -> u64 {
    uid & 0x0000_FFFF_FFFF_FFFF
}

/// Reset the generator (keeps namespace).
#[allow(dead_code)]
pub fn ug_reset(gen: &mut UidGen) {
    gen.counter = 1;
    gen.recycled.clear();
}

/// Peek at the next UID that would be allocated (without consuming).
#[allow(dead_code)]
pub fn ug_peek_next(gen: &UidGen) -> u64 {
    if let Some(&uid) = gen.recycled.last() {
        return uid;
    }
    ((gen.namespace as u64) << 48) | gen.counter
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_unique() {
        let mut gen = new_uid_gen(1);
        let a = ug_alloc(&mut gen);
        let b = ug_alloc(&mut gen);
        assert_ne!(a, b);
    }

    #[test]
    fn test_namespace_encoded() {
        let mut gen = new_uid_gen(7);
        let uid = ug_alloc(&mut gen);
        assert_eq!(ug_namespace(uid), 7);
    }

    #[test]
    fn test_local_id_starts_at_one() {
        let mut gen = new_uid_gen(0);
        let uid = ug_alloc(&mut gen);
        assert_eq!(ug_local_id(uid), 1);
    }

    #[test]
    fn test_recycle_reuses() {
        let mut gen = new_uid_gen(0);
        let uid1 = ug_alloc(&mut gen);
        ug_recycle(&mut gen, uid1);
        let uid2 = ug_alloc(&mut gen);
        assert_eq!(uid1, uid2);
    }

    #[test]
    fn test_recycled_count() {
        let mut gen = new_uid_gen(0);
        let uid = ug_alloc(&mut gen);
        ug_recycle(&mut gen, uid);
        assert_eq!(ug_recycled_count(&gen), 1);
    }

    #[test]
    fn test_allocated_count() {
        let mut gen = new_uid_gen(0);
        ug_alloc(&mut gen);
        ug_alloc(&mut gen);
        assert_eq!(ug_allocated_count(&gen), 2);
    }

    #[test]
    fn test_reset() {
        let mut gen = new_uid_gen(3);
        ug_alloc(&mut gen);
        ug_reset(&mut gen);
        assert_eq!(ug_allocated_count(&gen), 0);
    }

    #[test]
    fn test_peek_next() {
        let gen = new_uid_gen(0);
        let expected = 1u64;
        assert_eq!(ug_peek_next(&gen), expected);
    }

    #[test]
    fn test_no_duplicate_recycle() {
        let mut gen = new_uid_gen(0);
        let uid = ug_alloc(&mut gen);
        ug_recycle(&mut gen, uid);
        ug_recycle(&mut gen, uid);
        assert_eq!(ug_recycled_count(&gen), 1);
    }
}
