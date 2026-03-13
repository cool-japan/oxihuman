#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Shrinkwrap: project each vertex onto nearest point on target mesh.

#[allow(dead_code)]
pub fn nearest_vert(pos: [f32; 3], target: &[[f32; 3]]) -> [f32; 3] {
    if target.is_empty() {
        return pos;
    }
    let mut best = target[0];
    let mut best_dist = f32::MAX;
    for t in target {
        let dx = pos[0] - t[0];
        let dy = pos[1] - t[1];
        let dz = pos[2] - t[2];
        let d = dx * dx + dy * dy + dz * dz;
        if d < best_dist {
            best_dist = d;
            best = *t;
        }
    }
    best
}

#[allow(dead_code)]
pub fn shrinkwrap_project(src_verts: &[[f32; 3]], target_verts: &[[f32; 3]]) -> Vec<[f32; 3]> {
    src_verts.iter().map(|v| nearest_vert(*v, target_verts)).collect()
}

#[allow(dead_code)]
pub fn shrinkwrap_dist(src: &[[f32; 3]], target: &[[f32; 3]]) -> f32 {
    if src.is_empty() || target.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    for v in src {
        let n = nearest_vert(*v, target);
        let dx = v[0] - n[0];
        let dy = v[1] - n[1];
        let dz = v[2] - n[2];
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total / src.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
    }

    #[test]
    fn nearest_vert_finds_closest() {
        let n = nearest_vert([0.1, 0.0, 0.0], &target());
        assert!((n[0]).abs() < 1e-6);
    }

    #[test]
    fn nearest_vert_empty_returns_pos() {
        let v = [3.0, 3.0, 3.0];
        let n = nearest_vert(v, &[]);
        assert!((n[0] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_vert_exact_match() {
        let n = nearest_vert([1.0, 0.0, 0.0], &target());
        assert!((n[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn shrinkwrap_project_count_preserved() {
        let src = vec![[0.5, 0.5, 0.0], [1.5, 0.5, 0.0]];
        let out = shrinkwrap_project(&src, &target());
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn shrinkwrap_project_projects_to_target() {
        let src = vec![[0.1, 5.0, 0.0]];
        let out = shrinkwrap_project(&src, &target());
        // nearest target is [0,0,0]
        assert!((out[0][0]).abs() < 1e-6);
    }

    #[test]
    fn shrinkwrap_dist_zero_when_on_target() {
        let pts = target();
        let d = shrinkwrap_dist(&pts, &pts);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn shrinkwrap_dist_positive_when_off() {
        let src = vec![[0.0, 1.0, 0.0]];
        let d = shrinkwrap_dist(&src, &target());
        assert!(d > 0.0);
    }

    #[test]
    fn shrinkwrap_dist_empty_src_returns_zero() {
        let d = shrinkwrap_dist(&[], &target());
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn shrinkwrap_project_empty_target_returns_src() {
        let src = vec![[1.0, 2.0, 3.0]];
        let out = shrinkwrap_project(&src, &[]);
        assert!((out[0][0] - 1.0).abs() < 1e-6);
    }
}
