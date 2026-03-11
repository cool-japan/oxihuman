// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export bone/skin weight maps as images or data files.

/// A single bone weight assignment for a vertex.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BoneWeight {
    pub bone_index: u32,
    pub weight: f32,
}

/// Configuration for weight map generation.
#[allow(dead_code)]
pub struct WeightMapConfig {
    pub width: u32,
    pub height: u32,
    pub bone_index: u32,
    pub gamma: f32,
}

/// Pixel buffer storing grayscale weight values.
#[allow(dead_code)]
pub struct WeightMapBuffer {
    pub pixels: Vec<f32>,
    pub width: u32,
    pub height: u32,
}

/// Statistics for a weight map buffer.
#[allow(dead_code)]
pub struct WeightMapStats {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
}

/// Type alias for weight-per-vertex data (vertex_index, weights).
#[allow(dead_code)]
pub type VertexWeights = Vec<Vec<BoneWeight>>;

#[allow(dead_code)]
pub fn default_weight_map_config(width: u32, height: u32) -> WeightMapConfig {
    WeightMapConfig {
        width,
        height,
        bone_index: 0,
        gamma: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_weight_map_buffer(width: u32, height: u32) -> WeightMapBuffer {
    let count = (width * height) as usize;
    WeightMapBuffer {
        pixels: vec![0.0; count],
        width,
        height,
    }
}

#[allow(dead_code)]
pub fn set_weight_pixel(buffer: &mut WeightMapBuffer, x: u32, y: u32, value: f32) {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        buffer.pixels[idx] = value;
    }
}

#[allow(dead_code)]
pub fn get_weight_pixel(buffer: &WeightMapBuffer, x: u32, y: u32) -> f32 {
    if x < buffer.width && y < buffer.height {
        let idx = (y * buffer.width + x) as usize;
        buffer.pixels[idx]
    } else {
        0.0
    }
}

/// Rasterize per-vertex weights to UV space for a specific bone.
#[allow(dead_code)]
pub fn weight_map_from_vertices(
    vertex_weights: &[Vec<BoneWeight>],
    uvs: &[[f32; 2]],
    bone_index: u32,
    width: u32,
    height: u32,
) -> WeightMapBuffer {
    let mut buffer = new_weight_map_buffer(width, height);
    let count = vertex_weights.len().min(uvs.len());
    for i in 0..count {
        let w = vertex_weights[i]
            .iter()
            .find(|bw| bw.bone_index == bone_index)
            .map_or(0.0, |bw| bw.weight);
        let u = uvs[i][0].clamp(0.0, 1.0);
        let v = uvs[i][1].clamp(0.0, 1.0);
        let px = (u * (width as f32 - 1.0)).round() as u32;
        let py = (v * (height as f32 - 1.0)).round() as u32;
        set_weight_pixel(&mut buffer, px, py, w);
    }
    buffer
}

/// Encode weight map as grayscale PPM (P5 PGM).
#[allow(dead_code)]
pub fn encode_weight_map_ppm(buffer: &WeightMapBuffer) -> Vec<u8> {
    let header = format!("P5\n{} {}\n255\n", buffer.width, buffer.height);
    let mut out = header.into_bytes();
    for &val in &buffer.pixels {
        let byte = (val.clamp(0.0, 1.0) * 255.0).round() as u8;
        out.push(byte);
    }
    out
}

/// Normalize weights so that per-vertex weights sum to 1.0.
#[allow(dead_code)]
pub fn normalize_weights(vertex_weights: &mut [Vec<BoneWeight>]) {
    for weights in vertex_weights.iter_mut() {
        let sum: f32 = weights.iter().map(|bw| bw.weight).sum();
        if sum > 1e-8 {
            for bw in weights.iter_mut() {
                bw.weight /= sum;
            }
        }
    }
}

/// Get the top N highest weights per vertex.
#[allow(dead_code)]
pub fn top_n_weights(weights: &[BoneWeight], n: usize) -> Vec<BoneWeight> {
    let mut sorted: Vec<BoneWeight> = weights.to_vec();
    sorted.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.truncate(n);
    sorted
}

/// Export weight map buffer to CSV string.
#[allow(dead_code)]
pub fn weight_map_to_csv(buffer: &WeightMapBuffer) -> String {
    let mut out = String::from("x,y,weight\n");
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let val = get_weight_pixel(buffer, x, y);
            out.push_str(&format!("{},{},{:.6}\n", x, y, val));
        }
    }
    out
}

/// Blend two weight maps with factor t (0 = all a, 1 = all b).
#[allow(dead_code)]
pub fn blend_weight_maps(a: &WeightMapBuffer, b: &WeightMapBuffer, t: f32) -> WeightMapBuffer {
    let width = a.width;
    let height = a.height;
    let count = (width * height) as usize;
    let t = t.clamp(0.0, 1.0);
    let mut pixels = Vec::with_capacity(count);
    for i in 0..count {
        let va = a.pixels[i];
        let vb = if i < b.pixels.len() { b.pixels[i] } else { 0.0 };
        pixels.push(va * (1.0 - t) + vb * t);
    }
    WeightMapBuffer {
        pixels,
        width,
        height,
    }
}

/// Return total pixel count of the weight map.
#[allow(dead_code)]
pub fn weight_map_pixel_count(buffer: &WeightMapBuffer) -> usize {
    buffer.pixels.len()
}

/// Compute min, max, and average weight values.
#[allow(dead_code)]
pub fn weight_map_stats(buffer: &WeightMapBuffer) -> WeightMapStats {
    if buffer.pixels.is_empty() {
        return WeightMapStats {
            min: 0.0,
            max: 0.0,
            avg: 0.0,
        };
    }
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    let mut sum = 0.0f64;
    for &v in &buffer.pixels {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
        sum += v as f64;
    }
    let avg = (sum / buffer.pixels.len() as f64) as f32;
    WeightMapStats { min, max, avg }
}

/// Clear all pixels to zero.
#[allow(dead_code)]
pub fn clear_weight_map(buffer: &mut WeightMapBuffer) {
    for px in buffer.pixels.iter_mut() {
        *px = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_weight_map_config(256, 256);
        assert_eq!(cfg.width, 256);
        assert_eq!(cfg.height, 256);
        assert_eq!(cfg.bone_index, 0);
        assert!((cfg.gamma - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_new_buffer() {
        let buf = new_weight_map_buffer(8, 8);
        assert_eq!(buf.width, 8);
        assert_eq!(buf.height, 8);
        assert_eq!(buf.pixels.len(), 64);
    }

    #[test]
    fn test_set_get_pixel() {
        let mut buf = new_weight_map_buffer(4, 4);
        set_weight_pixel(&mut buf, 2, 3, 0.75);
        let v = get_weight_pixel(&buf, 2, 3);
        assert!((v - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_pixel_out_of_bounds() {
        let buf = new_weight_map_buffer(4, 4);
        let v = get_weight_pixel(&buf, 10, 10);
        assert!((v - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_pixel_out_of_bounds() {
        let mut buf = new_weight_map_buffer(4, 4);
        set_weight_pixel(&mut buf, 10, 10, 1.0);
        // No panic; all pixels stay 0
        for &px in &buf.pixels {
            assert!((px - 0.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_weight_map_from_vertices() {
        let vw = vec![
            vec![BoneWeight {
                bone_index: 0,
                weight: 1.0,
            }],
            vec![BoneWeight {
                bone_index: 1,
                weight: 0.5,
            }],
        ];
        let uvs = [[0.0, 0.0], [0.5, 0.5]];
        let buf = weight_map_from_vertices(&vw, &uvs, 0, 8, 8);
        assert_eq!(buf.width, 8);
        // The first vertex at (0,0) should have weight 1.0
        assert!((get_weight_pixel(&buf, 0, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_encode_ppm_starts_with_p5() {
        let buf = new_weight_map_buffer(2, 2);
        let ppm = encode_weight_map_ppm(&buf);
        assert!(ppm.starts_with(b"P5"));
    }

    #[test]
    fn test_encode_ppm_size() {
        let buf = new_weight_map_buffer(4, 4);
        let ppm = encode_weight_map_ppm(&buf);
        let header = "P5\n4 4\n255\n".to_string();
        assert_eq!(ppm.len(), header.len() + 16);
    }

    #[test]
    fn test_normalize_weights() {
        let mut vw = vec![vec![
            BoneWeight {
                bone_index: 0,
                weight: 2.0,
            },
            BoneWeight {
                bone_index: 1,
                weight: 3.0,
            },
        ]];
        normalize_weights(&mut vw);
        let sum: f32 = vw[0].iter().map(|bw| bw.weight).sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_weights_zero_sum() {
        let mut vw = vec![vec![BoneWeight {
            bone_index: 0,
            weight: 0.0,
        }]];
        normalize_weights(&mut vw);
        // Should not panic, weight stays 0
        assert!((vw[0][0].weight - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_top_n_weights() {
        let weights = vec![
            BoneWeight {
                bone_index: 0,
                weight: 0.1,
            },
            BoneWeight {
                bone_index: 1,
                weight: 0.5,
            },
            BoneWeight {
                bone_index: 2,
                weight: 0.3,
            },
            BoneWeight {
                bone_index: 3,
                weight: 0.1,
            },
        ];
        let top = top_n_weights(&weights, 2);
        assert_eq!(top.len(), 2);
        assert!((top[0].weight - 0.5).abs() < f32::EPSILON);
        assert!((top[1].weight - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_weight_map_to_csv() {
        let mut buf = new_weight_map_buffer(2, 2);
        set_weight_pixel(&mut buf, 0, 0, 1.0);
        let csv = weight_map_to_csv(&buf);
        assert!(csv.starts_with("x,y,weight\n"));
        assert!(csv.contains("0,0,1.000000"));
    }

    #[test]
    fn test_blend_weight_maps() {
        let mut a = new_weight_map_buffer(2, 2);
        let mut b = new_weight_map_buffer(2, 2);
        for px in a.pixels.iter_mut() {
            *px = 0.0;
        }
        for px in b.pixels.iter_mut() {
            *px = 1.0;
        }
        let blended = blend_weight_maps(&a, &b, 0.5);
        for &px in &blended.pixels {
            assert!((px - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_weight_map_pixel_count() {
        let buf = new_weight_map_buffer(8, 4);
        assert_eq!(weight_map_pixel_count(&buf), 32);
    }

    #[test]
    fn test_weight_map_stats() {
        let mut buf = new_weight_map_buffer(4, 4);
        set_weight_pixel(&mut buf, 0, 0, 0.2);
        set_weight_pixel(&mut buf, 1, 0, 0.8);
        let stats = weight_map_stats(&buf);
        assert!((stats.min - 0.0).abs() < f32::EPSILON);
        assert!((stats.max - 0.8).abs() < f32::EPSILON);
        assert!(stats.avg > 0.0);
    }

    #[test]
    fn test_weight_map_stats_empty() {
        let buf = WeightMapBuffer {
            pixels: vec![],
            width: 0,
            height: 0,
        };
        let stats = weight_map_stats(&buf);
        assert!((stats.min - 0.0).abs() < f32::EPSILON);
        assert!((stats.max - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clear_weight_map() {
        let mut buf = new_weight_map_buffer(4, 4);
        set_weight_pixel(&mut buf, 1, 1, 0.9);
        clear_weight_map(&mut buf);
        for &px in &buf.pixels {
            assert!((px - 0.0).abs() < f32::EPSILON);
        }
    }
}
