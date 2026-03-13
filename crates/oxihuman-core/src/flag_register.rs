// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A register of named boolean flags stored as a bitmask.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlagRegister {
    names: Vec<String>,
    bits: u64,
}

#[allow(dead_code)]
impl FlagRegister {
    pub fn new() -> Self {
        Self {
            names: Vec::new(),
            bits: 0,
        }
    }

    pub fn register(&mut self, name: &str) -> Option<usize> {
        if self.names.len() >= 64 || self.names.iter().any(|n| n == name) {
            return None;
        }
        let idx = self.names.len();
        self.names.push(name.to_string());
        Some(idx)
    }

    pub fn set(&mut self, idx: usize) {
        if idx < 64 {
            self.bits |= 1u64 << idx;
        }
    }

    pub fn clear_flag(&mut self, idx: usize) {
        if idx < 64 {
            self.bits &= !(1u64 << idx);
        }
    }

    pub fn toggle(&mut self, idx: usize) {
        if idx < 64 {
            self.bits ^= 1u64 << idx;
        }
    }

    pub fn is_set(&self, idx: usize) -> bool {
        if idx >= 64 {
            return false;
        }
        (self.bits & (1u64 << idx)) != 0
    }

    pub fn set_by_name(&mut self, name: &str) -> bool {
        if let Some(idx) = self.index_of(name) {
            self.set(idx);
            true
        } else {
            false
        }
    }

    pub fn is_set_by_name(&self, name: &str) -> bool {
        self.index_of(name).is_some_and(|idx| self.is_set(idx))
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    pub fn count_set(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn count_registered(&self) -> usize {
        self.names.len()
    }

    pub fn clear_all(&mut self) {
        self.bits = 0;
    }

    pub fn set_all(&mut self) {
        let n = self.names.len();
        if n >= 64 {
            self.bits = u64::MAX;
        } else {
            self.bits = (1u64 << n) - 1;
        }
    }

    pub fn raw_bits(&self) -> u64 {
        self.bits
    }
}

impl Default for FlagRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fr = FlagRegister::new();
        assert_eq!(fr.count_registered(), 0);
        assert_eq!(fr.count_set(), 0);
    }

    #[test]
    fn test_register() {
        let mut fr = FlagRegister::new();
        let idx = fr.register("flag_a");
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn test_register_duplicate() {
        let mut fr = FlagRegister::new();
        fr.register("x");
        assert_eq!(fr.register("x"), None);
    }

    #[test]
    fn test_set_and_check() {
        let mut fr = FlagRegister::new();
        let idx = fr.register("f").expect("should succeed");
        fr.set(idx);
        assert!(fr.is_set(idx));
    }

    #[test]
    fn test_clear_flag() {
        let mut fr = FlagRegister::new();
        let idx = fr.register("f").expect("should succeed");
        fr.set(idx);
        fr.clear_flag(idx);
        assert!(!fr.is_set(idx));
    }

    #[test]
    fn test_toggle() {
        let mut fr = FlagRegister::new();
        let idx = fr.register("f").expect("should succeed");
        fr.toggle(idx);
        assert!(fr.is_set(idx));
        fr.toggle(idx);
        assert!(!fr.is_set(idx));
    }

    #[test]
    fn test_set_by_name() {
        let mut fr = FlagRegister::new();
        fr.register("alpha");
        assert!(fr.set_by_name("alpha"));
        assert!(fr.is_set_by_name("alpha"));
    }

    #[test]
    fn test_count_set() {
        let mut fr = FlagRegister::new();
        fr.register("a");
        fr.register("b");
        fr.register("c");
        fr.set(0);
        fr.set(2);
        assert_eq!(fr.count_set(), 2);
    }

    #[test]
    fn test_clear_all() {
        let mut fr = FlagRegister::new();
        fr.register("a");
        fr.set(0);
        fr.clear_all();
        assert_eq!(fr.count_set(), 0);
    }

    #[test]
    fn test_set_all() {
        let mut fr = FlagRegister::new();
        fr.register("a");
        fr.register("b");
        fr.register("c");
        fr.set_all();
        assert_eq!(fr.count_set(), 3);
    }
}
