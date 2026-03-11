// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Environment light (IBL) state management.

use std::f32::consts::PI;

/// IBL source type.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvLightSource {
    Procedural,
    Hdri,
    SolidColor,
}

/// Environment light state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct EnvLight {
    pub source: EnvLightSource,
    pub intensity: f32,
    pub rotation_rad: f32,
    pub tint: [f32; 3],
    pub enabled: bool,
}

impl Default for EnvLight {
    fn default() -> Self {
        Self {
            source: EnvLightSource::Procedural,
            intensity: 1.0,
            rotation_rad: 0.0,
            tint: [1.0, 1.0, 1.0],
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_env_light() -> EnvLight {
    EnvLight::default()
}

#[allow(dead_code)]
pub fn el_set_intensity(light: &mut EnvLight, v: f32) {
    light.intensity = v.max(0.0);
}

#[allow(dead_code)]
pub fn el_set_rotation_deg(light: &mut EnvLight, deg: f32) {
    light.rotation_rad = deg.to_radians() % (2.0 * PI);
}

#[allow(dead_code)]
pub fn el_set_tint(light: &mut EnvLight, r: f32, g: f32, b: f32) {
    light.tint = [r.clamp(0.0, 2.0), g.clamp(0.0, 2.0), b.clamp(0.0, 2.0)];
}

#[allow(dead_code)]
pub fn el_set_enabled(light: &mut EnvLight, v: bool) {
    light.enabled = v;
}

#[allow(dead_code)]
pub fn el_set_source(light: &mut EnvLight, src: EnvLightSource) {
    light.source = src;
}

#[allow(dead_code)]
pub fn el_effective_intensity(light: &EnvLight) -> f32 {
    if light.enabled {
        light.intensity
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn el_tinted_intensity(light: &EnvLight) -> [f32; 3] {
    let i = el_effective_intensity(light);
    [light.tint[0] * i, light.tint[1] * i, light.tint[2] * i]
}

#[allow(dead_code)]
pub fn el_source_name(src: EnvLightSource) -> &'static str {
    match src {
        EnvLightSource::Procedural => "procedural",
        EnvLightSource::Hdri => "hdri",
        EnvLightSource::SolidColor => "solid_color",
    }
}

#[allow(dead_code)]
pub fn el_blend(a: &EnvLight, b: &EnvLight, t: f32) -> EnvLight {
    let t = t.clamp(0.0, 1.0);
    EnvLight {
        source: if t < 0.5 { a.source } else { b.source },
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        rotation_rad: a.rotation_rad + (b.rotation_rad - a.rotation_rad) * t,
        tint: [
            a.tint[0] + (b.tint[0] - a.tint[0]) * t,
            a.tint[1] + (b.tint[1] - a.tint[1]) * t,
            a.tint[2] + (b.tint[2] - a.tint[2]) * t,
        ],
        enabled: a.enabled,
    }
}

#[allow(dead_code)]
pub fn el_to_json(light: &EnvLight) -> String {
    format!(
        "{{\"source\":\"{}\",\"intensity\":{:.4},\"rotation_rad\":{:.4},\"enabled\":{}}}",
        el_source_name(light.source),
        light.intensity,
        light.rotation_rad,
        light.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_enabled() {
        assert!(new_env_light().enabled);
    }

    #[test]
    fn intensity_not_negative() {
        let mut l = new_env_light();
        el_set_intensity(&mut l, -5.0);
        assert!(l.intensity >= 0.0);
    }

    #[test]
    fn disabled_intensity_zero() {
        let mut l = new_env_light();
        el_set_enabled(&mut l, false);
        assert!((el_effective_intensity(&l)).abs() < 1e-5);
    }

    #[test]
    fn tint_clamps() {
        let mut l = new_env_light();
        el_set_tint(&mut l, 5.0, -1.0, 1.5);
        assert!(l.tint[0] <= 2.0);
        assert!(l.tint[1] >= 0.0);
    }

    #[test]
    fn rotation_wraps() {
        let mut l = new_env_light();
        el_set_rotation_deg(&mut l, 720.0);
        assert!(l.rotation_rad.abs() < 2.0 * PI + 1e-4);
    }

    #[test]
    fn source_name_hdri() {
        assert_eq!(el_source_name(EnvLightSource::Hdri), "hdri");
    }

    #[test]
    fn tinted_intensity_correct() {
        let mut l = new_env_light();
        el_set_intensity(&mut l, 2.0);
        el_set_tint(&mut l, 0.5, 1.0, 1.0);
        let t = el_tinted_intensity(&l);
        assert!((t[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn blend_midpoint_intensity() {
        let mut a = new_env_light();
        let mut b = new_env_light();
        el_set_intensity(&mut a, 0.0);
        el_set_intensity(&mut b, 2.0);
        let m = el_blend(&a, &b, 0.5);
        assert!((m.intensity - 1.0).abs() < 1e-4);
    }

    #[test]
    fn json_has_source() {
        assert!(el_to_json(&new_env_light()).contains("source"));
    }

    #[test]
    fn set_source() {
        let mut l = new_env_light();
        el_set_source(&mut l, EnvLightSource::Hdri);
        assert_eq!(l.source, EnvLightSource::Hdri);
    }
}
