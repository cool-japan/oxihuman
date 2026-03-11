// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export environment maps (cubemap, equirectangular) for IBL.

use std::f32::consts::PI;

/// Environment map face.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CubeFace { PosX, NegX, PosY, NegY, PosZ, NegZ }

/// Environment map data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EnvMap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<[f32; 3]>,
    pub is_hdr: bool,
}

#[allow(dead_code)]
pub fn new_env_map(w: u32, h: u32) -> EnvMap {
    EnvMap { width: w, height: h, data: vec![[0.0; 3]; (w * h) as usize], is_hdr: false }
}

#[allow(dead_code)]
pub fn env_set_pixel(map: &mut EnvMap, x: u32, y: u32, rgb: [f32; 3]) {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() { map.data[idx] = rgb; }
}

#[allow(dead_code)]
pub fn env_get_pixel(map: &EnvMap, x: u32, y: u32) -> [f32; 3] {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() { map.data[idx] } else { [0.0; 3] }
}

/// Convert equirectangular UV to direction vector.
#[allow(dead_code)]
pub fn equirect_to_dir(u: f32, v: f32) -> [f32; 3] {
    let theta = v * PI;
    let phi = u * 2.0 * PI;
    [theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin()]
}

/// Convert direction to equirectangular UV.
#[allow(dead_code)]
pub fn dir_to_equirect(dir: [f32; 3]) -> [f32; 2] {
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len < 1e-12 { return [0.0; 2]; }
    let d = [dir[0] / len, dir[1] / len, dir[2] / len];
    let theta = d[1].clamp(-1.0, 1.0).acos();
    let phi = d[2].atan2(d[0]);
    let u = (phi / (2.0 * PI) + 0.5).fract();
    let v = theta / PI;
    [u, v]
}

#[allow(dead_code)]
pub fn env_pixel_count(map: &EnvMap) -> usize { map.data.len() }

#[allow(dead_code)]
pub fn env_average_luminance(map: &EnvMap) -> f32 {
    if map.data.is_empty() { return 0.0; }
    let sum: f32 = map.data.iter().map(|c| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]).sum();
    sum / map.data.len() as f32
}

#[allow(dead_code)]
pub fn cube_face_name(face: CubeFace) -> &'static str {
    match face { CubeFace::PosX => "+X", CubeFace::NegX => "-X", CubeFace::PosY => "+Y",
        CubeFace::NegY => "-Y", CubeFace::PosZ => "+Z", CubeFace::NegZ => "-Z" }
}

#[allow(dead_code)]
pub fn env_to_json(map: &EnvMap) -> String {
    format!(r#"{{"width":{},"height":{},"hdr":{},"pixels":{}}}"#, map.width, map.height, map.is_hdr, map.data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::f32::consts::PI;

    #[test]
    fn test_new_env_map() {
        let m = new_env_map(4, 2);
        assert_eq!(env_pixel_count(&m), 8);
    }

    #[test]
    fn test_set_get() {
        let mut m = new_env_map(2, 2);
        env_set_pixel(&mut m, 1, 0, [1.0, 0.5, 0.0]);
        let p = env_get_pixel(&m, 1, 0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_equirect_roundtrip() {
        let dir = equirect_to_dir(0.25, 0.5);
        let uv = dir_to_equirect(dir);
        assert!((uv[0] - 0.25).abs() < 0.01);
        assert!((uv[1] - 0.5).abs() < 0.01);
        let _ = PI;
    }

    #[test]
    fn test_luminance() {
        let mut m = new_env_map(1, 1);
        env_set_pixel(&mut m, 0, 0, [1.0, 1.0, 1.0]);
        let lum = env_average_luminance(&m);
        assert!((lum - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_face_name() {
        assert_eq!(cube_face_name(CubeFace::PosX), "+X");
    }

    #[test]
    fn test_to_json() {
        let m = new_env_map(2, 2);
        let json = env_to_json(&m);
        assert!(json.contains("width"));
    }

    #[test]
    fn test_empty_luminance() {
        let m = new_env_map(0, 0);
        assert!((env_average_luminance(&m) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_equirect_pole() {
        let dir = equirect_to_dir(0.0, 0.0);
        assert!((dir[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_out_of_bounds() {
        let m = new_env_map(2, 2);
        let p = env_get_pixel(&m, 99, 99);
        assert!((p[0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_hdr_flag() {
        let mut m = new_env_map(1, 1);
        m.is_hdr = true;
        let json = env_to_json(&m);
        assert!(json.contains("true"));
    }

}
