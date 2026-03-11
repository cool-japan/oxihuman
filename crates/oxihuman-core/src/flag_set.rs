// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compact u64 bitset flag collection.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FlagSet {
    pub bits: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlagSetConfig {
    pub max_flags: usize,
}

#[allow(dead_code)]
pub fn default_flag_set_config() -> FlagSetConfig {
    FlagSetConfig { max_flags: 64 }
}

#[allow(dead_code)]
pub fn new_flag_set() -> FlagSet {
    FlagSet { bits: 0 }
}

#[allow(dead_code)]
pub fn fs_set(fs: &mut FlagSet, flag: u8) {
    if flag < 64 {
        fs.bits |= 1u64 << flag;
    }
}

#[allow(dead_code)]
pub fn fs_clear(fs: &mut FlagSet, flag: u8) {
    if flag < 64 {
        fs.bits &= !(1u64 << flag);
    }
}

#[allow(dead_code)]
pub fn fs_get(fs: &FlagSet, flag: u8) -> bool {
    if flag < 64 {
        fs.bits & (1u64 << flag) != 0
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn fs_toggle(fs: &mut FlagSet, flag: u8) {
    if flag < 64 {
        fs.bits ^= 1u64 << flag;
    }
}

#[allow(dead_code)]
pub fn fs_count(fs: &FlagSet) -> u32 {
    fs.bits.count_ones()
}

#[allow(dead_code)]
pub fn fs_any(fs: &FlagSet) -> bool {
    fs.bits != 0
}

#[allow(dead_code)]
pub fn fs_all_of(fs: &FlagSet, mask: u64) -> bool {
    fs.bits & mask == mask
}

#[allow(dead_code)]
pub fn fs_reset(fs: &mut FlagSet) {
    fs.bits = 0;
}

#[allow(dead_code)]
pub fn fs_to_u64(fs: &FlagSet) -> u64 {
    fs.bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let fs = new_flag_set();
        assert!(!fs_any(&fs));
        assert_eq!(fs_count(&fs), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 3);
        assert!(fs_get(&fs, 3));
        assert!(!fs_get(&fs, 4));
    }

    #[test]
    fn test_clear() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 5);
        fs_clear(&mut fs, 5);
        assert!(!fs_get(&fs, 5));
    }

    #[test]
    fn test_toggle() {
        let mut fs = new_flag_set();
        fs_toggle(&mut fs, 7);
        assert!(fs_get(&fs, 7));
        fs_toggle(&mut fs, 7);
        assert!(!fs_get(&fs, 7));
    }

    #[test]
    fn test_count() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 0);
        fs_set(&mut fs, 1);
        fs_set(&mut fs, 63);
        assert_eq!(fs_count(&fs), 3);
    }

    #[test]
    fn test_any() {
        let mut fs = new_flag_set();
        assert!(!fs_any(&fs));
        fs_set(&mut fs, 10);
        assert!(fs_any(&fs));
    }

    #[test]
    fn test_all_of() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 0);
        fs_set(&mut fs, 1);
        assert!(fs_all_of(&fs, 0b11));
        assert!(!fs_all_of(&fs, 0b111));
    }

    #[test]
    fn test_reset() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 15);
        fs_reset(&mut fs);
        assert!(!fs_any(&fs));
    }

    #[test]
    fn test_to_u64() {
        let mut fs = new_flag_set();
        fs_set(&mut fs, 0);
        assert_eq!(fs_to_u64(&fs), 1u64);
    }
}
