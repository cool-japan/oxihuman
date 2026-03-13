// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Deferred lighting pass configuration and light accumulation.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DeferredLightType {
    Directional,
    Point,
    Spot,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeferredLight {
    pub light_type: DeferredLightType,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32,
    pub spot_angle_deg: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeferredLightingConfig {
    pub max_lights: usize,
    pub ambient_color: [f32; 3],
    pub ambient_intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightAccumBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 4]>,
}

#[allow(dead_code)]
pub fn default_deferred_config() -> DeferredLightingConfig {
    DeferredLightingConfig {
        max_lights: 64,
        ambient_color: [0.1, 0.1, 0.15],
        ambient_intensity: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_directional_light(dir: [f32; 3], color: [f32; 3], intensity: f32) -> DeferredLight {
    DeferredLight {
        light_type: DeferredLightType::Directional,
        position: [0.0; 3],
        direction: dir,
        color,
        intensity,
        radius: 0.0,
        spot_angle_deg: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_point_light(pos: [f32; 3], color: [f32; 3], intensity: f32, radius: f32) -> DeferredLight {
    DeferredLight {
        light_type: DeferredLightType::Point,
        position: pos,
        direction: [0.0; 3],
        color,
        intensity,
        radius,
        spot_angle_deg: 0.0,
    }
}

#[allow(dead_code)]
pub fn point_light_attenuation(distance: f32, radius: f32) -> f32 {
    if distance >= radius || radius <= 0.0 {
        return 0.0;
    }
    let ratio = distance / radius;
    (1.0 - ratio * ratio).max(0.0)
}

#[allow(dead_code)]
pub fn spot_light_falloff(angle_rad: f32, cone_angle_rad: f32) -> f32 {
    if angle_rad >= cone_angle_rad {
        return 0.0;
    }
    let t = angle_rad / cone_angle_rad;
    let _ = PI; // use PI constant
    (1.0 - t * t).max(0.0)
}

#[allow(dead_code)]
pub fn new_accum_buffer(w: u32, h: u32) -> LightAccumBuffer {
    LightAccumBuffer {
        width: w,
        height: h,
        data: vec![[0.0, 0.0, 0.0, 1.0]; (w as usize) * (h as usize)],
    }
}

#[allow(dead_code)]
pub fn clear_accum_buffer(buf: &mut LightAccumBuffer) {
    for px in &mut buf.data {
        *px = [0.0, 0.0, 0.0, 1.0];
    }
}

#[allow(dead_code)]
pub fn light_to_json(light: &DeferredLight) -> String {
    let t = match &light.light_type {
        DeferredLightType::Directional => "directional",
        DeferredLightType::Point => "point",
        DeferredLightType::Spot => "spot",
    };
    format!(
        r#"{{"type":"{}","intensity":{},"radius":{}}}"#,
        t, light.intensity, light.radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_deferred_config();
        assert_eq!(c.max_lights, 64);
    }

    #[test]
    fn test_new_directional() {
        let l = new_directional_light([0.0, -1.0, 0.0], [1.0; 3], 1.0);
        assert_eq!(l.light_type, DeferredLightType::Directional);
    }

    #[test]
    fn test_new_point() {
        let l = new_point_light([0.0, 2.0, 0.0], [1.0; 3], 2.0, 5.0);
        assert_eq!(l.light_type, DeferredLightType::Point);
        assert!((l.radius - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_attenuation_zero() {
        assert!(point_light_attenuation(5.0, 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_attenuation_center() {
        let a = point_light_attenuation(0.0, 5.0);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_attenuation_mid() {
        let a = point_light_attenuation(2.5, 5.0);
        assert!((0.0..=1.0).contains(&a));
    }

    #[test]
    fn test_spot_falloff_edge() {
        assert!(spot_light_falloff(1.0, 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_accum_buffer() {
        let buf = new_accum_buffer(4, 4);
        assert_eq!(buf.data.len(), 16);
    }

    #[test]
    fn test_clear_buffer() {
        let mut buf = new_accum_buffer(2, 2);
        buf.data[0] = [1.0, 1.0, 1.0, 1.0];
        clear_accum_buffer(&mut buf);
        assert!(buf.data[0][0].abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let l = new_directional_light([0.0, -1.0, 0.0], [1.0; 3], 1.0);
        let j = light_to_json(&l);
        assert!(j.contains("directional"));
    }
}
