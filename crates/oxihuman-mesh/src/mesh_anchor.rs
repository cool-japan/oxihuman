//! Anchor points for mesh deformation.
#![allow(dead_code)]

/// A single anchor point binding a mesh vertex to a target position.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Anchor {
    pub vertex_index: usize,
    pub position: [f32; 3],
    pub weight: f32,
}

/// A collection of anchors.
#[allow(dead_code)]
pub struct AnchorSet {
    pub anchors: Vec<Anchor>,
}

/// Create a new anchor.
#[allow(dead_code)]
pub fn new_anchor(vertex_index: usize, position: [f32; 3], weight: f32) -> Anchor {
    Anchor { vertex_index, position, weight }
}

/// Create a new, empty anchor set.
#[allow(dead_code)]
pub fn anchor_set_new() -> AnchorSet {
    AnchorSet { anchors: Vec::new() }
}

/// Add an anchor to the set.
#[allow(dead_code)]
pub fn add_anchor(set: &mut AnchorSet, anchor: Anchor) {
    set.anchors.push(anchor);
}

/// Remove the anchor at index `i`.
#[allow(dead_code)]
pub fn remove_anchor(set: &mut AnchorSet, i: usize) {
    if i < set.anchors.len() {
        set.anchors.remove(i);
    }
}

/// Get a reference to the anchor at index `i`.
#[allow(dead_code)]
pub fn get_anchor(set: &AnchorSet, i: usize) -> Option<&Anchor> {
    set.anchors.get(i)
}

/// Compute influence of anchor `i` on vertex `v` (1/distance, clamped).
#[allow(dead_code)]
pub fn anchor_influence(set: &AnchorSet, anchor_idx: usize, vertex_pos: [f32; 3]) -> f32 {
    if let Some(a) = get_anchor(set, anchor_idx) {
        let dx = a.position[0] - vertex_pos[0];
        let dy = a.position[1] - vertex_pos[1];
        let dz = a.position[2] - vertex_pos[2];
        let dist = (dx*dx+dy*dy+dz*dz).sqrt();
        if dist < 1e-8 { a.weight } else { a.weight / (1.0 + dist) }
    } else {
        0.0
    }
}

/// Apply anchors to positions: blend each anchor's target position by influence.
#[allow(dead_code)]
pub fn apply_anchors(positions: &[[f32; 3]], set: &AnchorSet) -> Vec<[f32; 3]> {
    let mut result = positions.to_vec();
    for anchor in &set.anchors {
        let vi = anchor.vertex_index;
        if vi >= result.len() { continue; }
        let w = anchor.weight.clamp(0.0, 1.0);
        result[vi][0] = result[vi][0] + (anchor.position[0] - result[vi][0]) * w;
        result[vi][1] = result[vi][1] + (anchor.position[1] - result[vi][1]) * w;
        result[vi][2] = result[vi][2] + (anchor.position[2] - result[vi][2]) * w;
    }
    result
}

/// Return the number of anchors in the set.
#[allow(dead_code)]
pub fn anchor_count(set: &AnchorSet) -> usize {
    set.anchors.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_anchor_set_empty() {
        let s = anchor_set_new();
        assert_eq!(anchor_count(&s), 0);
    }

    #[test]
    fn test_add_anchor() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [1.0,0.0,0.0], 1.0));
        assert_eq!(anchor_count(&s), 1);
    }

    #[test]
    fn test_remove_anchor() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [0.0,0.0,0.0], 1.0));
        remove_anchor(&mut s, 0);
        assert_eq!(anchor_count(&s), 0);
    }

    #[test]
    fn test_get_anchor() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(2, [3.0,0.0,0.0], 0.5));
        let a = get_anchor(&s, 0).expect("should succeed");
        assert_eq!(a.vertex_index, 2);
    }

    #[test]
    fn test_anchor_influence_at_zero_dist() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [1.0,0.0,0.0], 1.0));
        let inf = anchor_influence(&s, 0, [1.0,0.0,0.0]);
        assert!((inf - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_anchors_full_weight() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [5.0,0.0,0.0], 1.0));
        let pos = vec![[0.0f32,0.0,0.0]];
        let r = apply_anchors(&pos, &s);
        assert!((r[0][0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_anchors_zero_weight() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [5.0,0.0,0.0], 0.0));
        let pos = vec![[1.0f32,0.0,0.0]];
        let r = apply_anchors(&pos, &s);
        assert!((r[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_anchors_out_of_range() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(10, [5.0,0.0,0.0], 1.0));
        let pos = vec![[1.0f32,0.0,0.0]];
        let r = apply_anchors(&pos, &s);
        assert!((r[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_anchor_influence_oob() {
        let s = anchor_set_new();
        let inf = anchor_influence(&s, 99, [0.0,0.0,0.0]);
        assert!((inf).abs() < 1e-5);
    }

    #[test]
    fn test_anchor_influence_decreases_with_distance() {
        let mut s = anchor_set_new();
        add_anchor(&mut s, new_anchor(0, [0.0,0.0,0.0], 1.0));
        let i1 = anchor_influence(&s, 0, [1.0,0.0,0.0]);
        let i2 = anchor_influence(&s, 0, [10.0,0.0,0.0]);
        assert!(i1 > i2);
    }
}
