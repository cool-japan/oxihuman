// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cylinder cap generation: flat and dome caps for open cylinders.

use std::f32::consts::PI;

/// Type of cylinder cap.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapType {
    Flat,
    Dome,
}

/// Cap mesh output.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CylinderCap {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub cap_type: CapType,
}

/// Generate a flat circular cap.
#[allow(dead_code)]
pub fn flat_cap(radius: f32, segments: usize, y: f32, flip: bool) -> CylinderCap {
    let segs = segments.max(3);
    let mut positions = Vec::with_capacity(segs + 1);
    let mut indices = Vec::with_capacity(segs * 3);
    positions.push([0.0, y, 0.0]);
    for i in 0..segs {
        let theta = 2.0 * PI * (i as f32) / (segs as f32);
        positions.push([radius * theta.cos(), y, radius * theta.sin()]);
    }
    for i in 0..segs {
        let a = (i + 1) as u32;
        let b = ((i + 1) % segs + 1) as u32;
        if flip {
            indices.extend_from_slice(&[0, b, a]);
        } else {
            indices.extend_from_slice(&[0, a, b]);
        }
    }
    CylinderCap {
        positions,
        indices,
        cap_type: CapType::Flat,
    }
}

/// Generate a dome cap (hemisphere).
#[allow(dead_code)]
pub fn dome_cap(
    radius: f32,
    segments: usize,
    rings: usize,
    y_base: f32,
    flip: bool,
) -> CylinderCap {
    let segs = segments.max(3);
    let rings = rings.max(1);
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for r in 0..=rings {
        let phi = PI * 0.5 * (r as f32) / (rings as f32);
        let yr = y_base + radius * phi.sin();
        let xr = radius * phi.cos();
        for s in 0..segs {
            let theta = 2.0 * PI * (s as f32) / (segs as f32);
            positions.push([xr * theta.cos(), yr, xr * theta.sin()]);
        }
    }
    for r in 0..rings {
        for s in 0..segs {
            let a = (r * segs + s) as u32;
            let b = (r * segs + (s + 1) % segs) as u32;
            let c = ((r + 1) * segs + s) as u32;
            let d = ((r + 1) * segs + (s + 1) % segs) as u32;
            if flip {
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            } else {
                indices.extend_from_slice(&[a, b, c, b, d, c]);
            }
        }
    }
    CylinderCap {
        positions,
        indices,
        cap_type: CapType::Dome,
    }
}

/// Vertex count of a cap.
#[allow(dead_code)]
pub fn cap_vertex_count(cap: &CylinderCap) -> usize {
    cap.positions.len()
}

/// Triangle count of a cap.
#[allow(dead_code)]
pub fn cap_triangle_count(cap: &CylinderCap) -> usize {
    cap.indices.len() / 3
}

/// Whether a cap is flat.
#[allow(dead_code)]
pub fn is_flat_cap(cap: &CylinderCap) -> bool {
    cap.cap_type == CapType::Flat
}

/// Serialise to JSON stub.
#[allow(dead_code)]
pub fn cap_to_json(cap: &CylinderCap) -> String {
    format!(
        "{{\"vertices\":{},\"triangles\":{}}}",
        cap_vertex_count(cap),
        cap_triangle_count(cap)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_cap_vertex_count() {
        let cap = flat_cap(1.0, 8, 0.0, false);
        assert_eq!(cap_vertex_count(&cap), 9); // center + 8 rim
    }

    #[test]
    fn flat_cap_triangle_count() {
        let cap = flat_cap(1.0, 8, 0.0, false);
        assert_eq!(cap_triangle_count(&cap), 8);
    }

    #[test]
    fn flat_cap_is_flat() {
        let cap = flat_cap(1.0, 6, 0.0, false);
        assert!(is_flat_cap(&cap));
    }

    #[test]
    fn dome_cap_not_flat() {
        let cap = dome_cap(1.0, 8, 4, 0.0, false);
        assert!(!is_flat_cap(&cap));
    }

    #[test]
    fn dome_cap_has_vertices() {
        let cap = dome_cap(1.0, 8, 4, 0.0, false);
        assert!(cap_vertex_count(&cap) > 0);
    }

    #[test]
    fn flat_cap_center_at_y() {
        let cap = flat_cap(1.0, 6, 2.5, false);
        assert!((cap.positions[0][1] - 2.5).abs() < 1e-5);
    }

    #[test]
    fn flat_cap_flipped_has_same_count() {
        let a = flat_cap(1.0, 8, 0.0, false);
        let b = flat_cap(1.0, 8, 0.0, true);
        assert_eq!(cap_vertex_count(&a), cap_vertex_count(&b));
        assert_eq!(cap_triangle_count(&a), cap_triangle_count(&b));
    }

    #[test]
    fn json_contains_vertices() {
        let cap = flat_cap(1.0, 6, 0.0, false);
        let j = cap_to_json(&cap);
        assert!(j.contains("vertices"));
    }

    #[test]
    fn pi_from_consts() {
        let circumference = 2.0 * PI * 1.0_f32;
        assert!(circumference > 6.0);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }
}
