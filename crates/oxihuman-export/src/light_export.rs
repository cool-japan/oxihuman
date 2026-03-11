// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export light data to JSON-compatible format.

pub const LIGHT_POINT: u8 = 0;
pub const LIGHT_DIRECTIONAL: u8 = 1;
pub const LIGHT_SPOT: u8 = 2;

/* ── legacy API (kept) ── */

#[derive(Debug, Clone)]
pub struct LightExport {
    pub name: String,
    pub light_type: u8,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub cast_shadow: bool,
}

pub fn default_point_light_export(name: &str) -> LightExport {
    LightExport {
        name: name.to_string(),
        light_type: LIGHT_POINT,
        position: [0.0; 3],
        direction: [0.0, -1.0, 0.0],
        color: [1.0; 3],
        intensity: 1.0,
        radius: 0.0,
        cast_shadow: true,
    }
}

/* ── spec functions (wave 150B) ── */

/// Spec-style light data.
#[derive(Debug, Clone)]
pub struct LightData {
    pub name: String,
    pub light_type: String,
    pub color: [f32; 3],
    pub energy: f32,
    pub shadow: bool,
}

/// Create a new `LightData`.
pub fn new_light_data(name: &str, light_type: &str, energy: f32) -> LightData {
    LightData {
        name: name.to_string(),
        light_type: light_type.to_string(),
        color: [1.0, 1.0, 1.0],
        energy,
        shadow: true,
    }
}

/// Serialize a `LightData` to JSON.
pub fn light_to_json(l: &LightData) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"energy\":{},\"shadow\":{}}}",
        l.name, l.light_type, l.energy, l.shadow
    )
}

/// Serialize multiple lights to a JSON array.
pub fn lights_to_json(lights: &[LightData]) -> String {
    let inner: Vec<String> = lights.iter().map(light_to_json).collect();
    format!("[{}]", inner.join(","))
}

/// Lux-equivalent illuminance at a given distance (point light: E = energy / (4π d²)).
pub fn light_lux_at_distance(l: &LightData, distance: f32) -> f32 {
    use std::f32::consts::PI;
    if distance < f32::EPSILON {
        return f32::MAX;
    }
    l.energy / (4.0 * PI * distance * distance)
}

/// Returns true if the light is directional.
pub fn light_is_directional(l: &LightData) -> bool {
    l.light_type.eq_ignore_ascii_case("SUN") || l.light_type.eq_ignore_ascii_case("DIRECTIONAL")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_light_data() {
        let l = new_light_data("sun", "SUN", 5.0);
        assert_eq!(l.name, "sun");
    }

    #[test]
    fn test_light_to_json() {
        let l = new_light_data("pt", "POINT", 1.0);
        let j = light_to_json(&l);
        assert!(j.contains("POINT"));
    }

    #[test]
    fn test_light_lux_at_distance() {
        let l = new_light_data("l", "POINT", 4.0);
        let lux = light_lux_at_distance(&l, 1.0);
        assert!(lux > 0.0);
    }

    #[test]
    fn test_light_is_directional_true() {
        let l = new_light_data("sun", "SUN", 1.0);
        assert!(light_is_directional(&l));
    }

    #[test]
    fn test_light_is_directional_false() {
        let l = new_light_data("pt", "POINT", 1.0);
        assert!(!light_is_directional(&l));
    }
}
