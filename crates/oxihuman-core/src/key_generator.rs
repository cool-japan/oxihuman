// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Generates unique sequential keys with an optional prefix.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyGenerator {
    prefix: String,
    counter: u64,
    separator: char,
}

#[allow(dead_code)]
impl KeyGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            counter: 0,
            separator: '_',
        }
    }

    pub fn with_separator(prefix: &str, separator: char) -> Self {
        Self {
            prefix: prefix.to_string(),
            counter: 0,
            separator,
        }
    }

    pub fn next_key(&mut self) -> String {
        let key = if self.prefix.is_empty() {
            self.counter.to_string()
        } else {
            format!("{}{}{}", self.prefix, self.separator, self.counter)
        };
        self.counter += 1;
        key
    }

    pub fn next_with_suffix(&mut self, suffix: &str) -> String {
        let key = format!(
            "{}{}{}{}{}",
            self.prefix, self.separator, self.counter, self.separator, suffix
        );
        self.counter += 1;
        key
    }

    pub fn peek(&self) -> u64 {
        self.counter
    }

    pub fn reset(&mut self) {
        self.counter = 0;
    }

    pub fn set_counter(&mut self, val: u64) {
        self.counter = val;
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn total_generated(&self) -> u64 {
        self.counter
    }

    pub fn next_n(&mut self, n: usize) -> Vec<String> {
        (0..n).map(|_| self.next_key()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let kg = KeyGenerator::new("item");
        assert_eq!(kg.prefix(), "item");
        assert_eq!(kg.peek(), 0);
    }

    #[test]
    fn test_next() {
        let mut kg = KeyGenerator::new("obj");
        assert_eq!(kg.next_key(), "obj_0");
        assert_eq!(kg.next_key(), "obj_1");
    }

    #[test]
    fn test_empty_prefix() {
        let mut kg = KeyGenerator::new("");
        assert_eq!(kg.next_key(), "0");
        assert_eq!(kg.next_key(), "1");
    }

    #[test]
    fn test_custom_separator() {
        let mut kg = KeyGenerator::with_separator("node", '-');
        assert_eq!(kg.next_key(), "node-0");
    }

    #[test]
    fn test_next_with_suffix() {
        let mut kg = KeyGenerator::new("tex");
        let key = kg.next_with_suffix("diffuse");
        assert_eq!(key, "tex_0_diffuse");
    }

    #[test]
    fn test_reset() {
        let mut kg = KeyGenerator::new("k");
        kg.next_key();
        kg.next_key();
        kg.reset();
        assert_eq!(kg.peek(), 0);
        assert_eq!(kg.next_key(), "k_0");
    }

    #[test]
    fn test_set_counter() {
        let mut kg = KeyGenerator::new("k");
        kg.set_counter(100);
        assert_eq!(kg.next_key(), "k_100");
    }

    #[test]
    fn test_total_generated() {
        let mut kg = KeyGenerator::new("k");
        kg.next_key();
        kg.next_key();
        kg.next_key();
        assert_eq!(kg.total_generated(), 3);
    }

    #[test]
    fn test_next_n() {
        let mut kg = KeyGenerator::new("v");
        let keys = kg.next_n(3);
        assert_eq!(keys, vec!["v_0", "v_1", "v_2"]);
    }

    #[test]
    fn test_peek_does_not_advance() {
        let kg = KeyGenerator::new("k");
        assert_eq!(kg.peek(), 0);
        assert_eq!(kg.peek(), 0);
    }
}
