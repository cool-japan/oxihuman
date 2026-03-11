// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TerrainParams {
    pub width: f32,
    pub depth: f32,
    pub nx: u32,
    pub nz: u32,
    pub max_height: f32,
}

pub fn new_terrain(width: f32, depth: f32, nx: u32, nz: u32, max_height: f32) -> TerrainParams {
    TerrainParams {
        width,
        depth,
        nx,
        nz,
        max_height,
    }
}

fn hash_noise(x: f32, z: f32) -> f32 {
    let xi = (x * 127.1) as i32;
    let zi = (z * 311.7) as i32;
    let n = xi.wrapping_mul(127) ^ zi.wrapping_mul(311);
    let n2 = n
        .wrapping_mul(n)
        .wrapping_mul(n.wrapping_mul(n).wrapping_add(1013904223));
    (n2 as f32 / i32::MAX as f32).abs() * 2.0 - 1.0
}

pub fn terrain_height_fbm(x: f32, z: f32, octaves: u32) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 1.0f32;
    let mut frequency = 1.0f32;
    let mut max_val = 0.0f32;
    for _ in 0..octaves {
        value += hash_noise(x * frequency, z * frequency) * amplitude;
        max_val += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    if max_val > 0.0 {
        value / max_val
    } else {
        0.0
    }
}

pub fn terrain_vertex(p: &TerrainParams, ix: u32, iz: u32) -> [f32; 3] {
    let x = (ix as f32 / p.nx as f32) * p.width;
    let z = (iz as f32 / p.nz as f32) * p.depth;
    let h = terrain_height_fbm(x / p.width, z / p.depth, 4) * p.max_height;
    [x, h, z]
}

pub fn terrain_vertex_count(p: &TerrainParams) -> usize {
    ((p.nx + 1) * (p.nz + 1)) as usize
}

pub fn terrain_face_count(p: &TerrainParams) -> usize {
    (p.nx * p.nz * 2) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_terrain() {
        /* construction */
        let t = new_terrain(10.0, 10.0, 32, 32, 2.0);
        assert!((t.width - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_terrain_vertex_count() {
        /* (nx+1)*(nz+1) */
        let t = new_terrain(10.0, 10.0, 8, 8, 1.0);
        assert_eq!(terrain_vertex_count(&t), 81);
    }

    #[test]
    fn test_terrain_face_count() {
        /* nx*nz*2 */
        let t = new_terrain(10.0, 10.0, 8, 8, 1.0);
        assert_eq!(terrain_face_count(&t), 128);
    }

    #[test]
    fn test_terrain_vertex_height_range() {
        /* height within max_height bounds */
        let t = new_terrain(10.0, 10.0, 8, 8, 2.0);
        let v = terrain_vertex(&t, 4, 4);
        assert!(v[1] >= -2.0 && v[1] <= 2.0);
    }

    #[test]
    fn test_terrain_fbm_range() {
        /* fbm in [-1,1] */
        let h = terrain_height_fbm(0.5, 0.3, 4);
        assert!((-1.0..=1.0).contains(&h));
    }

    #[test]
    fn test_terrain_vertex_xz_bounds() {
        /* x and z within width/depth */
        let t = new_terrain(5.0, 7.0, 4, 4, 1.0);
        let v = terrain_vertex(&t, 4, 4);
        assert!((v[0] - 5.0).abs() < 1e-4);
        assert!((v[2] - 7.0).abs() < 1e-4);
    }
}
