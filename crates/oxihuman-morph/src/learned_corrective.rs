// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ML-learned corrective shape stub.

/// A single learned corrective entry mapping a driver value to a delta.
#[derive(Debug, Clone)]
pub struct CorrectiveEntry {
    pub driver_index: usize,
    pub driver_value: f32,
    pub delta: Vec<[f32; 3]>,
    pub weight: f32,
}

/// Learned corrective shape system.
#[derive(Debug, Clone)]
pub struct LearnedCorrective {
    pub entries: Vec<CorrectiveEntry>,
    pub vertex_count: usize,
    pub enabled: bool,
}

impl LearnedCorrective {
    pub fn new(vertex_count: usize) -> Self {
        LearnedCorrective {
            entries: Vec::new(),
            vertex_count,
            enabled: true,
        }
    }
}

/// Create a new learned corrective system.
pub fn new_learned_corrective(vertex_count: usize) -> LearnedCorrective {
    LearnedCorrective::new(vertex_count)
}

/// Add a corrective entry.
pub fn lc_add_entry(lc: &mut LearnedCorrective, entry: CorrectiveEntry) {
    lc.entries.push(entry);
}

/// Evaluate all corrective entries and accumulate deltas (stub: zeroed output).
pub fn lc_evaluate(lc: &LearnedCorrective, _drivers: &[f32]) -> Vec<[f32; 3]> {
    /* Stub: returns zeroed delta array */
    vec![[0.0; 3]; lc.vertex_count]
}

/// Return entry count.
pub fn lc_entry_count(lc: &LearnedCorrective) -> usize {
    lc.entries.len()
}

/// Enable or disable the corrective system.
pub fn lc_set_enabled(lc: &mut LearnedCorrective, enabled: bool) {
    lc.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn lc_to_json(lc: &LearnedCorrective) -> String {
    format!(
        r#"{{"vertex_count":{},"entry_count":{},"enabled":{}}}"#,
        lc.vertex_count,
        lc.entries.len(),
        lc.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_vertex_count() {
        let lc = new_learned_corrective(100);
        assert_eq!(lc.vertex_count, 100 /* vertex count must match */,);
    }

    #[test]
    fn test_default_no_entries() {
        let lc = new_learned_corrective(10);
        assert_eq!(lc_entry_count(&lc), 0 /* initially no entries */,);
    }

    #[test]
    fn test_add_entry() {
        let mut lc = new_learned_corrective(4);
        let e = CorrectiveEntry {
            driver_index: 0,
            driver_value: 1.0,
            delta: vec![[0.1, 0.0, 0.0]; 4],
            weight: 1.0,
        };
        lc_add_entry(&mut lc, e);
        assert_eq!(
            lc_entry_count(&lc),
            1, /* entry count must be 1 after add */
        );
    }

    #[test]
    fn test_evaluate_length() {
        let lc = new_learned_corrective(8);
        let out = lc_evaluate(&lc, &[]);
        assert_eq!(
            out.len(),
            8, /* evaluate output length must match vertex count */
        );
    }

    #[test]
    fn test_evaluate_zeroed() {
        let lc = new_learned_corrective(3);
        let out = lc_evaluate(&lc, &[1.0]);
        assert!((out[0][0]).abs() < 1e-6 /* stub output must be zero */,);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut lc = new_learned_corrective(4);
        lc_set_enabled(&mut lc, false);
        assert!(!lc.enabled /* enabled flag must be false */,);
    }

    #[test]
    fn test_to_json_contains_vertex_count() {
        let lc = new_learned_corrective(20);
        let j = lc_to_json(&lc);
        assert!(j.contains("\"vertex_count\""), /* json must contain vertex_count */);
    }

    #[test]
    fn test_to_json_contains_entry_count() {
        let lc = new_learned_corrective(5);
        let j = lc_to_json(&lc);
        assert!(j.contains("\"entry_count\""), /* json must contain entry_count */);
    }

    #[test]
    fn test_multiple_entries() {
        let mut lc = new_learned_corrective(2);
        for i in 0..5 {
            lc_add_entry(
                &mut lc,
                CorrectiveEntry {
                    driver_index: i,
                    driver_value: 0.5,
                    delta: vec![[0.0; 3]; 2],
                    weight: 1.0,
                },
            );
        }
        assert_eq!(
            lc_entry_count(&lc),
            5, /* five entries must be stored */
        );
    }

    #[test]
    fn test_enabled_by_default() {
        let lc = new_learned_corrective(1);
        assert!(lc.enabled /* must be enabled by default */,);
    }
}
