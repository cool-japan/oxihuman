#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A strong reference with an ID and reference count.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StrongRef {
    id: u64,
    strong: u32,
    weak: u32,
    alive: bool,
}

/// A weak reference that may or may not still be valid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeakRef {
    id: u64,
    alive: bool,
}

#[allow(dead_code)]
pub fn new_strong_ref(id: u64) -> StrongRef {
    StrongRef {
        id,
        strong: 1,
        weak: 0,
        alive: true,
    }
}

#[allow(dead_code)]
pub fn downgrade(strong: &mut StrongRef) -> WeakRef {
    strong.weak += 1;
    WeakRef {
        id: strong.id,
        alive: strong.alive,
    }
}

#[allow(dead_code)]
pub fn upgrade(weak: &WeakRef) -> Option<u64> {
    if weak.alive {
        Some(weak.id)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn is_alive(strong: &StrongRef) -> bool {
    strong.alive
}

#[allow(dead_code)]
pub fn ref_count(strong: &StrongRef) -> u32 {
    strong.strong + strong.weak
}

#[allow(dead_code)]
pub fn strong_count(strong: &StrongRef) -> u32 {
    strong.strong
}

#[allow(dead_code)]
pub fn weak_count(strong: &StrongRef) -> u32 {
    strong.weak
}

#[allow(dead_code)]
pub fn ref_id(strong: &StrongRef) -> u64 {
    strong.id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_strong_ref() {
        let s = new_strong_ref(42);
        assert_eq!(s.id, 42);
        assert!(s.alive);
    }

    #[test]
    fn test_downgrade() {
        let mut s = new_strong_ref(1);
        let w = downgrade(&mut s);
        assert_eq!(w.id, 1);
        assert!(w.alive);
    }

    #[test]
    fn test_upgrade() {
        let mut s = new_strong_ref(1);
        let w = downgrade(&mut s);
        assert_eq!(upgrade(&w), Some(1));
    }

    #[test]
    fn test_upgrade_dead() {
        let w = WeakRef {
            id: 1,
            alive: false,
        };
        assert_eq!(upgrade(&w), None);
    }

    #[test]
    fn test_is_alive() {
        let s = new_strong_ref(1);
        assert!(is_alive(&s));
    }

    #[test]
    fn test_ref_count() {
        let mut s = new_strong_ref(1);
        assert_eq!(ref_count(&s), 1);
        downgrade(&mut s);
        assert_eq!(ref_count(&s), 2);
    }

    #[test]
    fn test_strong_count() {
        let s = new_strong_ref(1);
        assert_eq!(strong_count(&s), 1);
    }

    #[test]
    fn test_weak_count() {
        let mut s = new_strong_ref(1);
        assert_eq!(weak_count(&s), 0);
        downgrade(&mut s);
        assert_eq!(weak_count(&s), 1);
    }

    #[test]
    fn test_ref_id() {
        let s = new_strong_ref(99);
        assert_eq!(ref_id(&s), 99);
    }

    #[test]
    fn test_multiple_downgrades() {
        let mut s = new_strong_ref(1);
        downgrade(&mut s);
        downgrade(&mut s);
        assert_eq!(weak_count(&s), 2);
    }
}
