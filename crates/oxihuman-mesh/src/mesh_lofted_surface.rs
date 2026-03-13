// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A lofted surface built from a sequence of profile curves.
#[allow(dead_code)]
pub struct LoftedSurface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub profile_count: usize,
    pub profile_vertex_count: usize,
}

#[allow(dead_code)]
pub struct LoftConfig {
    pub closed_profiles: bool,
    pub smooth: bool,
}

#[allow(dead_code)]
pub fn default_loft_config() -> LoftConfig {
    LoftConfig {
        closed_profiles: false,
        smooth: true,
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Build a circle profile in XZ plane at height y with given radius.
#[allow(dead_code)]
pub fn circle_profile_at(y: f32, radius: f32, n: usize) -> Vec<[f32; 3]> {
    if n == 0 {
        return vec![];
    }
    (0..n)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / n as f32;
            [radius * angle.cos(), y, radius * angle.sin()]
        })
        .collect()
}

/// Loft a surface through ordered profile curves.
/// All profiles must have the same vertex count.
#[allow(dead_code)]
pub fn loft_surface(profiles: &[Vec<[f32; 3]>], _cfg: &LoftConfig) -> LoftedSurface {
    if profiles.is_empty() {
        return LoftedSurface {
            positions: vec![],
            indices: vec![],
            profile_count: 0,
            profile_vertex_count: 0,
        };
    }
    let pvc = profiles[0].len();
    if pvc == 0 {
        return LoftedSurface {
            positions: vec![],
            indices: vec![],
            profile_count: profiles.len(),
            profile_vertex_count: 0,
        };
    }

    let mut positions: Vec<[f32; 3]> = Vec::new();
    for prof in profiles {
        let take = prof.len().min(pvc);
        positions.extend_from_slice(&prof[..take]);
        for _ in take..pvc {
            positions.push([0.0; 3]);
        }
    }

    let mut indices: Vec<u32> = Vec::new();
    let pc = profiles.len();
    #[allow(clippy::needless_range_loop)]
    for pi in 0..pc.saturating_sub(1) {
        for vi in 0..pvc {
            let next_vi = (vi + 1) % pvc;
            let a = (pi * pvc + vi) as u32;
            let b = (pi * pvc + next_vi) as u32;
            let c = ((pi + 1) * pvc + next_vi) as u32;
            let d = ((pi + 1) * pvc + vi) as u32;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    LoftedSurface {
        positions,
        indices,
        profile_count: pc,
        profile_vertex_count: pvc,
    }
}

#[allow(dead_code)]
pub fn loft_vertex_count(surf: &LoftedSurface) -> usize {
    surf.positions.len()
}

#[allow(dead_code)]
pub fn loft_face_count(surf: &LoftedSurface) -> usize {
    surf.indices.len() / 3
}

#[allow(dead_code)]
pub fn loft_bounds(surf: &LoftedSurface) -> ([f32; 3], [f32; 3]) {
    if surf.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = surf.positions[0];
    let mut mx = surf.positions[0];
    for p in &surf.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn loft_centroid(surf: &LoftedSurface) -> [f32; 3] {
    if surf.positions.is_empty() {
        return [0.0; 3];
    }
    let n = surf.positions.len() as f32;
    let mut s = [0.0f32; 3];
    for p in &surf.positions {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

#[allow(dead_code)]
pub fn interpolate_profiles(a: &[[f32; 3]], b: &[[f32; 3]], t: f32) -> Vec<[f32; 3]> {
    let len = a.len().min(b.len());
    (0..len).map(|i| lerp3(a[i], b[i], t)).collect()
}

#[allow(dead_code)]
pub fn loft_to_json(surf: &LoftedSurface) -> String {
    format!(
        "{{\"profile_count\":{},\"profile_vertex_count\":{},\"vertex_count\":{},\"face_count\":{}}}",
        surf.profile_count,
        surf.profile_vertex_count,
        surf.positions.len(),
        surf.indices.len() / 3
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_circles() -> Vec<Vec<[f32; 3]>> {
        vec![
            circle_profile_at(0.0, 1.0, 8),
            circle_profile_at(1.0, 1.0, 8),
        ]
    }

    #[test]
    fn test_loft_vertex_count() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        assert_eq!(s.positions.len(), 16);
    }

    #[test]
    fn test_loft_face_count() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        assert_eq!(loft_face_count(&s), 16);
    }

    #[test]
    fn test_loft_empty_profiles() {
        let cfg = default_loft_config();
        let s = loft_surface(&[], &cfg);
        assert_eq!(s.positions.len(), 0);
        assert_eq!(s.indices.len(), 0);
    }

    #[test]
    fn test_loft_single_profile_no_faces() {
        let cfg = default_loft_config();
        let profs = vec![circle_profile_at(0.0, 1.0, 6)];
        let s = loft_surface(&profs, &cfg);
        assert_eq!(s.indices.len(), 0);
    }

    #[test]
    fn test_loft_bounds_y() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        let (mn, mx) = loft_bounds(&s);
        assert!(mn[1] <= 0.0 + 1e-5);
        assert!(mx[1] >= 1.0 - 1e-5);
    }

    #[test]
    fn test_loft_centroid_height() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        let c = loft_centroid(&s);
        assert!((c[1] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_interpolate_profiles() {
        let a = circle_profile_at(0.0, 1.0, 4);
        let b = circle_profile_at(2.0, 1.0, 4);
        let mid = interpolate_profiles(&a, &b, 0.5);
        assert_eq!(mid.len(), 4);
        for p in &mid {
            assert!((p[1] - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn test_circle_profile_at_count() {
        let prof = circle_profile_at(0.0, 1.0, 12);
        assert_eq!(prof.len(), 12);
    }

    #[test]
    fn test_loft_to_json() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        let j = loft_to_json(&s);
        assert!(j.contains("profile_count"));
        assert!(j.contains("face_count"));
    }

    #[test]
    fn test_loft_vertex_count_fn() {
        let cfg = default_loft_config();
        let s = loft_surface(&two_circles(), &cfg);
        assert_eq!(loft_vertex_count(&s), s.positions.len());
    }
}
