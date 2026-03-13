// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Feature-preserving smoothing: protect sharp edges during smoothing.

#[allow(dead_code)]
pub struct FeaturePreserver {
    pub crease_angle: f32,
    pub protected_edges: Vec<[u32; 2]>,
}

#[allow(dead_code)]
pub fn new_feature_preserver(crease_angle: f32) -> FeaturePreserver {
    FeaturePreserver { crease_angle, protected_edges: Vec::new() }
}

#[allow(dead_code)]
pub fn fp_add_protected_edge(fp: &mut FeaturePreserver, a: u32, b: u32) {
    fp.protected_edges.push([a, b]);
}

#[allow(dead_code)]
pub fn fp_edge_count(fp: &FeaturePreserver) -> usize {
    fp.protected_edges.len()
}

#[allow(dead_code)]
pub fn fp_is_protected(fp: &FeaturePreserver, a: u32, b: u32) -> bool {
    fp.protected_edges.iter().any(|e| (e[0] == a && e[1] == b) || (e[0] == b && e[1] == a))
}

#[allow(dead_code)]
pub fn fp_clear_edges(fp: &mut FeaturePreserver) {
    fp.protected_edges.clear();
}

#[allow(dead_code)]
pub fn fp_crease_angle(fp: &FeaturePreserver) -> f32 {
    fp.crease_angle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let fp = new_feature_preserver(45.0);
        assert_eq!(fp_edge_count(&fp), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut fp = new_feature_preserver(30.0);
        fp_add_protected_edge(&mut fp, 0, 1);
        assert_eq!(fp_edge_count(&fp), 1);
    }

    #[test]
    fn test_is_protected_forward() {
        let mut fp = new_feature_preserver(30.0);
        fp_add_protected_edge(&mut fp, 2, 3);
        assert!(fp_is_protected(&fp, 2, 3));
    }

    #[test]
    fn test_is_protected_reverse() {
        let mut fp = new_feature_preserver(30.0);
        fp_add_protected_edge(&mut fp, 2, 3);
        assert!(fp_is_protected(&fp, 3, 2));
    }

    #[test]
    fn test_is_not_protected() {
        let mut fp = new_feature_preserver(30.0);
        fp_add_protected_edge(&mut fp, 2, 3);
        assert!(!fp_is_protected(&fp, 0, 1));
    }

    #[test]
    fn test_clear_edges() {
        let mut fp = new_feature_preserver(30.0);
        fp_add_protected_edge(&mut fp, 0, 1);
        fp_add_protected_edge(&mut fp, 2, 3);
        fp_clear_edges(&mut fp);
        assert_eq!(fp_edge_count(&fp), 0);
    }

    #[test]
    fn test_crease_angle() {
        let fp = new_feature_preserver(60.0);
        assert!((fp_crease_angle(&fp) - 60.0).abs() < 1e-5);
    }

    #[test]
    fn test_multiple_edges() {
        let mut fp = new_feature_preserver(45.0);
        for i in 0..5u32 {
            fp_add_protected_edge(&mut fp, i, i + 1);
        }
        assert_eq!(fp_edge_count(&fp), 5);
    }
}
