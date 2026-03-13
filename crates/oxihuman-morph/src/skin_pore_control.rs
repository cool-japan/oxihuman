// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin pore size and density morph parameters.

/// Parameters for skin pore morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinPoreParams {
    pub size: f32,
    pub density: f32,
    pub depth: f32,
    pub variation: f32,
}

impl Default for SkinPoreParams {
    fn default() -> Self {
        SkinPoreParams {
            size: 0.0,
            density: 0.0,
            depth: 0.0,
            variation: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn default_skin_pore_params() -> SkinPoreParams {
    SkinPoreParams::default()
}

#[allow(dead_code)]
pub fn sp_set_size(p: &mut SkinPoreParams, v: f32) {
    p.size = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sp_set_density(p: &mut SkinPoreParams, v: f32) {
    p.density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sp_set_depth(p: &mut SkinPoreParams, v: f32) {
    p.depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sp_set_variation(p: &mut SkinPoreParams, v: f32) {
    p.variation = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sp_reset(p: &mut SkinPoreParams) {
    *p = SkinPoreParams::default();
}

#[allow(dead_code)]
pub fn sp_is_neutral(p: &SkinPoreParams) -> bool {
    p.size.abs() < 1e-6 && p.density.abs() < 1e-6
}

#[allow(dead_code)]
pub fn sp_pore_visibility(p: &SkinPoreParams) -> f32 {
    p.size * 0.5 + p.depth * 0.3 + p.density * 0.2
}

#[allow(dead_code)]
pub fn sp_blend(a: &SkinPoreParams, b: &SkinPoreParams, t: f32) -> SkinPoreParams {
    let t = t.clamp(0.0, 1.0);
    SkinPoreParams {
        size: a.size + (b.size - a.size) * t,
        density: a.density + (b.density - a.density) * t,
        depth: a.depth + (b.depth - a.depth) * t,
        variation: a.variation + (b.variation - a.variation) * t,
    }
}

#[allow(dead_code)]
pub fn sp_to_json(p: &SkinPoreParams) -> String {
    format!(
        r#"{{"size":{:.4},"density":{:.4},"depth":{:.4},"variation":{:.4}}}"#,
        p.size, p.density, p.depth, p.variation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral() {
        assert!(sp_is_neutral(&default_skin_pore_params()));
    }

    #[test]
    fn set_size_clamps() {
        let mut p = default_skin_pore_params();
        sp_set_size(&mut p, 2.0);
        assert!((p.size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_density_clamps_negative() {
        let mut p = default_skin_pore_params();
        sp_set_density(&mut p, -0.5);
        assert!(p.density.abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut p = default_skin_pore_params();
        sp_set_size(&mut p, 0.8);
        sp_reset(&mut p);
        assert!(sp_is_neutral(&p));
    }

    #[test]
    fn pore_visibility_zero_when_neutral() {
        assert!(sp_pore_visibility(&default_skin_pore_params()).abs() < 1e-6);
    }

    #[test]
    fn pore_visibility_increases() {
        let mut p = default_skin_pore_params();
        sp_set_size(&mut p, 1.0);
        assert!(sp_pore_visibility(&p) > 0.0);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_skin_pore_params();
        let mut b = default_skin_pore_params();
        sp_set_size(&mut b, 1.0);
        let m = sp_blend(&a, &b, 0.5);
        assert!((m.size - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_clamp_t() {
        let a = default_skin_pore_params();
        let b = default_skin_pore_params();
        let m = sp_blend(&a, &b, 5.0);
        assert!((m.size - b.size).abs() < 1e-5);
    }

    #[test]
    fn to_json_has_density() {
        assert!(sp_to_json(&default_skin_pore_params()).contains("density"));
    }
}
