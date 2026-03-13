// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Coefficient-of-restitution (COR) lookup table for material pairs.

use std::collections::HashMap;

/// Coefficient-of-restitution lookup table.
#[derive(Debug, Clone)]
pub struct BounceTable {
    /// Map from (material_a, material_b) → COR.
    table: HashMap<(String, String), f64>,
    /// Default COR when a pair is not found.
    pub default_cor: f64,
}

impl BounceTable {
    /// Create a new bounce table with default COR.
    pub fn new(default_cor: f64) -> Self {
        BounceTable { table: HashMap::new(), default_cor: default_cor.clamp(0.0, 1.0) }
    }

    /// Register a COR for a material pair.  Order of materials does not matter.
    pub fn register(&mut self, mat_a: &str, mat_b: &str, cor: f64) {
        let cor = cor.clamp(0.0, 1.0);
        self.table.insert((mat_a.to_owned(), mat_b.to_owned()), cor);
        self.table.insert((mat_b.to_owned(), mat_a.to_owned()), cor);
    }

    /// Look up the COR for a material pair (returns default if not found).
    pub fn lookup(&self, mat_a: &str, mat_b: &str) -> f64 {
        *self
            .table
            .get(&(mat_a.to_owned(), mat_b.to_owned()))
            .unwrap_or(&self.default_cor)
    }

    /// Apply the bounce: given incoming speed `v_in`, return rebound speed.
    pub fn rebound_speed(&self, mat_a: &str, mat_b: &str, v_in: f64) -> f64 {
        self.lookup(mat_a, mat_b) * v_in.abs()
    }

    /// Energy retained after a bounce (ratio).
    pub fn energy_ratio(&self, mat_a: &str, mat_b: &str) -> f64 {
        let cor = self.lookup(mat_a, mat_b);
        cor * cor
    }

    /// Number of registered pairs (each pair is stored twice, so divides by 2).
    pub fn pair_count(&self) -> usize {
        self.table.len() / 2
    }

    /// True if the pair is registered.
    pub fn has_pair(&self, mat_a: &str, mat_b: &str) -> bool {
        self.table.contains_key(&(mat_a.to_owned(), mat_b.to_owned()))
    }
}

/// Create a bounce table with common material pairs pre-populated.
pub fn default_bounce_table() -> BounceTable {
    let mut t = BounceTable::new(0.5);
    t.register("rubber", "concrete", 0.8);
    t.register("steel", "steel", 0.7);
    t.register("glass", "glass", 0.65);
    t.register("rubber", "rubber", 0.85);
    t.register("wood", "concrete", 0.4);
    t.register("clay", "concrete", 0.1);
    t
}

/// Create a new bounce table.
pub fn new_bounce_table(default_cor: f64) -> BounceTable {
    BounceTable::new(default_cor)
}

/// Look up COR.
pub fn bt_lookup(table: &BounceTable, mat_a: &str, mat_b: &str) -> f64 {
    table.lookup(mat_a, mat_b)
}

/// Rebound speed.
pub fn bt_rebound_speed(table: &BounceTable, mat_a: &str, mat_b: &str, v_in: f64) -> f64 {
    table.rebound_speed(mat_a, mat_b, v_in)
}

/// Register a pair.
pub fn bt_register(table: &mut BounceTable, mat_a: &str, mat_b: &str, cor: f64) {
    table.register(mat_a, mat_b, cor);
}

/// Energy ratio after bounce.
pub fn bt_energy_ratio(table: &BounceTable, mat_a: &str, mat_b: &str) -> f64 {
    table.energy_ratio(mat_a, mat_b)
}

/// Pair count.
pub fn bt_pair_count(table: &BounceTable) -> usize {
    table.pair_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "ball", "floor", 0.7);
        assert!((bt_lookup(&t, "ball", "floor") - 0.7).abs() < 1e-9 /* registered COR */);
    }

    #[test]
    fn test_symmetric_lookup() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "A", "B", 0.6);
        assert!((bt_lookup(&t, "B", "A") - 0.6).abs() < 1e-9 /* symmetric */);
    }

    #[test]
    fn test_default_cor_for_unknown() {
        let t = new_bounce_table(0.3);
        assert!((bt_lookup(&t, "unknown1", "unknown2") - 0.3).abs() < 1e-9 /* default */);
    }

    #[test]
    fn test_rebound_speed() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "ball", "wall", 0.8);
        let v = bt_rebound_speed(&t, "ball", "wall", 10.0);
        assert!((v - 8.0).abs() < 1e-9 /* 0.8 * 10 */);
    }

    #[test]
    fn test_energy_ratio() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "x", "y", 0.5);
        let er = bt_energy_ratio(&t, "x", "y");
        assert!((er - 0.25).abs() < 1e-9 /* 0.5² = 0.25 */);
    }

    #[test]
    fn test_perfect_bounce() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "elastic", "wall", 1.0);
        assert!((bt_energy_ratio(&t, "elastic", "wall") - 1.0).abs() < 1e-9 /* elastic */);
    }

    #[test]
    fn test_pair_count() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "a", "b", 0.6);
        bt_register(&mut t, "c", "d", 0.7);
        assert_eq!(bt_pair_count(&t), 2 /* two pairs */);
    }

    #[test]
    fn test_cor_clamped_to_one() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "x", "y", 1.5); /* clamped to 1 */
        assert!((bt_lookup(&t, "x", "y") - 1.0).abs() < 1e-9 /* clamped */);
    }

    #[test]
    fn test_default_table_has_pairs() {
        let t = default_bounce_table();
        assert!(bt_pair_count(&t) > 0 /* pre-populated */);
    }

    #[test]
    fn test_has_pair() {
        let mut t = new_bounce_table(0.5);
        bt_register(&mut t, "m1", "m2", 0.6);
        assert!(t.has_pair("m1", "m2") /* pair registered */);
        assert!(!t.has_pair("m1", "m3") /* pair not registered */);
    }
}
