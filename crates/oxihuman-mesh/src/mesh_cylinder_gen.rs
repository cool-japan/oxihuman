// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

// --- new API (Wave 151B) ---

pub struct CylinderParams {
    pub radius_top: f32,
    pub radius_bottom: f32,
    pub height: f32,
    pub segments: u32,
    pub cap_top: bool,
    pub cap_bottom: bool,
}

pub fn new_cylinder(radius: f32, height: f32, segments: u32) -> CylinderParams {
    CylinderParams {
        radius_top: radius,
        radius_bottom: radius,
        height,
        segments,
        cap_top: true,
        cap_bottom: true,
    }
}

pub fn cylinder_vertex(p: &CylinderParams, ring: u32, seg: u32) -> [f32; 3] {
    let angle = (seg as f32 / p.segments as f32) * TAU;
    let t = ring as f32;
    let r = p.radius_bottom + t * (p.radius_top - p.radius_bottom);
    let x = r * angle.cos();
    let y = r * angle.sin();
    let z = t * p.height;
    [x, y, z]
}

pub fn cylinder_vertex_count(p: &CylinderParams) -> usize {
    let side = 2 * p.segments as usize;
    let caps = if p.cap_top { 1 } else { 0 } + if p.cap_bottom { 1 } else { 0 };
    side + caps
}

pub fn cylinder_face_count(p: &CylinderParams) -> usize {
    let side = p.segments as usize * 2;
    let cap_top = if p.cap_top { p.segments as usize } else { 0 };
    let cap_bot = if p.cap_bottom { p.segments as usize } else { 0 };
    side + cap_top + cap_bot
}

pub fn cylinder_volume(p: &CylinderParams) -> f32 {
    PI * p.height / 3.0
        * (p.radius_bottom * p.radius_bottom
            + p.radius_bottom * p.radius_top
            + p.radius_top * p.radius_top)
}

pub fn cylinder_is_cone(p: &CylinderParams) -> bool {
    p.radius_top < 1e-6
}

// --- legacy API stubs (previously expected by lib.rs) ---

pub struct CylinderGenConfig {
    pub radius: f32,
    pub height: f32,
    pub segments: u32,
}

pub struct CylinderGenResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub fn default_cylinder_gen_config() -> CylinderGenConfig {
    CylinderGenConfig {
        radius: 1.0,
        height: 2.0,
        segments: 16,
    }
}

pub fn generate_cylinder(cfg: &CylinderGenConfig) -> CylinderGenResult {
    let p = new_cylinder(cfg.radius, cfg.height, cfg.segments);
    let mut positions = Vec::new();
    for seg in 0..cfg.segments {
        positions.push(cylinder_vertex(&p, 0, seg));
        positions.push(cylinder_vertex(&p, 1, seg));
    }
    CylinderGenResult {
        positions,
        indices: Vec::new(),
    }
}

pub fn cylinder_gen_to_json(r: &CylinderGenResult) -> String {
    format!(r#"{{"vertex_count":{}}}"#, r.positions.len())
}

pub fn cylinder_indices_valid(r: &CylinderGenResult) -> bool {
    r.indices.iter().all(|&i| (i as usize) < r.positions.len())
}

pub fn cylinder_lateral_area(cfg: &CylinderGenConfig) -> f32 {
    TAU * cfg.radius * cfg.height
}

pub fn cylinder_side_index_count(cfg: &CylinderGenConfig) -> usize {
    cfg.segments as usize * 6
}

pub fn cylinder_side_vertex_count(cfg: &CylinderGenConfig) -> usize {
    (cfg.segments as usize + 1) * 2
}

pub fn cylinder_total_area(cfg: &CylinderGenConfig) -> f32 {
    TAU * cfg.radius * (cfg.height + cfg.radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cylinder() {
        /* both radii equal */
        let c = new_cylinder(1.0, 2.0, 16);
        assert!((c.radius_top - c.radius_bottom).abs() < 1e-6);
    }

    #[test]
    fn test_cylinder_vertex() {
        /* seg=0 gives x=r, y=0 */
        let c = new_cylinder(2.0, 1.0, 8);
        let v = cylinder_vertex(&c, 0, 0);
        assert!((v[0] - 2.0).abs() < 1e-5);
        assert!(v[1].abs() < 1e-5);
    }

    #[test]
    fn test_cylinder_volume_uniform() {
        /* volume = pi*r^2*h */
        let c = new_cylinder(1.0, 2.0, 16);
        let vol = cylinder_volume(&c);
        assert!((vol - PI * 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_cylinder_is_cone_false() {
        /* uniform cylinder is not cone */
        let c = new_cylinder(1.0, 1.0, 8);
        assert!(!cylinder_is_cone(&c));
    }

    #[test]
    fn test_cylinder_is_cone_true() {
        /* top radius ~ 0 is cone */
        let c = CylinderParams {
            radius_top: 0.0,
            radius_bottom: 1.0,
            height: 1.0,
            segments: 8,
            cap_top: false,
            cap_bottom: true,
        };
        assert!(cylinder_is_cone(&c));
    }

    #[test]
    fn test_cylinder_face_count() {
        /* basic count */
        let c = new_cylinder(1.0, 1.0, 8);
        let fc = cylinder_face_count(&c);
        assert!(fc > 0);
    }

    #[test]
    fn test_cylinder_vertex_count() {
        /* vertex count > 0 */
        let c = new_cylinder(1.0, 1.0, 8);
        let vc = cylinder_vertex_count(&c);
        assert!(vc > 0);
    }
}
