// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct CapabilityFlags {
    pub bits: u64,
}

impl CapabilityFlags {
    pub fn new() -> Self {
        CapabilityFlags { bits: 0 }
    }
}

impl Default for CapabilityFlags {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_capability_flags() -> CapabilityFlags {
    CapabilityFlags::new()
}

pub fn flags_set(f: &mut CapabilityFlags, bit: u64) {
    f.bits |= bit;
}

pub fn flags_clear(f: &mut CapabilityFlags, bit: u64) {
    f.bits &= !bit;
}

pub fn flags_test(f: &CapabilityFlags, bit: u64) -> bool {
    f.bits & bit != 0
}

pub fn flags_any(f: &CapabilityFlags) -> bool {
    f.bits != 0
}

pub fn flags_all(f: &CapabilityFlags, mask: u64) -> bool {
    f.bits & mask == mask
}

pub fn flags_count(f: &CapabilityFlags) -> u32 {
    f.bits.count_ones()
}

pub fn flags_reset(f: &mut CapabilityFlags) {
    f.bits = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero() {
        /* new flags are all zero */
        let f = new_capability_flags();
        assert!(!flags_any(&f));
        assert_eq!(flags_count(&f), 0);
    }

    #[test]
    fn test_set_and_test() {
        /* set then test a bit */
        let mut f = new_capability_flags();
        flags_set(&mut f, 0b0100);
        assert!(flags_test(&f, 0b0100));
        assert!(!flags_test(&f, 0b0010));
    }

    #[test]
    fn test_clear() {
        /* clear removes a specific bit */
        let mut f = new_capability_flags();
        flags_set(&mut f, 0b1111);
        flags_clear(&mut f, 0b0010);
        assert!(flags_test(&f, 0b0001));
        assert!(!flags_test(&f, 0b0010));
    }

    #[test]
    fn test_flags_any() {
        /* flags_any detects any set bit */
        let mut f = new_capability_flags();
        assert!(!flags_any(&f));
        flags_set(&mut f, 1);
        assert!(flags_any(&f));
    }

    #[test]
    fn test_flags_all() {
        /* flags_all checks all mask bits */
        let mut f = new_capability_flags();
        flags_set(&mut f, 0b0011);
        assert!(flags_all(&f, 0b0011));
        assert!(!flags_all(&f, 0b0111));
    }

    #[test]
    fn test_flags_count() {
        /* count set bits */
        let mut f = new_capability_flags();
        flags_set(&mut f, 0b1011);
        assert_eq!(flags_count(&f), 3);
    }

    #[test]
    fn test_reset() {
        /* reset clears all bits */
        let mut f = new_capability_flags();
        flags_set(&mut f, u64::MAX);
        flags_reset(&mut f);
        assert!(!flags_any(&f));
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let f = CapabilityFlags::default();
        assert_eq!(f.bits, 0);
    }
}
