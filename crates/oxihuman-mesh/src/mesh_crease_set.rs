// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Crease set management for subdivision surfaces.

/// A named crease set grouping edges by sharpness level.
pub struct CreaseSet {
    pub name: String,
    pub edge_pairs: Vec<(u32, u32)>,
    pub sharpness: f32,
}

/// Collection of crease sets for a mesh.
pub struct CreaseSetCollection {
    pub sets: Vec<CreaseSet>,
}

/// Create a new empty crease set collection.
pub fn new_crease_set_collection() -> CreaseSetCollection {
    CreaseSetCollection { sets: Vec::new() }
}

/// Add a named crease set with given sharpness.
pub fn add_crease_set(col: &mut CreaseSetCollection, name: &str, sharpness: f32) {
    col.sets.push(CreaseSet {
        name: name.to_string(),
        edge_pairs: Vec::new(),
        sharpness: sharpness.clamp(0.0, 10.0),
    });
}

/// Add an edge pair to the named crease set. Returns false if set not found.
pub fn add_edge_to_crease_set(col: &mut CreaseSetCollection, name: &str, v0: u32, v1: u32) -> bool {
    if let Some(set) = col.sets.iter_mut().find(|s| s.name == name) {
        set.edge_pairs.push((v0.min(v1), v0.max(v1)));
        true
    } else {
        false
    }
}

/// Total number of crease edges across all sets.
pub fn total_crease_edges(col: &CreaseSetCollection) -> usize {
    col.sets.iter().map(|s| s.edge_pairs.len()).sum()
}

/// Find a crease set by name.
pub fn find_crease_set<'a>(col: &'a CreaseSetCollection, name: &str) -> Option<&'a CreaseSet> {
    col.sets.iter().find(|s| s.name == name)
}

/// Validate that all sharpness values are in [0, 10].
pub fn validate_crease_sets(col: &CreaseSetCollection) -> bool {
    col.sets.iter().all(|s| (0.0..=10.0).contains(&s.sharpness))
}

/// Merge two crease set collections into one.
pub fn merge_crease_set_collections(
    a: CreaseSetCollection,
    mut b: CreaseSetCollection,
) -> CreaseSetCollection {
    let mut result = a;
    result.sets.append(&mut b.sets);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_collection_is_empty() {
        let col = new_crease_set_collection();
        assert_eq!(col.sets.len(), 0 /* empty */);
    }

    #[test]
    fn add_crease_set_increments_count() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "sharp_edges", 2.0);
        assert_eq!(col.sets.len(), 1 /* one set added */);
    }

    #[test]
    fn add_edge_to_existing_set() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "rim", 1.5);
        let ok = add_edge_to_crease_set(&mut col, "rim", 3, 7);
        assert!(ok /* edge added */);
        assert_eq!(col.sets[0].edge_pairs.len(), 1 /* one edge */);
    }

    #[test]
    fn add_edge_to_missing_set_returns_false() {
        let mut col = new_crease_set_collection();
        let ok = add_edge_to_crease_set(&mut col, "missing", 0, 1);
        assert!(!ok /* not found */);
    }

    #[test]
    fn total_crease_edges_sums_all_sets() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "a", 1.0);
        add_crease_set(&mut col, "b", 2.0);
        add_edge_to_crease_set(&mut col, "a", 0, 1);
        add_edge_to_crease_set(&mut col, "b", 2, 3);
        add_edge_to_crease_set(&mut col, "b", 4, 5);
        assert_eq!(total_crease_edges(&col), 3 /* three total */);
    }

    #[test]
    fn find_returns_correct_set() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "found", 3.0);
        let s = find_crease_set(&col, "found");
        assert!(s.is_some() /* found */);
        assert!((s.unwrap().sharpness - 3.0).abs() < 1e-6 /* sharpness matches */);
    }

    #[test]
    fn find_missing_returns_none() {
        let col = new_crease_set_collection();
        assert!(find_crease_set(&col, "nope").is_none() /* none */);
    }

    #[test]
    fn validate_passes_valid_sharpness() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "x", 5.0);
        assert!(validate_crease_sets(&col) /* valid */);
    }

    #[test]
    fn merge_combines_sets() {
        let mut a = new_crease_set_collection();
        let mut b = new_crease_set_collection();
        add_crease_set(&mut a, "a1", 1.0);
        add_crease_set(&mut b, "b1", 2.0);
        let merged = merge_crease_set_collections(a, b);
        assert_eq!(merged.sets.len(), 2 /* two sets after merge */);
    }

    #[test]
    fn edge_pair_normalised_order() {
        let mut col = new_crease_set_collection();
        add_crease_set(&mut col, "norm", 1.0);
        add_edge_to_crease_set(&mut col, "norm", 9, 2);
        let pair = col.sets[0].edge_pairs[0];
        assert!(pair.0 <= pair.1 /* min first */);
    }
}
