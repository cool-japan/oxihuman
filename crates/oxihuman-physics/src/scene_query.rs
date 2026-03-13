#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Scene query system for spatial overlap tests.

/// Result of a scene query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub body_id: u32,
    pub distance: f32,
}

/// Scene query manager.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneQuery {
    bodies: Vec<(u32, [f32; 3], f32)>, // (id, center, radius)
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
pub fn new_scene_query() -> SceneQuery {
    SceneQuery { bodies: Vec::new() }
}

impl SceneQuery {
    #[allow(dead_code)]
    pub fn add_body(&mut self, id: u32, center: [f32; 3], radius: f32) {
        self.bodies.push((id, center, radius));
    }
}

#[allow(dead_code)]
pub fn overlap_sphere(sq: &SceneQuery, center: [f32; 3], radius: f32) -> Vec<QueryResult> {
    let mut results = Vec::new();
    for &(id, bc, br) in &sq.bodies {
        let d = vec3_len(vec3_sub(center, bc));
        if d <= radius + br {
            results.push(QueryResult { body_id: id, distance: (d - br).max(0.0) });
        }
    }
    results
}

#[allow(dead_code)]
pub fn overlap_box(sq: &SceneQuery, center: [f32; 3], half: [f32; 3]) -> Vec<QueryResult> {
    let mut results = Vec::new();
    for &(id, bc, br) in &sq.bodies {
        // Check if sphere overlaps AABB.
        let mut dist_sq = 0.0f32;
        for i in 0..3 {
            let lo = center[i] - half[i];
            let hi = center[i] + half[i];
            if bc[i] < lo {
                dist_sq += (lo - bc[i]) * (lo - bc[i]);
            } else if bc[i] > hi {
                dist_sq += (bc[i] - hi) * (bc[i] - hi);
            }
        }
        if dist_sq <= br * br {
            results.push(QueryResult { body_id: id, distance: dist_sq.sqrt() });
        }
    }
    results
}

#[allow(dead_code)]
pub fn query_closest(sq: &SceneQuery, point: [f32; 3]) -> Option<QueryResult> {
    sq.bodies.iter().map(|&(id, bc, br)| {
        let d = (vec3_len(vec3_sub(point, bc)) - br).max(0.0);
        QueryResult { body_id: id, distance: d }
    }).min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
}

#[allow(dead_code)]
pub fn query_all_in_aabb(
    sq: &SceneQuery,
    min: [f32; 3], max: [f32; 3],
) -> Vec<QueryResult> {
    let center = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let half = [
        (max[0] - min[0]) * 0.5,
        (max[1] - min[1]) * 0.5,
        (max[2] - min[2]) * 0.5,
    ];
    overlap_box(sq, center, half)
}

#[allow(dead_code)]
pub fn query_result_count(results: &[QueryResult]) -> usize {
    results.len()
}

#[allow(dead_code)]
pub fn query_first(results: &[QueryResult]) -> Option<&QueryResult> {
    results.first()
}

#[allow(dead_code)]
pub fn query_is_empty(results: &[QueryResult]) -> bool {
    results.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene() -> SceneQuery {
        let mut sq = new_scene_query();
        sq.add_body(1, [0.0, 0.0, 0.0], 1.0);
        sq.add_body(2, [5.0, 0.0, 0.0], 1.0);
        sq.add_body(3, [10.0, 0.0, 0.0], 1.0);
        sq
    }

    #[test]
    fn test_overlap_sphere_hit() {
        let sq = make_scene();
        let r = overlap_sphere(&sq, [0.0, 0.0, 0.0], 2.0);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_overlap_sphere_miss() {
        let sq = make_scene();
        let r = overlap_sphere(&sq, [100.0, 0.0, 0.0], 0.1);
        assert!(r.is_empty());
    }

    #[test]
    fn test_overlap_box() {
        let sq = make_scene();
        let r = overlap_box(&sq, [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_query_closest() {
        let sq = make_scene();
        let r = query_closest(&sq, [0.5, 0.0, 0.0]).expect("should succeed");
        assert_eq!(r.body_id, 1);
    }

    #[test]
    fn test_query_all_in_aabb() {
        let sq = make_scene();
        let r = query_all_in_aabb(&sq, [-2.0, -2.0, -2.0], [2.0, 2.0, 2.0]);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_result_count() {
        let r = vec![
            QueryResult { body_id: 1, distance: 0.0 },
            QueryResult { body_id: 2, distance: 1.0 },
        ];
        assert_eq!(query_result_count(&r), 2);
    }

    #[test]
    fn test_query_first() {
        let r = vec![QueryResult { body_id: 5, distance: 0.0 }];
        assert_eq!(query_first(&r).expect("should succeed").body_id, 5);
    }

    #[test]
    fn test_query_is_empty() {
        let r: Vec<QueryResult> = vec![];
        assert!(query_is_empty(&r));
    }

    #[test]
    fn test_empty_scene() {
        let sq = new_scene_query();
        let r = overlap_sphere(&sq, [0.0, 0.0, 0.0], 100.0);
        assert!(r.is_empty());
    }

    #[test]
    fn test_closest_empty() {
        let sq = new_scene_query();
        assert!(query_closest(&sq, [0.0, 0.0, 0.0]).is_none());
    }
}
