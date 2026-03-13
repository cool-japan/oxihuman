// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct ProximityDeformer {
    pub source_positions: Vec<[f32; 3]>,
    pub influence_radius: f32,
    pub strength: f32,
}

pub fn new_proximity_deformer(source: Vec<[f32; 3]>, radius: f32) -> ProximityDeformer {
    ProximityDeformer {
        source_positions: source,
        influence_radius: radius,
        strength: 1.0,
    }
}

pub fn proximity_nearest_distance(d: &ProximityDeformer, p: [f32; 3]) -> f32 {
    d.source_positions
        .iter()
        .map(|s| ((p[0] - s[0]).powi(2) + (p[1] - s[1]).powi(2) + (p[2] - s[2]).powi(2)).sqrt())
        .fold(f32::MAX, f32::min)
}

pub fn proximity_influence(d: &ProximityDeformer, point: [f32; 3]) -> f32 {
    if d.influence_radius <= 0.0 {
        return 0.0;
    }
    let dist = proximity_nearest_distance(d, point);
    (1.0 - dist / d.influence_radius).clamp(0.0, 1.0)
}

pub fn proximity_deform_vertex(
    d: &ProximityDeformer,
    p: [f32; 3],
    target_normal: [f32; 3],
) -> [f32; 3] {
    let infl = proximity_influence(d, p);
    let amount = infl * d.strength;
    [
        p[0] + target_normal[0] * amount,
        p[1] + target_normal[1] * amount,
        p[2] + target_normal[2] * amount,
    ]
}

pub fn proximity_count_influenced(d: &ProximityDeformer, points: &[[f32; 3]]) -> usize {
    points
        .iter()
        .filter(|&&p| proximity_influence(d, p) > 0.0)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_proximity_deformer() {
        /* stores source positions and radius */
        let src = vec![[0.0f32, 0.0, 0.0]];
        let d = new_proximity_deformer(src, 2.0);
        assert!((d.influence_radius - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_proximity_nearest_distance_zero() {
        /* distance to coincident source is zero */
        let d = new_proximity_deformer(vec![[1.0f32, 0.0, 0.0]], 1.0);
        assert!(proximity_nearest_distance(&d, [1.0, 0.0, 0.0]) < 1e-6);
    }

    #[test]
    fn test_proximity_influence_at_source() {
        /* at source point influence is 1 */
        let d = new_proximity_deformer(vec![[0.0f32, 0.0, 0.0]], 2.0);
        assert!((proximity_influence(&d, [0.0, 0.0, 0.0]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_proximity_influence_beyond_radius() {
        /* beyond radius influence is 0 */
        let d = new_proximity_deformer(vec![[0.0f32, 0.0, 0.0]], 1.0);
        assert!(proximity_influence(&d, [5.0, 0.0, 0.0]) < 1e-6);
    }

    #[test]
    fn test_proximity_deform_vertex() {
        /* deforms vertex along normal by influence*strength */
        let d = new_proximity_deformer(vec![[0.0f32, 0.0, 0.0]], 2.0);
        let out = proximity_deform_vertex(&d, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(out[1] > 0.0);
    }

    #[test]
    fn test_proximity_count_influenced() {
        /* points within radius are counted */
        let d = new_proximity_deformer(vec![[0.0f32, 0.0, 0.0]], 2.0);
        let pts = vec![[0.5f32, 0.0, 0.0], [5.0, 0.0, 0.0]];
        assert_eq!(proximity_count_influenced(&d, &pts), 1);
    }
}
