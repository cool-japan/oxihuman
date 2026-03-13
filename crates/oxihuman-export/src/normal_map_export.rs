// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Normal map generation and export (tangent-space and object-space).

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum NormalMapSpace {
    TangentSpace,
    ObjectSpace,
    WorldSpace,
}

#[allow(dead_code)]
pub struct NormalMapConfig {
    pub width: u32,
    pub height: u32,
    pub space: NormalMapSpace,
    pub flip_green: bool,
    pub samples: u32,
}

#[allow(dead_code)]
pub struct NormalMapBuffer {
    pub pixels: Vec<[u8; 3]>,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
pub fn default_normal_map_config(width: u32, height: u32) -> NormalMapConfig {
    NormalMapConfig {
        width,
        height,
        space: NormalMapSpace::TangentSpace,
        flip_green: false,
        samples: 1,
    }
}

#[allow(dead_code)]
pub fn new_normal_map_buffer(width: u32, height: u32) -> NormalMapBuffer {
    let count = (width * height) as usize;
    NormalMapBuffer {
        pixels: vec![[128, 128, 255]; count],
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn normal_to_rgb(n: [f32; 3]) -> [u8; 3] {
    let clamp = |v: f32| v.clamp(-1.0, 1.0);
    let to_u8 = |v: f32| ((clamp(v) * 0.5 + 0.5) * 255.0).round() as u8;
    [to_u8(n[0]), to_u8(n[1]), to_u8(n[2])]
}

#[allow(dead_code)]
pub fn rgb_to_normal(rgb: [u8; 3]) -> [f32; 3] {
    let to_f = |v: u8| (v as f32 / 255.0) * 2.0 - 1.0;
    let nx = to_f(rgb[0]);
    let ny = to_f(rgb[1]);
    let nz = to_f(rgb[2]);
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-8);
    [nx / len, ny / len, nz / len]
}

#[allow(dead_code)]
pub fn flat_normal_map(buffer: &mut NormalMapBuffer, normal: [f32; 3]) {
    let rgb = normal_to_rgb(normal);
    for pixel in buffer.pixels.iter_mut() {
        *pixel = rgb;
    }
}

#[allow(dead_code)]
pub fn encode_normal_map_ppm(buffer: &NormalMapBuffer) -> Vec<u8> {
    let header = format!("P6\n{} {}\n255\n", buffer.width, buffer.height);
    let mut out = header.into_bytes();
    for pixel in &buffer.pixels {
        out.push(pixel[0]);
        out.push(pixel[1]);
        out.push(pixel[2]);
    }
    out
}

#[allow(dead_code)]
pub fn compute_object_space_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    let tri_count = indices.len() / 3;
    for tri in 0..tri_count {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        let p0 = positions[i0];
        let p1 = positions[i1];
        let p2 = positions[i2];
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let cross = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        for idx in [i0, i1, i2] {
            normals[idx][0] += cross[0];
            normals[idx][1] += cross[1];
            normals[idx][2] += cross[2];
        }
    }
    for n in normals.iter_mut() {
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-8);
        n[0] /= len;
        n[1] /= len;
        n[2] /= len;
    }
    normals
}

#[allow(dead_code)]
pub fn normal_map_from_vertex_normals(
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    width: u32,
    height: u32,
) -> NormalMapBuffer {
    let mut buffer = new_normal_map_buffer(width, height);
    let count = normals.len().min(uvs.len());
    for i in 0..count {
        let u = uvs[i][0].clamp(0.0, 1.0);
        let v = uvs[i][1].clamp(0.0, 1.0);
        let px = (u * (width as f32 - 1.0)).round() as u32;
        let py = (v * (height as f32 - 1.0)).round() as u32;
        set_normal_pixel(&mut buffer, px, py, normals[i]);
    }
    buffer
}

#[allow(dead_code)]
pub fn blend_normal_maps(a: &NormalMapBuffer, b: &NormalMapBuffer, t: f32) -> NormalMapBuffer {
    let width = a.width;
    let height = a.height;
    let count = (width * height) as usize;
    let mut pixels = Vec::with_capacity(count);
    let t = t.clamp(0.0, 1.0);
    for i in 0..count {
        let na = rgb_to_normal(a.pixels[i]);
        let nb = if i < b.pixels.len() {
            rgb_to_normal(b.pixels[i])
        } else {
            [0.0, 0.0, 1.0]
        };
        let blended = [
            na[0] * (1.0 - t) + nb[0] * t,
            na[1] * (1.0 - t) + nb[1] * t,
            na[2] * (1.0 - t) + nb[2] * t,
        ];
        let len = (blended[0] * blended[0] + blended[1] * blended[1] + blended[2] * blended[2])
            .sqrt()
            .max(1e-8);
        let norm = [blended[0] / len, blended[1] / len, blended[2] / len];
        pixels.push(normal_to_rgb(norm));
    }
    NormalMapBuffer {
        pixels,
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn normal_map_pixel_count(buffer: &NormalMapBuffer) -> usize {
    buffer.pixels.len()
}

#[allow(dead_code)]
pub fn normal_map_size_bytes(buffer: &NormalMapBuffer) -> usize {
    buffer.pixels.len() * 3
}

#[allow(dead_code)]
pub fn set_normal_pixel(buffer: &mut NormalMapBuffer, x: u32, y: u32, normal: [f32; 3]) {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        buffer.pixels[idx] = normal_to_rgb(normal);
    }
}

#[allow(dead_code)]
pub fn get_normal_pixel(buffer: &NormalMapBuffer, x: u32, y: u32) -> [f32; 3] {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        rgb_to_normal(buffer.pixels[idx])
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_normal_map_config(512, 256);
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 256);
        assert_eq!(cfg.space, NormalMapSpace::TangentSpace);
        assert!(!cfg.flip_green);
        assert_eq!(cfg.samples, 1);
    }

    #[test]
    fn test_new_buffer() {
        let buf = new_normal_map_buffer(4, 4);
        assert_eq!(buf.width, 4);
        assert_eq!(buf.height, 4);
        assert_eq!(buf.pixels.len(), 16);
    }

    #[test]
    fn test_normal_to_rgb_round_trip() {
        let n = [0.0f32, 0.0, 1.0];
        let rgb = normal_to_rgb(n);
        let back = rgb_to_normal(rgb);
        assert!((back[2] - 1.0).abs() < 0.02);
    }

    #[test]
    fn test_normal_to_rgb_values() {
        let rgb = normal_to_rgb([0.0, 0.0, 1.0]);
        assert_eq!(rgb[0], 128);
        assert_eq!(rgb[1], 128);
        assert_eq!(rgb[2], 255);
    }

    #[test]
    fn test_rgb_to_normal_normalized() {
        let n = rgb_to_normal([255, 128, 128]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_flat_normal_map() {
        let mut buf = new_normal_map_buffer(4, 4);
        flat_normal_map(&mut buf, [0.0, 0.0, 1.0]);
        let rgb = normal_to_rgb([0.0, 0.0, 1.0]);
        for pixel in &buf.pixels {
            assert_eq!(pixel, &rgb);
        }
    }

    #[test]
    fn test_encode_ppm_starts_with_p6() {
        let buf = new_normal_map_buffer(2, 2);
        let ppm = encode_normal_map_ppm(&buf);
        assert!(ppm.starts_with(b"P6"));
    }

    #[test]
    fn test_encode_ppm_size() {
        let buf = new_normal_map_buffer(3, 3);
        let ppm = encode_normal_map_ppm(&buf);
        let header = b"P6\n3 3\n255\n";
        assert!(ppm.len() >= header.len() + 3 * 3 * 3);
    }

    #[test]
    fn test_normal_map_pixel_count() {
        let buf = new_normal_map_buffer(8, 8);
        assert_eq!(normal_map_pixel_count(&buf), 64);
    }

    #[test]
    fn test_normal_map_size_bytes() {
        let buf = new_normal_map_buffer(8, 8);
        assert_eq!(normal_map_size_bytes(&buf), 192);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut buf = new_normal_map_buffer(4, 4);
        let n = [1.0f32, 0.0, 0.0];
        set_normal_pixel(&mut buf, 2, 1, n);
        let got = get_normal_pixel(&buf, 2, 1);
        assert!((got[0] - 1.0).abs() < 0.05);
    }

    #[test]
    fn test_blend_maps() {
        let a = new_normal_map_buffer(2, 2);
        let mut b = new_normal_map_buffer(2, 2);
        flat_normal_map(&mut b, [1.0, 0.0, 0.0]);
        let blended = blend_normal_maps(&a, &b, 0.5);
        assert_eq!(blended.pixels.len(), 4);
    }

    #[test]
    fn test_object_space_normals_triangle() {
        let positions = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = [0u32, 1, 2];
        let normals = compute_object_space_normals(&positions, &indices);
        assert_eq!(normals.len(), 3);
        for n in &normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_normal_map_from_vertex_normals() {
        let normals = vec![[0.0f32, 0.0, 1.0]; 4];
        let uvs = vec![[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        let buf = normal_map_from_vertex_normals(&normals, &uvs, 8, 8);
        assert_eq!(buf.width, 8);
        assert_eq!(buf.height, 8);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let buf = new_normal_map_buffer(4, 4);
        let n = get_normal_pixel(&buf, 10, 10);
        assert_eq!(n, [0.0, 0.0, 1.0]);
    }
}
