// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth self-collision proxy — simplified geometry used to detect cloth self-intersections.

/// A bounding sphere proxy for one cluster of cloth vertices.
#[derive(Debug, Clone)]
pub struct ClothColProxy {
    pub center: [f32; 3],
    pub radius: f32,
    pub vertex_indices: Vec<usize>,
}

/// Collection of cloth collision proxies.
#[derive(Debug, Default)]
pub struct ClothCollisionSet {
    proxies: Vec<ClothColProxy>,
    pub self_collision_margin: f32,
}

/// Create a new empty cloth collision set.
pub fn new_cloth_collision_set(margin: f32) -> ClothCollisionSet {
    ClothCollisionSet {
        proxies: Vec::new(),
        self_collision_margin: margin.max(0.0),
    }
}

/// Add a proxy sphere covering the given vertices.
pub fn add_proxy(set: &mut ClothCollisionSet, center: [f32; 3], radius: f32, vertices: Vec<usize>) {
    set.proxies.push(ClothColProxy {
        center,
        radius: radius.max(0.0),
        vertex_indices: vertices,
    });
}

/// Number of proxies.
pub fn proxy_count(set: &ClothCollisionSet) -> usize {
    set.proxies.len()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Check whether two proxies overlap (accounting for margin).
pub fn proxies_overlap(a: &ClothColProxy, b: &ClothColProxy, margin: f32) -> bool {
    let d = dist3(a.center, b.center);
    d < a.radius + b.radius + margin
}

/// Total vertices covered by all proxies.
pub fn total_covered_vertices(set: &ClothCollisionSet) -> usize {
    set.proxies.iter().map(|p| p.vertex_indices.len()).sum()
}

/// Average proxy radius.
pub fn average_proxy_radius(set: &ClothCollisionSet) -> f32 {
    if set.proxies.is_empty() {
        return 0.0;
    }
    let sum: f32 = set.proxies.iter().map(|p| p.radius).sum();
    sum / set.proxies.len() as f32
}

/// Serialize to JSON-style string.
pub fn cloth_collision_to_json(set: &ClothCollisionSet) -> String {
    format!(
        r#"{{"margin":{:.4}, "proxy_count":{}, "total_vertices":{}}}"#,
        set.self_collision_margin,
        proxy_count(set),
        total_covered_vertices(set)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_has_no_proxies() {
        /* fresh set has no proxies */
        let s = new_cloth_collision_set(0.01);
        assert_eq!(proxy_count(&s), 0);
    }

    #[test]
    fn add_proxy_increments_count() {
        /* adding a proxy yields count 1 */
        let mut s = new_cloth_collision_set(0.01);
        add_proxy(&mut s, [0.0; 3], 1.0, vec![0, 1, 2]);
        assert_eq!(proxy_count(&s), 1);
    }

    #[test]
    fn proxies_overlap_close_spheres() {
        /* two overlapping spheres should detect overlap */
        let a = ClothColProxy {
            center: [0.0; 3],
            radius: 1.0,
            vertex_indices: vec![],
        };
        let b = ClothColProxy {
            center: [1.5, 0.0, 0.0],
            radius: 1.0,
            vertex_indices: vec![],
        };
        assert!(proxies_overlap(&a, &b, 0.0));
    }

    #[test]
    fn proxies_no_overlap_far_spheres() {
        /* far spheres should not overlap */
        let a = ClothColProxy {
            center: [0.0; 3],
            radius: 0.5,
            vertex_indices: vec![],
        };
        let b = ClothColProxy {
            center: [10.0, 0.0, 0.0],
            radius: 0.5,
            vertex_indices: vec![],
        };
        assert!(!proxies_overlap(&a, &b, 0.0));
    }

    #[test]
    fn total_covered_vertices_sums() {
        /* total should be sum of all proxy vertex counts */
        let mut s = new_cloth_collision_set(0.01);
        add_proxy(&mut s, [0.0; 3], 1.0, vec![0, 1]);
        add_proxy(&mut s, [2.0, 0.0, 0.0], 1.0, vec![2, 3, 4]);
        assert_eq!(total_covered_vertices(&s), 5);
    }

    #[test]
    fn average_radius_empty_is_zero() {
        /* empty set average radius is zero */
        let s = new_cloth_collision_set(0.01);
        assert_eq!(average_proxy_radius(&s), 0.0);
    }

    #[test]
    fn average_radius_correct() {
        /* average of 2 and 4 is 3 */
        let mut s = new_cloth_collision_set(0.01);
        add_proxy(&mut s, [0.0; 3], 2.0, vec![]);
        add_proxy(&mut s, [0.0; 3], 4.0, vec![]);
        assert!((average_proxy_radius(&s) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn json_contains_margin() {
        /* JSON should contain margin field */
        let s = new_cloth_collision_set(0.05);
        assert!(cloth_collision_to_json(&s).contains("margin"));
    }

    #[test]
    fn negative_radius_clamped_to_zero() {
        /* negative radius should become 0 */
        let mut s = new_cloth_collision_set(0.01);
        add_proxy(&mut s, [0.0; 3], -1.0, vec![]);
        assert!(s.proxies[0].radius < 1e-8);
    }
}
