// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mesh anchor/pin point for simulation — marks vertices as fixed constraints.

/// A single anchor pin on the mesh.
#[derive(Debug, Clone)]
pub struct AnchorPoint {
    pub vertex_index: usize,
    pub strength: f32,
    pub label: String,
}

/// Collection of anchor points for a mesh.
#[derive(Debug, Default)]
pub struct AnchorPointSet {
    anchors: Vec<AnchorPoint>,
}

/// Create a new, empty anchor-point set.
pub fn new_anchor_set() -> AnchorPointSet {
    AnchorPointSet::default()
}

/// Add an anchor with full strength at the given vertex.
pub fn add_anchor(set: &mut AnchorPointSet, vertex_index: usize, label: &str) {
    set.anchors.push(AnchorPoint {
        vertex_index,
        strength: 1.0,
        label: label.to_owned(),
    });
}

/// Add an anchor with a custom strength in `[0, 1]`.
pub fn add_anchor_weighted(
    set: &mut AnchorPointSet,
    vertex_index: usize,
    strength: f32,
    label: &str,
) {
    let clamped = strength.clamp(0.0, 1.0);
    set.anchors.push(AnchorPoint {
        vertex_index,
        strength: clamped,
        label: label.to_owned(),
    });
}

/// Return the number of anchors in the set.
pub fn anchor_count(set: &AnchorPointSet) -> usize {
    set.anchors.len()
}

/// Find the first anchor for a given vertex index.
pub fn find_anchor(set: &AnchorPointSet, vertex_index: usize) -> Option<&AnchorPoint> {
    set.anchors.iter().find(|a| a.vertex_index == vertex_index)
}

/// Remove all anchors whose strength is below the threshold.
pub fn prune_weak_anchors(set: &mut AnchorPointSet, threshold: f32) {
    set.anchors.retain(|a| a.strength >= threshold);
}

/// Average strength of all anchors.
pub fn average_anchor_strength(set: &AnchorPointSet) -> f32 {
    if set.anchors.is_empty() {
        return 0.0;
    }
    let sum: f32 = set.anchors.iter().map(|a| a.strength).sum();
    sum / set.anchors.len() as f32
}

/// Serialize the anchor set to a simple JSON-style string.
pub fn anchor_set_to_json(set: &AnchorPointSet) -> String {
    let entries: Vec<String> = set
        .anchors
        .iter()
        .map(|a| {
            format!(
                r#"{{"vertex":{}, "strength":{:.4}, "label":"{}"}}"#,
                a.vertex_index, a.strength, a.label
            )
        })
        .collect();
    format!("[{}]", entries.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        /* anchor set starts with zero entries */
        let s = new_anchor_set();
        assert_eq!(anchor_count(&s), 0);
    }

    #[test]
    fn add_anchor_increments_count() {
        /* adding one anchor should yield count 1 */
        let mut s = new_anchor_set();
        add_anchor(&mut s, 0, "root");
        assert_eq!(anchor_count(&s), 1);
    }

    #[test]
    fn add_anchor_weighted_clamps_strength() {
        /* strength above 1 should be clamped to 1 */
        let mut s = new_anchor_set();
        add_anchor_weighted(&mut s, 5, 1.5, "top");
        assert!((s.anchors[0].strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn find_anchor_returns_correct_vertex() {
        /* find_anchor should return the entry for vertex 3 */
        let mut s = new_anchor_set();
        add_anchor(&mut s, 3, "mid");
        let a = find_anchor(&s, 3).expect("should succeed");
        assert_eq!(a.vertex_index, 3);
    }

    #[test]
    fn find_anchor_missing_returns_none() {
        /* querying a non-existent vertex yields None */
        let s = new_anchor_set();
        assert!(find_anchor(&s, 99).is_none());
    }

    #[test]
    fn prune_removes_weak_anchors() {
        /* anchors below 0.5 strength should be pruned */
        let mut s = new_anchor_set();
        add_anchor_weighted(&mut s, 0, 0.2, "weak");
        add_anchor_weighted(&mut s, 1, 0.8, "strong");
        prune_weak_anchors(&mut s, 0.5);
        assert_eq!(anchor_count(&s), 1);
    }

    #[test]
    fn average_strength_correct() {
        /* average of 0.4 and 0.6 should be 0.5 */
        let mut s = new_anchor_set();
        add_anchor_weighted(&mut s, 0, 0.4, "a");
        add_anchor_weighted(&mut s, 1, 0.6, "b");
        assert!((average_anchor_strength(&s) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn average_strength_empty_set_returns_zero() {
        /* empty set average should be 0 */
        let s = new_anchor_set();
        assert_eq!(average_anchor_strength(&s), 0.0);
    }

    #[test]
    fn json_output_contains_vertex_key() {
        /* JSON string should contain the word vertex */
        let mut s = new_anchor_set();
        add_anchor(&mut s, 7, "pin");
        let j = anchor_set_to_json(&s);
        assert!(j.contains("vertex"));
    }

    #[test]
    fn json_output_empty_is_empty_array() {
        /* empty set serialises to [] */
        let s = new_anchor_set();
        assert_eq!(anchor_set_to_json(&s), "[]");
    }
}
