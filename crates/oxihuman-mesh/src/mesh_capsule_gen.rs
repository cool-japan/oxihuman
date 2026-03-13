// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::{PI, TAU};

// --- new API (Wave 151B) ---

pub struct CapsuleParams {
    pub radius: f32,
    pub height: f32,
    pub segments: u32,
    pub rings: u32,
}

pub fn new_capsule(radius: f32, height: f32, segments: u32, rings: u32) -> CapsuleParams {
    CapsuleParams {
        radius,
        height,
        segments,
        rings,
    }
}

pub fn capsule_vertex_count(p: &CapsuleParams) -> usize {
    let s = p.segments as usize;
    let r = p.rings as usize;
    s * (r + 1) + 2
}

pub fn capsule_face_count(p: &CapsuleParams) -> usize {
    let s = p.segments as usize;
    let r = p.rings as usize;
    s * r * 2
}

pub fn capsule_volume(p: &CapsuleParams) -> f32 {
    let r = p.radius;
    let h = p.height;
    PI * r * r * h + (4.0 / 3.0) * PI * r * r * r
}

pub fn capsule_surface_area(p: &CapsuleParams) -> f32 {
    let r = p.radius;
    let h = p.height;
    TAU * r * h + 4.0 * PI * r * r
}

// --- legacy API stubs (previously expected by lib.rs) ---

pub struct CapsuleGenConfig {
    pub radius: f32,
    pub height: f32,
    pub segments: u32,
    pub rings: u32,
}

pub struct CapsuleGenResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

pub fn default_capsule_gen_config() -> CapsuleGenConfig {
    CapsuleGenConfig {
        radius: 0.5,
        height: 1.0,
        segments: 16,
        rings: 8,
    }
}

pub fn generate_capsule(cfg: &CapsuleGenConfig) -> CapsuleGenResult {
    let p = new_capsule(cfg.radius, cfg.height, cfg.segments, cfg.rings);
    CapsuleGenResult {
        positions: vec![[0.0, 0.0, 0.0]; capsule_vertex_count(&p)],
        indices: Vec::new(),
    }
}

pub fn capsule_gen_to_json(r: &CapsuleGenResult) -> String {
    format!(r#"{{"vertex_count":{}}}"#, r.positions.len())
}

pub fn capsule_index_count(cfg: &CapsuleGenConfig) -> usize {
    let p = CapsuleParams {
        radius: cfg.radius,
        height: cfg.height,
        segments: cfg.segments,
        rings: cfg.rings,
    };
    capsule_face_count(&p) * 3
}

pub fn capsule_total_height(cfg: &CapsuleGenConfig) -> f32 {
    cfg.height + 2.0 * cfg.radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_capsule() {
        /* construction */
        let c = new_capsule(0.5, 1.0, 16, 8);
        assert!((c.radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_capsule_volume_sphere_only() {
        /* h=0 => sphere volume */
        let c = new_capsule(1.0, 0.0, 16, 8);
        let v = capsule_volume(&c);
        let sphere_vol = (4.0 / 3.0) * PI;
        assert!((v - sphere_vol).abs() < 1e-4);
    }

    #[test]
    fn test_capsule_surface_area_sphere() {
        /* h=0 => sphere surface area */
        let c = new_capsule(1.0, 0.0, 16, 8);
        let sa = capsule_surface_area(&c);
        assert!((sa - 4.0 * PI).abs() < 1e-4);
    }

    #[test]
    fn test_capsule_vertex_count() {
        /* > 0 */
        let c = new_capsule(0.5, 1.0, 8, 4);
        assert!(capsule_vertex_count(&c) > 0);
    }

    #[test]
    fn test_capsule_face_count() {
        /* > 0 */
        let c = new_capsule(0.5, 1.0, 8, 4);
        assert!(capsule_face_count(&c) > 0);
    }

    #[test]
    fn test_capsule_volume_increases_with_height() {
        /* more height = more volume */
        let c1 = new_capsule(0.5, 1.0, 8, 4);
        let c2 = new_capsule(0.5, 2.0, 8, 4);
        assert!(capsule_volume(&c2) > capsule_volume(&c1));
    }
}
