// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export ambient occlusion maps for mesh surfaces.

/// AO map buffer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionMapExport {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_occlusion_map(w: u32, h: u32) -> OcclusionMapExport {
    OcclusionMapExport { width: w, height: h, data: vec![1.0; (w * h) as usize] }
}

#[allow(dead_code)]
pub fn occ_set_pixel(map: &mut OcclusionMapExport, x: u32, y: u32, value: f32) {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() { map.data[idx] = value.clamp(0.0, 1.0); }
}

#[allow(dead_code)]
pub fn occ_get_pixel(map: &OcclusionMapExport, x: u32, y: u32) -> f32 {
    let idx = (y * map.width + x) as usize;
    if idx < map.data.len() { map.data[idx] } else { 1.0 }
}

#[allow(dead_code)]
pub fn occ_pixel_count(map: &OcclusionMapExport) -> usize { map.data.len() }

#[allow(dead_code)]
pub fn occ_average(map: &OcclusionMapExport) -> f32 {
    if map.data.is_empty() { return 1.0; }
    map.data.iter().sum::<f32>() / map.data.len() as f32
}

#[allow(dead_code)]
pub fn occ_fill(map: &mut OcclusionMapExport, value: f32) {
    let v = value.clamp(0.0, 1.0);
    for d in map.data.iter_mut() { *d = v; }
}

#[allow(dead_code)]
pub fn occ_blur(map: &mut OcclusionMapExport) {
    let w = map.width as i32;
    let h = map.height as i32;
    let src = map.data.clone();
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            let mut count = 0u32;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let nx = x + dx;
                    let ny = y + dy;
                    if (0..w).contains(&nx) && (0..h).contains(&ny) {
                        sum += src[(ny * w + nx) as usize];
                        count += 1;
                    }
                }
            }
            map.data[(y * w + x) as usize] = sum / count as f32;
        }
    }
}

#[allow(dead_code)]
pub fn occ_to_bytes(map: &OcclusionMapExport) -> Vec<u8> {
    map.data.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect()
}

#[allow(dead_code)]
pub fn occ_to_json(map: &OcclusionMapExport) -> String {
    format!(r#"{{"width":{},"height":{},"avg":{:.4}}}"#, map.width, map.height, occ_average(map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_occlusion_map(4, 4);
        assert_eq!(occ_pixel_count(&m), 16);
    }

    #[test]
    fn test_default_white() {
        let m = new_occlusion_map(2, 2);
        assert!((occ_get_pixel(&m, 0, 0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_get() {
        let mut m = new_occlusion_map(2, 2);
        occ_set_pixel(&mut m, 1, 1, 0.5);
        assert!((occ_get_pixel(&m, 1, 1) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_average() {
        let m = new_occlusion_map(2, 2);
        assert!((occ_average(&m) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fill() {
        let mut m = new_occlusion_map(3, 3);
        occ_fill(&mut m, 0.3);
        assert!((occ_average(&m) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_blur() {
        let mut m = new_occlusion_map(3, 3);
        occ_set_pixel(&mut m, 1, 1, 0.0);
        occ_blur(&mut m);
        // Center should be affected
        let center = occ_get_pixel(&m, 1, 1);
        assert!(center > 0.0 && center < 1.0);
    }

    #[test]
    fn test_to_bytes() {
        let m = new_occlusion_map(1, 1);
        let bytes = occ_to_bytes(&m);
        assert_eq!(bytes[0], 255);
    }

    #[test]
    fn test_to_json() {
        let m = new_occlusion_map(4, 4);
        let json = occ_to_json(&m);
        assert!(json.contains("width"));
    }

    #[test]
    fn test_clamp() {
        let mut m = new_occlusion_map(1, 1);
        occ_set_pixel(&mut m, 0, 0, -1.0);
        assert!((occ_get_pixel(&m, 0, 0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_out_of_bounds() {
        let m = new_occlusion_map(2, 2);
        assert!((occ_get_pixel(&m, 99, 99) - 1.0).abs() < 1e-5);
    }

}
