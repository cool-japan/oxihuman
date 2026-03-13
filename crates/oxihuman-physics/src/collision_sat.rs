#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Result of a Separating Axis Test.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SatResult {
    separating: bool,
    normal: [f32; 3],
    depth: f32,
    contact_count: u32,
}

#[allow(dead_code)]
pub fn sat_test_boxes(min_a: [f32; 3], max_a: [f32; 3], min_b: [f32; 3], max_b: [f32; 3]) -> SatResult {
    let mut min_overlap = f32::MAX;
    let mut best_axis = 0usize;

    for i in 0..3 {
        let overlap_start = min_a[i].max(min_b[i]);
        let overlap_end = max_a[i].min(max_b[i]);
        let overlap = overlap_end - overlap_start;
        if overlap < 0.0 {
            return SatResult {
                separating: true,
                normal: [0.0; 3],
                depth: 0.0,
                contact_count: 0,
            };
        }
        if overlap < min_overlap {
            min_overlap = overlap;
            best_axis = i;
        }
    }
    let mut normal = [0.0f32; 3];
    normal[best_axis] = 1.0;
    SatResult {
        separating: false,
        normal,
        depth: min_overlap,
        contact_count: 1,
    }
}

#[allow(dead_code)]
pub fn sat_test_convex(vertices_a: &[[f32; 3]], vertices_b: &[[f32; 3]]) -> SatResult {
    if vertices_a.is_empty() || vertices_b.is_empty() {
        return SatResult {
            separating: true,
            normal: [0.0; 3],
            depth: 0.0,
            contact_count: 0,
        };
    }
    // Simplified: check overlap on each axis
    let mut min_depth = f32::MAX;
    let mut best_normal = [1.0f32, 0.0, 0.0];
    for axis in 0..3 {
        let min_a = vertices_a.iter().map(|v| v[axis]).fold(f32::INFINITY, f32::min);
        let max_a = vertices_a.iter().map(|v| v[axis]).fold(f32::NEG_INFINITY, f32::max);
        let min_b = vertices_b.iter().map(|v| v[axis]).fold(f32::INFINITY, f32::min);
        let max_b = vertices_b.iter().map(|v| v[axis]).fold(f32::NEG_INFINITY, f32::max);
        let overlap = max_a.min(max_b) - min_a.max(min_b);
        if overlap < 0.0 {
            return SatResult {
                separating: true,
                normal: [0.0; 3],
                depth: 0.0,
                contact_count: 0,
            };
        }
        if overlap < min_depth {
            min_depth = overlap;
            best_normal = [0.0; 3];
            best_normal[axis] = 1.0;
        }
    }
    SatResult {
        separating: false,
        normal: best_normal,
        depth: min_depth,
        contact_count: 1,
    }
}

#[allow(dead_code)]
pub fn sat_overlap(result: &SatResult) -> bool {
    !result.separating
}

#[allow(dead_code)]
pub fn sat_min_penetration(result: &SatResult) -> f32 {
    result.depth
}

#[allow(dead_code)]
pub fn sat_normal(result: &SatResult) -> [f32; 3] {
    result.normal
}

#[allow(dead_code)]
pub fn sat_depth(result: &SatResult) -> f32 {
    result.depth
}

#[allow(dead_code)]
pub fn sat_contact_count(result: &SatResult) -> u32 {
    result.contact_count
}

#[allow(dead_code)]
pub fn sat_is_separating(result: &SatResult) -> bool {
    result.separating
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sat_boxes_overlap() {
        let r = sat_test_boxes([0.0; 3], [2.0; 3], [1.0; 3], [3.0; 3]);
        assert!(sat_overlap(&r));
    }

    #[test]
    fn test_sat_boxes_separate() {
        let r = sat_test_boxes([0.0; 3], [1.0; 3], [2.0; 3], [3.0; 3]);
        assert!(sat_is_separating(&r));
    }

    #[test]
    fn test_sat_depth() {
        let r = sat_test_boxes([0.0; 3], [2.0; 3], [1.0; 3], [3.0; 3]);
        assert!((sat_depth(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sat_normal() {
        let r = sat_test_boxes([0.0; 3], [2.0; 3], [1.0; 3], [3.0; 3]);
        let n = sat_normal(&r);
        assert!((n[0] - 1.0).abs() < 1e-6 || (n[1] - 1.0).abs() < 1e-6 || (n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sat_contact_count() {
        let r = sat_test_boxes([0.0; 3], [2.0; 3], [1.0; 3], [3.0; 3]);
        assert_eq!(sat_contact_count(&r), 1);
    }

    #[test]
    fn test_sat_is_separating() {
        let r = sat_test_boxes([0.0; 3], [1.0; 3], [5.0; 3], [6.0; 3]);
        assert!(sat_is_separating(&r));
    }

    #[test]
    fn test_sat_convex_overlap() {
        let a = [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]];
        let b = [[1.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 2.0, 0.0]];
        let r = sat_test_convex(&a, &b);
        assert!(sat_overlap(&r));
    }

    #[test]
    fn test_sat_convex_separate() {
        let a = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let b = [[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.5, 1.0, 0.0]];
        let r = sat_test_convex(&a, &b);
        assert!(sat_is_separating(&r));
    }

    #[test]
    fn test_sat_convex_empty() {
        let r = sat_test_convex(&[], &[]);
        assert!(sat_is_separating(&r));
    }

    #[test]
    fn test_sat_min_penetration() {
        let r = sat_test_boxes([0.0; 3], [2.0; 3], [1.5; 3], [3.0; 3]);
        assert!((sat_min_penetration(&r) - 0.5).abs() < 1e-6);
    }
}
