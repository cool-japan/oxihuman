// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct BitmaskFlags {
    pub bits: u32,
    pub names: Vec<String>,
}

pub fn new_bitmask_flags(names: Vec<&str>) -> BitmaskFlags {
    BitmaskFlags {
        bits: 0,
        names: names.iter().map(|s| s.to_string()).collect(),
    }
}

fn find_idx(f: &BitmaskFlags, name: &str) -> Option<usize> {
    f.names.iter().position(|n| n == name)
}

pub fn bmf_set(f: &mut BitmaskFlags, name: &str) -> bool {
    if let Some(i) = find_idx(f, name) {
        f.bits |= 1 << i;
        true
    } else {
        false
    }
}

pub fn bmf_clear(f: &mut BitmaskFlags, name: &str) -> bool {
    if let Some(i) = find_idx(f, name) {
        f.bits &= !(1 << i);
        true
    } else {
        false
    }
}

pub fn bmf_test(f: &BitmaskFlags, name: &str) -> bool {
    if let Some(i) = find_idx(f, name) {
        (f.bits >> i) & 1 == 1
    } else {
        false
    }
}

pub fn bmf_raw(f: &BitmaskFlags) -> u32 {
    f.bits
}

pub fn bmf_set_by_index(f: &mut BitmaskFlags, idx: usize) {
    if idx < f.names.len() {
        f.bits |= 1 << idx;
    }
}

pub fn bmf_count_set(f: &BitmaskFlags) -> u32 {
    f.bits.count_ones()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_test() {
        /* set a flag by name and test it */
        let mut f = new_bitmask_flags(vec!["alpha", "beta", "gamma"]);
        bmf_set(&mut f, "beta");
        assert!(bmf_test(&f, "beta"));
        assert!(!bmf_test(&f, "alpha"));
    }

    #[test]
    fn clear_flag() {
        /* clear resets a set flag */
        let mut f = new_bitmask_flags(vec!["x"]);
        bmf_set(&mut f, "x");
        bmf_clear(&mut f, "x");
        assert!(!bmf_test(&f, "x"));
    }

    #[test]
    fn unknown_name_returns_false() {
        /* unknown name returns false without panic */
        let mut f = new_bitmask_flags(vec!["a"]);
        assert!(!bmf_set(&mut f, "zzz"));
    }

    #[test]
    fn raw_bits() {
        /* raw bits match manual expectation */
        let mut f = new_bitmask_flags(vec!["a", "b", "c"]);
        bmf_set(&mut f, "a");
        bmf_set(&mut f, "c");
        assert_eq!(bmf_raw(&f), 0b101);
    }

    #[test]
    fn set_by_index() {
        /* set_by_index works like set by name */
        let mut f = new_bitmask_flags(vec!["x", "y"]);
        bmf_set_by_index(&mut f, 1);
        assert!(bmf_test(&f, "y"));
    }

    #[test]
    fn count_set() {
        /* count_set returns number of 1-bits */
        let mut f = new_bitmask_flags(vec!["a", "b", "c"]);
        bmf_set(&mut f, "a");
        bmf_set(&mut f, "b");
        assert_eq!(bmf_count_set(&f), 2);
    }

    #[test]
    fn initial_all_clear() {
        /* all flags start cleared */
        let f = new_bitmask_flags(vec!["p", "q"]);
        assert_eq!(bmf_count_set(&f), 0);
    }
}
