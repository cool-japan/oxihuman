// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Accumulate impulses over solver iterations.

#![allow(dead_code)]

/// A single accumulated impulse entry for one body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AccumEntry {
    pub id: u32,
    pub impulse: [f32; 3],
    pub torque: [f32; 3],
}

/// Accumulator collecting impulses across solver iterations.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ImpulseAccumulator {
    pub entries: Vec<AccumEntry>,
}

/// Creates a new empty impulse accumulator.
#[allow(dead_code)]
pub fn new_impulse_accumulator() -> ImpulseAccumulator {
    ImpulseAccumulator {
        entries: Vec::new(),
    }
}

/// Adds or accumulates impulse and torque for a body id.
#[allow(dead_code)]
pub fn accum_add(acc: &mut ImpulseAccumulator, id: u32, impulse: [f32; 3], torque: [f32; 3]) {
    if let Some(entry) = acc.entries.iter_mut().find(|e| e.id == id) {
        entry.impulse[0] += impulse[0];
        entry.impulse[1] += impulse[1];
        entry.impulse[2] += impulse[2];
        entry.torque[0] += torque[0];
        entry.torque[1] += torque[1];
        entry.torque[2] += torque[2];
    } else {
        acc.entries.push(AccumEntry { id, impulse, torque });
    }
}

/// Returns the accumulated entry for a given body id, if it exists.
#[allow(dead_code)]
pub fn accum_get(acc: &ImpulseAccumulator, id: u32) -> Option<&AccumEntry> {
    acc.entries.iter().find(|e| e.id == id)
}

/// Clears all accumulated entries.
#[allow(dead_code)]
pub fn accum_clear(acc: &mut ImpulseAccumulator) {
    acc.entries.clear();
}

/// Returns the number of bodies with accumulated data.
#[allow(dead_code)]
pub fn accum_count(acc: &ImpulseAccumulator) -> usize {
    acc.entries.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    #[test]
    fn test_new_empty() {
        let acc = new_impulse_accumulator();
        assert_eq!(accum_count(&acc), 0);
    }

    #[test]
    fn test_add_new_entry() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 1, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert_eq!(accum_count(&acc), 1);
    }

    #[test]
    fn test_add_accumulates() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 1, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        accum_add(&mut acc, 1, [2.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let e = accum_get(&acc, 1).unwrap();
        assert!((e.impulse[0] - 3.0).abs() < EPS);
    }

    #[test]
    fn test_get_missing() {
        let acc = new_impulse_accumulator();
        assert!(accum_get(&acc, 99).is_none());
    }

    #[test]
    fn test_clear() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 1, [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        accum_clear(&mut acc);
        assert_eq!(accum_count(&acc), 0);
    }

    #[test]
    fn test_torque_accumulated() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 5, [0.0; 3], [1.0, 2.0, 3.0]);
        accum_add(&mut acc, 5, [0.0; 3], [0.5, 0.5, 0.5]);
        let e = accum_get(&acc, 5).unwrap();
        assert!((e.torque[0] - 1.5).abs() < EPS);
    }

    #[test]
    fn test_multiple_bodies() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 1, [1.0, 0.0, 0.0], [0.0; 3]);
        accum_add(&mut acc, 2, [0.0, 1.0, 0.0], [0.0; 3]);
        assert_eq!(accum_count(&acc), 2);
    }

    #[test]
    fn test_get_correct_id() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 10, [5.0, 0.0, 0.0], [0.0; 3]);
        let e = accum_get(&acc, 10).unwrap();
        assert_eq!(e.id, 10);
        assert!((e.impulse[0] - 5.0).abs() < EPS);
    }

    #[test]
    fn test_clear_then_add() {
        let mut acc = new_impulse_accumulator();
        accum_add(&mut acc, 1, [1.0, 0.0, 0.0], [0.0; 3]);
        accum_clear(&mut acc);
        accum_add(&mut acc, 2, [2.0, 0.0, 0.0], [0.0; 3]);
        assert_eq!(accum_count(&acc), 1);
    }
}
