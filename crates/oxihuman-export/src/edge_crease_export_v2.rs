// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge crease export v2: per-edge crease sharpness for subdivision.

/// Edge key (min,max vertex pair).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CreaseEdgeKey {
    pub a: u32,
    pub b: u32,
}

/// A crease entry with sharpness.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CreaseEntryV2 {
    pub edge: CreaseEdgeKey,
    pub sharpness: f32,
}

/// Edge crease export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCreaseExportV2 {
    pub creases: Vec<CreaseEntryV2>,
}

/// Make a canonical edge key.
#[allow(dead_code)]
pub fn make_crease_key(a: u32, b: u32) -> CreaseEdgeKey {
    CreaseEdgeKey {
        a: a.min(b),
        b: a.max(b),
    }
}

/// Create a new edge crease export.
#[allow(dead_code)]
pub fn new_edge_crease_export_v2() -> EdgeCreaseExportV2 {
    EdgeCreaseExportV2 {
        creases: Vec::new(),
    }
}

/// Add a crease.
#[allow(dead_code)]
pub fn add_crease_v2(exp: &mut EdgeCreaseExportV2, a: u32, b: u32, sharpness: f32) {
    let key = make_crease_key(a, b);
    let sharpness = sharpness.clamp(0.0, 10.0);
    if let Some(entry) = exp.creases.iter_mut().find(|c| c.edge == key) {
        entry.sharpness = sharpness;
    } else {
        exp.creases.push(CreaseEntryV2 {
            edge: key,
            sharpness,
        });
    }
}

/// Crease count.
#[allow(dead_code)]
pub fn crease_count_v2(exp: &EdgeCreaseExportV2) -> usize {
    exp.creases.len()
}

/// Get sharpness for an edge.
#[allow(dead_code)]
pub fn get_sharpness_v2(exp: &EdgeCreaseExportV2, a: u32, b: u32) -> Option<f32> {
    let key = make_crease_key(a, b);
    exp.creases
        .iter()
        .find(|c| c.edge == key)
        .map(|c| c.sharpness)
}

/// Maximum sharpness.
#[allow(dead_code)]
pub fn max_sharpness_v2(exp: &EdgeCreaseExportV2) -> f32 {
    exp.creases
        .iter()
        .map(|c| c.sharpness)
        .fold(0.0_f32, f32::max)
}

/// Validate: sharpness in `[0,10]`.
#[allow(dead_code)]
pub fn validate_creases_v2(exp: &EdgeCreaseExportV2) -> bool {
    exp.creases
        .iter()
        .all(|c| (0.0..=10.0).contains(&c.sharpness))
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn edge_crease_v2_to_json(exp: &EdgeCreaseExportV2) -> String {
    format!(
        "{{\"crease_count\":{},\"max_sharpness\":{}}}",
        crease_count_v2(exp),
        max_sharpness_v2(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let exp = new_edge_crease_export_v2();
        assert_eq!(crease_count_v2(&exp), 0);
    }

    #[test]
    fn add_crease_increments() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 0, 1, 5.0);
        assert_eq!(crease_count_v2(&exp), 1);
    }

    #[test]
    fn duplicate_edge_updates() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 0, 1, 2.0);
        add_crease_v2(&mut exp, 1, 0, 8.0);
        assert_eq!(crease_count_v2(&exp), 1);
        assert!((get_sharpness_v2(&exp, 0, 1).expect("should succeed") - 8.0).abs() < 1e-5);
    }

    #[test]
    fn canonical_key_order() {
        let k = make_crease_key(5, 2);
        assert!(k.a <= k.b);
    }

    #[test]
    fn get_existing() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 3, 7, 4.5);
        assert!((get_sharpness_v2(&exp, 3, 7).expect("should succeed") - 4.5).abs() < 1e-5);
    }

    #[test]
    fn get_missing_none() {
        let exp = new_edge_crease_export_v2();
        assert!(get_sharpness_v2(&exp, 0, 1).is_none());
    }

    #[test]
    fn max_sharpness_correct() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 0, 1, 3.0);
        add_crease_v2(&mut exp, 2, 3, 9.0);
        assert!((max_sharpness_v2(&exp) - 9.0).abs() < 1e-5);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 0, 1, 5.0);
        assert!(validate_creases_v2(&exp));
    }

    #[test]
    fn json_contains_crease_count() {
        let exp = new_edge_crease_export_v2();
        let j = edge_crease_v2_to_json(&exp);
        assert!(j.contains("crease_count"));
    }

    #[test]
    fn sharpness_clamped() {
        let mut exp = new_edge_crease_export_v2();
        add_crease_v2(&mut exp, 0, 1, 999.0);
        assert!((get_sharpness_v2(&exp, 0, 1).expect("should succeed") - 10.0).abs() < 1e-5);
    }
}
