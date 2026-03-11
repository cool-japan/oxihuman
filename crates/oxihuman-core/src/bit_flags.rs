// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic 64-bit flag set.

#![allow(dead_code)]

/// A 64-bit flag set.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitFlags64 {
    pub bits: u64,
}

/// Create a new zeroed flag set.
#[allow(dead_code)]
pub fn new_bit_flags() -> BitFlags64 {
    BitFlags64 { bits: 0 }
}

/// Set bit `bit` (0-63).
#[allow(dead_code)]
pub fn set_flag(flags: &mut BitFlags64, bit: u8) {
    if bit < 64 {
        flags.bits |= 1u64 << bit;
    }
}

/// Clear bit `bit` (0-63).
#[allow(dead_code)]
pub fn clear_flag(flags: &mut BitFlags64, bit: u8) {
    if bit < 64 {
        flags.bits &= !(1u64 << bit);
    }
}

/// Return true if bit `bit` is set.
#[allow(dead_code)]
pub fn has_flag(flags: &BitFlags64, bit: u8) -> bool {
    bit < 64 && (flags.bits & (1u64 << bit)) != 0
}

/// Toggle bit `bit`.
#[allow(dead_code)]
pub fn toggle_flag(flags: &mut BitFlags64, bit: u8) {
    if bit < 64 {
        flags.bits ^= 1u64 << bit;
    }
}

/// Count the number of set bits.
#[allow(dead_code)]
pub fn flags_count(flags: &BitFlags64) -> u32 {
    flags.bits.count_ones()
}

/// Return true if any bit is set.
#[allow(dead_code)]
pub fn any_set(flags: &BitFlags64) -> bool {
    flags.bits != 0
}

/// Return true if no bits are set.
#[allow(dead_code)]
pub fn all_clear(flags: &BitFlags64) -> bool {
    flags.bits == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flags_all_clear() {
        let f = new_bit_flags();
        assert!(all_clear(&f));
        assert!(!any_set(&f));
    }

    #[test]
    fn test_set_and_has_flag() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 3);
        assert!(has_flag(&f, 3));
        assert!(!has_flag(&f, 4));
    }

    #[test]
    fn test_clear_flag() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 7);
        clear_flag(&mut f, 7);
        assert!(!has_flag(&f, 7));
    }

    #[test]
    fn test_toggle_flag() {
        let mut f = new_bit_flags();
        toggle_flag(&mut f, 1);
        assert!(has_flag(&f, 1));
        toggle_flag(&mut f, 1);
        assert!(!has_flag(&f, 1));
    }

    #[test]
    fn test_flags_count() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 0);
        set_flag(&mut f, 10);
        set_flag(&mut f, 63);
        assert_eq!(flags_count(&f), 3);
    }

    #[test]
    fn test_any_set_after_set() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 5);
        assert!(any_set(&f));
    }

    #[test]
    fn test_all_clear_after_clear() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 5);
        clear_flag(&mut f, 5);
        assert!(all_clear(&f));
    }

    #[test]
    fn test_bit_63_boundary() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 63);
        assert!(has_flag(&f, 63));
        assert_eq!(flags_count(&f), 1);
    }

    #[test]
    fn test_set_flag_above_63_is_noop() {
        let mut f = new_bit_flags();
        set_flag(&mut f, 64); // should be ignored
        assert!(all_clear(&f));
    }
}
