// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Decal normal-map blending — blend decal normals with surface normals.

use std::f32::consts::FRAC_PI_2;

/// Normal in tangent space [x, y, z], expected to be unit-length.
type Normal = [f32; 3];

/// Blend mode for normals.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalBlendMode {
    /// Overlay (Reoriented Normal Mapping).
    Rnm,
    /// Simple linear blend.
    Linear,
}

/// Decal normal entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalNormal {
    pub id: u32,
    pub normal: Normal,
    pub strength: f32,
    pub blend_mode: NormalBlendMode,
    pub enabled: bool,
}

/// Manager.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DecalNormalManager {
    decals: Vec<DecalNormal>,
}

#[allow(dead_code)]
pub fn new_decal_normal_manager() -> DecalNormalManager {
    DecalNormalManager::default()
}

#[allow(dead_code)]
pub fn dn_add(mgr: &mut DecalNormalManager, id: u32, normal: Normal, strength: f32) {
    let strength = strength.clamp(0.0, 1.0);
    mgr.decals.push(DecalNormal {
        id,
        normal: normalize3(normal),
        strength,
        blend_mode: NormalBlendMode::Rnm,
        enabled: true,
    });
}

#[allow(dead_code)]
pub fn dn_remove(mgr: &mut DecalNormalManager, id: u32) {
    mgr.decals.retain(|d| d.id != id);
}

#[allow(dead_code)]
pub fn dn_set_enabled(mgr: &mut DecalNormalManager, id: u32, enabled: bool) {
    for d in mgr.decals.iter_mut() {
        if d.id == id {
            d.enabled = enabled;
        }
    }
}

#[allow(dead_code)]
pub fn dn_count(mgr: &DecalNormalManager) -> usize {
    mgr.decals.len()
}

#[allow(dead_code)]
pub fn dn_enabled_count(mgr: &DecalNormalManager) -> usize {
    mgr.decals.iter().filter(|d| d.enabled).count()
}

#[allow(dead_code)]
pub fn dn_clear(mgr: &mut DecalNormalManager) {
    mgr.decals.clear();
}

/// Apply a decal normal onto a surface normal via RNM or linear blend.
#[allow(dead_code)]
pub fn dn_blend_normal(surface: Normal, decal: &DecalNormal) -> Normal {
    if !decal.enabled {
        return surface;
    }
    match decal.blend_mode {
        NormalBlendMode::Linear => {
            let t = decal.strength;
            let inv = 1.0 - t;
            normalize3([
                surface[0] * inv + decal.normal[0] * t,
                surface[1] * inv + decal.normal[1] * t,
                surface[2] * inv + decal.normal[2] * t,
            ])
        }
        NormalBlendMode::Rnm => {
            // Reoriented Normal Mapping
            let s = [surface[0], surface[1], surface[2] + 1.0];
            let d = [-decal.normal[0], -decal.normal[1], decal.normal[2]];
            let dot_sd = s[0] * d[0] + s[1] * d[1] + s[2] * d[2];
            let t = decal.strength;
            let blended = [
                surface[0] * (1.0 - t) + (s[0] * dot_sd - d[0]) * t,
                surface[1] * (1.0 - t) + (s[1] * dot_sd - d[1]) * t,
                surface[2] * (1.0 - t) + (s[2] * dot_sd - d[2]) * t,
            ];
            normalize3(blended)
        }
    }
}

/// Apply all enabled decals onto a surface normal sequentially.
#[allow(dead_code)]
pub fn dn_apply_all(mgr: &DecalNormalManager, surface: Normal) -> Normal {
    let mut n = surface;
    for d in &mgr.decals {
        n = dn_blend_normal(n, d);
    }
    n
}

/// Angle between two normals in radians.
#[allow(dead_code)]
pub fn dn_angle_rad(a: Normal, b: Normal) -> f32 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    dot.acos().min(FRAC_PI_2)
}

#[allow(dead_code)]
pub fn dn_to_json(mgr: &DecalNormalManager) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{}}}",
        mgr.decals.len(),
        dn_enabled_count(mgr)
    )
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manager() {
        let mgr = new_decal_normal_manager();
        assert_eq!(dn_count(&mgr), 0);
    }

    #[test]
    fn add_and_count() {
        let mut mgr = new_decal_normal_manager();
        dn_add(&mut mgr, 1, [0.0, 0.0, 1.0], 0.5);
        assert_eq!(dn_count(&mgr), 1);
    }

    #[test]
    fn remove_reduces_count() {
        let mut mgr = new_decal_normal_manager();
        dn_add(&mut mgr, 1, [0.0, 0.0, 1.0], 0.5);
        dn_remove(&mut mgr, 1);
        assert_eq!(dn_count(&mgr), 0);
    }

    #[test]
    fn disable_reduces_enabled() {
        let mut mgr = new_decal_normal_manager();
        dn_add(&mut mgr, 1, [0.0, 0.0, 1.0], 0.5);
        dn_set_enabled(&mut mgr, 1, false);
        assert_eq!(dn_enabled_count(&mgr), 0);
    }

    #[test]
    fn clear_empties_manager() {
        let mut mgr = new_decal_normal_manager();
        dn_add(&mut mgr, 1, [0.0, 0.0, 1.0], 0.5);
        dn_clear(&mut mgr);
        assert!(mgr.decals.is_empty());
    }

    #[test]
    fn blend_disabled_decal_unchanged() {
        let surface: Normal = [0.0, 0.0, 1.0];
        let d = DecalNormal {
            id: 1,
            normal: [1.0, 0.0, 0.0],
            strength: 1.0,
            blend_mode: NormalBlendMode::Linear,
            enabled: false,
        };
        let out = dn_blend_normal(surface, &d);
        assert!((out[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn linear_blend_at_zero_strength_unchanged() {
        let surface: Normal = [0.0, 0.0, 1.0];
        let d = DecalNormal {
            id: 1,
            normal: [1.0, 0.0, 0.0],
            strength: 0.0,
            blend_mode: NormalBlendMode::Linear,
            enabled: true,
        };
        let out = dn_blend_normal(surface, &d);
        assert!((out[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn angle_zero_same_normals() {
        let n: Normal = [0.0, 0.0, 1.0];
        assert!(dn_angle_rad(n, n) < 1e-5);
    }

    #[test]
    fn json_has_count() {
        let mgr = new_decal_normal_manager();
        assert!(dn_to_json(&mgr).contains("count"));
    }

    #[test]
    fn apply_all_with_no_decals_unchanged() {
        let mgr = new_decal_normal_manager();
        let n: Normal = [0.0, 0.0, 1.0];
        let out = dn_apply_all(&mgr, n);
        assert!((out[2] - 1.0).abs() < 1e-5);
    }
}
