// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

// --- new API (Wave 151B) ---

pub struct HelixParams {
    pub radius: f32,
    pub pitch: f32,
    pub turns: f32,
    pub segments_per_turn: u32,
}

pub fn new_helix(radius: f32, pitch: f32, turns: f32) -> HelixParams {
    HelixParams {
        radius,
        pitch,
        turns,
        segments_per_turn: 32,
    }
}

pub fn helix_point(p: &HelixParams, t: f32) -> [f32; 3] {
    let angle = t * p.turns * TAU;
    let x = p.radius * angle.cos();
    let y = p.radius * angle.sin();
    let z = t * p.turns * p.pitch;
    [x, y, z]
}

pub fn helix_length(p: &HelixParams) -> f32 {
    let circumference = TAU * p.radius;
    let turns = p.turns;
    ((circumference * turns).powi(2) + (p.pitch * turns).powi(2)).sqrt()
}

pub fn helix_to_polyline(p: &HelixParams) -> Vec<[f32; 3]> {
    let n = (p.segments_per_turn as usize) * (p.turns.ceil() as usize).max(1);
    (0..=n)
        .map(|i| helix_point(p, i as f32 / n as f32))
        .collect()
}

pub fn helix_point_count(p: &HelixParams) -> usize {
    let n = (p.segments_per_turn as usize) * (p.turns.ceil() as usize).max(1);
    n + 1
}

// --- legacy API stubs (previously expected by lib.rs) ---

pub struct HelixMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub fn build_helix_mesh(p: &HelixParams, tube_radius: f32, tube_segments: u32) -> HelixMesh {
    let _ = tube_radius;
    let _ = tube_segments;
    let pts = helix_to_polyline(p);
    HelixMesh {
        positions: pts,
        indices: Vec::new(),
    }
}

pub fn helix_path(p: &HelixParams) -> Vec<[f32; 3]> {
    helix_to_polyline(p)
}

pub fn helix_triangle_count(p: &HelixParams, tube_segments: u32) -> usize {
    let segs = (p.segments_per_turn as usize) * (p.turns.ceil() as usize).max(1);
    segs * tube_segments as usize * 2
}

pub fn helix_vertex_count(p: &HelixParams, tube_segments: u32) -> usize {
    let segs = (p.segments_per_turn as usize) * (p.turns.ceil() as usize).max(1);
    (segs + 1) * tube_segments as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_helix_defaults() {
        /* segments_per_turn defaults to 32 */
        let h = new_helix(1.0, 0.5, 2.0);
        assert_eq!(h.segments_per_turn, 32);
    }

    #[test]
    fn test_helix_point_at_zero() {
        /* at t=0, z=0 */
        let h = new_helix(1.0, 1.0, 3.0);
        let pt = helix_point(&h, 0.0);
        assert!((pt[2]).abs() < 1e-5);
    }

    #[test]
    fn test_helix_point_at_one() {
        /* at t=1, z = turns*pitch */
        let h = new_helix(1.0, 2.0, 3.0);
        let pt = helix_point(&h, 1.0);
        assert!((pt[2] - 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_helix_length_positive() {
        /* length is positive */
        let h = new_helix(1.0, 1.0, 2.0);
        assert!(helix_length(&h) > 0.0);
    }

    #[test]
    fn test_helix_to_polyline_count() {
        /* point count matches helix_point_count */
        let h = new_helix(1.0, 1.0, 2.0);
        let pts = helix_to_polyline(&h);
        assert_eq!(pts.len(), helix_point_count(&h));
    }

    #[test]
    fn test_helix_radius_in_xy() {
        /* x^2+y^2 ~ r^2 */
        let h = new_helix(2.0, 0.5, 1.0);
        let pt = helix_point(&h, 0.25);
        let r_sq = pt[0] * pt[0] + pt[1] * pt[1];
        assert!((r_sq.sqrt() - 2.0).abs() < 1e-4);
    }
}
