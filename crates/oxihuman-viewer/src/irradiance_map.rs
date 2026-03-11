// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Irradiance map generation and sampling for diffuse environment lighting.

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceMapConfig {
    pub resolution: u32,
    pub sample_count: u32,
    pub intensity: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceSample {
    pub direction: [f32; 3],
    pub color: [f32; 3],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IrradianceMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn default_irradiance_config() -> IrradianceMapConfig {
    IrradianceMapConfig {
        resolution: 32,
        sample_count: 64,
        intensity: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_irradiance_map(w: u32, h: u32) -> IrradianceMap {
    IrradianceMap {
        width: w,
        height: h,
        data: vec![[0.0; 3]; (w as usize) * (h as usize)],
    }
}

#[allow(dead_code)]
pub fn spherical_to_direction(theta: f32, phi: f32) -> [f32; 3] {
    [
        theta.sin() * phi.cos(),
        theta.cos(),
        theta.sin() * phi.sin(),
    ]
}

#[allow(dead_code)]
pub fn direction_to_spherical(dir: [f32; 3]) -> (f32, f32) {
    let theta = dir[1].clamp(-1.0, 1.0).acos();
    let phi = dir[2].atan2(dir[0]);
    (theta, phi)
}

#[allow(dead_code)]
pub fn uv_to_direction(u: f32, v: f32) -> [f32; 3] {
    let theta = v * PI;
    let phi = u * 2.0 * PI;
    spherical_to_direction(theta, phi)
}

#[allow(dead_code)]
pub fn direction_to_uv(dir: [f32; 3]) -> (f32, f32) {
    let (theta, phi) = direction_to_spherical(dir);
    let u = (phi / (2.0 * PI) + 0.5) % 1.0;
    let v = theta / PI;
    (u, v)
}

#[allow(dead_code)]
pub fn sample_irradiance(map: &IrradianceMap, u: f32, v: f32) -> [f32; 3] {
    let x = ((u * map.width as f32) as u32).min(map.width - 1) as usize;
    let y = ((v * map.height as f32) as u32).min(map.height - 1) as usize;
    map.data[y * map.width as usize + x]
}

#[allow(dead_code)]
pub fn set_irradiance_pixel(map: &mut IrradianceMap, x: u32, y: u32, color: [f32; 3]) {
    if x < map.width && y < map.height {
        map.data[(y as usize) * (map.width as usize) + (x as usize)] = color;
    }
}

#[allow(dead_code)]
pub fn irradiance_map_to_json(cfg: &IrradianceMapConfig) -> String {
    format!(
        r#"{{"resolution":{},"samples":{},"intensity":{}}}"#,
        cfg.resolution, cfg.sample_count, cfg.intensity
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = default_irradiance_config();
        assert_eq!(c.resolution, 32);
    }

    #[test]
    fn test_new_map() {
        let m = new_irradiance_map(16, 8);
        assert_eq!(m.data.len(), 128);
    }

    #[test]
    fn test_spherical_up() {
        let d = spherical_to_direction(0.0, 0.0);
        assert!((d[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_spherical_down() {
        let d = spherical_to_direction(PI, 0.0);
        assert!((d[1] - (-1.0)).abs() < 1e-4);
    }

    #[test]
    fn test_direction_roundtrip() {
        let dir = [0.0, 1.0, 0.0];
        let (theta, phi) = direction_to_spherical(dir);
        let back = spherical_to_direction(theta, phi);
        assert!((back[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_uv_center() {
        let d = uv_to_direction(0.5, 0.5);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_sample() {
        let mut m = new_irradiance_map(4, 4);
        m.data[0] = [1.0, 0.5, 0.2];
        let s = sample_irradiance(&m, 0.0, 0.0);
        assert!((s[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_pixel() {
        let mut m = new_irradiance_map(4, 4);
        set_irradiance_pixel(&mut m, 1, 1, [0.5, 0.5, 0.5]);
        assert!((m.data[5][0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let c = default_irradiance_config();
        let j = irradiance_map_to_json(&c);
        assert!(j.contains("resolution"));
    }

    #[test]
    fn test_direction_to_uv() {
        let (u, v) = direction_to_uv([0.0, 1.0, 0.0]);
        assert!((0.0..=1.0).contains(&u));
        assert!((0.0..=1.0).contains(&v));
    }
}
