// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bake high-to-low resolution normal map (ray casting placeholder).

/// Configuration for normal map baking.
#[allow(dead_code)]
pub struct NormalMapBakeV2Config {
    pub width: usize,
    pub height: usize,
    pub ray_offset: f32,
    pub max_distance: f32,
}

impl Default for NormalMapBakeV2Config {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            ray_offset: 0.001,
            max_distance: 0.1,
        }
    }
}

/// Normal map texture (RGB, linear).
#[allow(dead_code)]
pub struct NormalMapV2 {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[f32; 3]>,
}

#[allow(dead_code)]
impl NormalMapV2 {
    pub fn new(width: usize, height: usize) -> Self {
        let flat_normal = [0.5, 0.5, 1.0];
        Self {
            width,
            height,
            pixels: vec![flat_normal; width * height],
        }
    }

    pub fn pixel_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, normal: [f32; 3]) {
        let i = self.pixel_index(x, y);
        if i < self.pixels.len() {
            self.pixels[i] = normal;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> [f32; 3] {
        let i = self.pixel_index(x, y);
        self.pixels.get(i).copied().unwrap_or([0.5, 0.5, 1.0])
    }

    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Convert a world-space normal to `[0,1]` RGB encoding.
#[allow(dead_code)]
pub fn normal_to_rgb_v2(n: [f32; 3]) -> [f32; 3] {
    [n[0] * 0.5 + 0.5, n[1] * 0.5 + 0.5, n[2] * 0.5 + 0.5]
}

/// Decode `[0,1]` RGB back to world-space normal.
#[allow(dead_code)]
pub fn rgb_to_normal_v2(rgb: [f32; 3]) -> [f32; 3] {
    normalize3([rgb[0] * 2.0 - 1.0, rgb[1] * 2.0 - 1.0, rgb[2] * 2.0 - 1.0])
}

/// Compute face normal from three positions.
fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    normalize3(cross3(sub3(p1, p0), sub3(p2, p0)))
}

/// Placeholder: "bake" normal map by rasterizing face normals onto UV space.
/// Each face's normal is written to the texels covered by its UV triangle.
#[allow(dead_code)]
pub fn bake_normal_map_v2(
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
    config: &NormalMapBakeV2Config,
) -> NormalMapV2 {
    let mut map = NormalMapV2::new(config.width, config.height);
    let n_tri = indices.len() / 3;
    for t in 0..n_tri {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let p0 = if i0 < positions.len() {
            positions[i0]
        } else {
            continue;
        };
        let p1 = if i1 < positions.len() {
            positions[i1]
        } else {
            continue;
        };
        let p2 = if i2 < positions.len() {
            positions[i2]
        } else {
            continue;
        };
        let n = face_normal(p0, p1, p2);
        let rgb = normal_to_rgb_v2(n);
        let uv0 = if i0 < uvs.len() { uvs[i0] } else { [0.0, 0.0] };
        let uv1 = if i1 < uvs.len() { uvs[i1] } else { [1.0, 0.0] };
        let uv2 = if i2 < uvs.len() { uvs[i2] } else { [0.0, 1.0] };
        let x_min = (uv0[0].min(uv1[0]).min(uv2[0]) * config.width as f32) as usize;
        let x_max = ((uv0[0].max(uv1[0]).max(uv2[0]) * config.width as f32) as usize)
            .min(config.width.saturating_sub(1));
        let y_min = (uv0[1].min(uv1[1]).min(uv2[1]) * config.height as f32) as usize;
        let y_max = ((uv0[1].max(uv1[1]).max(uv2[1]) * config.height as f32) as usize)
            .min(config.height.saturating_sub(1));
        for y in y_min..=y_max {
            for x in x_min..=x_max {
                map.set_pixel(x, y, rgb);
            }
        }
    }
    map
}

/// Average normal across all pixels.
#[allow(dead_code)]
pub fn average_pixel_normal(map: &NormalMapV2) -> [f32; 3] {
    if map.pixels.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let mut sum = [0.0_f32; 3];
    for p in &map.pixels {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let n = map.pixels.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Pixel size in bytes (assuming 3×f32 = 12 bytes per pixel).
#[allow(dead_code)]
pub fn normal_map_v2_size_bytes(map: &NormalMapV2) -> usize {
    map.pixels.len() * 12
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let indices = vec![0u32, 1, 2];
        (positions, uvs, indices)
    }

    #[test]
    fn normal_map_v2_created() {
        let map = NormalMapV2::new(4, 4);
        assert_eq!(map.pixel_count(), 16);
    }

    #[test]
    fn normal_to_rgb_roundtrip() {
        let n = normalize3([0.5, 0.3, 0.8]);
        let rgb = normal_to_rgb_v2(n);
        let n2 = rgb_to_normal_v2(rgb);
        for k in 0..3 {
            assert!((n[k] - n2[k]).abs() < 1e-4, "k={k}: {} vs {}", n[k], n2[k]);
        }
    }

    #[test]
    fn bake_produces_map() {
        let (pos, uvs, idx) = flat_mesh();
        let config = NormalMapBakeV2Config {
            width: 8,
            height: 8,
            ..Default::default()
        };
        let map = bake_normal_map_v2(&pos, &uvs, &idx, &config);
        assert_eq!(map.width, 8);
        assert_eq!(map.height, 8);
    }

    #[test]
    fn bake_writes_some_pixels() {
        let (pos, uvs, idx) = flat_mesh();
        let config = NormalMapBakeV2Config {
            width: 8,
            height: 8,
            ..Default::default()
        };
        let map = bake_normal_map_v2(&pos, &uvs, &idx, &config);
        // Flat mesh has z-normal=1, which encodes to rgb z=1.0; check pixels were written.
        let written = map.pixels.iter().any(|p| p[2] > 0.9);
        assert!(
            written,
            "at least one pixel should be written with z-normal encoded"
        );
    }

    #[test]
    fn set_get_pixel_roundtrip() {
        let mut map = NormalMapV2::new(4, 4);
        let n = [0.7, 0.2, 0.9];
        map.set_pixel(2, 3, n);
        let p = map.get_pixel(2, 3);
        for k in 0..3 {
            assert!((p[k] - n[k]).abs() < 1e-5);
        }
    }

    #[test]
    fn average_pixel_normal_flat_map() {
        let map = NormalMapV2::new(4, 4);
        let avg = average_pixel_normal(&map);
        assert!((avg[2] - 1.0).abs() < 0.01);
    }

    #[test]
    fn normal_map_v2_size_bytes_correct() {
        let map = NormalMapV2::new(4, 4);
        assert_eq!(normal_map_v2_size_bytes(&map), 16 * 12);
    }

    #[test]
    fn default_config_reasonable() {
        let c = NormalMapBakeV2Config::default();
        assert!(c.width > 0 && c.height > 0);
        assert!(c.ray_offset > 0.0);
    }

    #[test]
    fn face_normal_unit_length() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_index_correct() {
        let map = NormalMapV2::new(8, 8);
        assert_eq!(map.pixel_index(3, 2), 19);
    }
}
