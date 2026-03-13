// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export heightfield/terrain data.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeightfieldExport {
    pub width: u32,
    pub depth: u32,
    pub cell_size: f32,
    pub heights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_heightfield_export(width: u32, depth: u32, cell_size: f32) -> HeightfieldExport {
    let n = (width * depth) as usize;
    HeightfieldExport { width, depth, cell_size: cell_size.max(0.01), heights: vec![0.0; n] }
}

#[allow(dead_code)]
pub fn hfe_set_height(hfe: &mut HeightfieldExport, x: u32, z: u32, h: f32) {
    if x < hfe.width && z < hfe.depth {
        let idx = (z * hfe.width + x) as usize;
        if idx < hfe.heights.len() { hfe.heights[idx] = h; }
    }
}

#[allow(dead_code)]
pub fn hfe_get_height(hfe: &HeightfieldExport, x: u32, z: u32) -> f32 {
    if x < hfe.width && z < hfe.depth {
        let idx = (z * hfe.width + x) as usize;
        if idx < hfe.heights.len() { return hfe.heights[idx]; }
    }
    0.0
}

#[allow(dead_code)]
pub fn hfe_min_height(hfe: &HeightfieldExport) -> f32 {
    hfe.heights.iter().copied().fold(f32::MAX, f32::min)
}

#[allow(dead_code)]
pub fn hfe_max_height(hfe: &HeightfieldExport) -> f32 {
    hfe.heights.iter().copied().fold(f32::MIN, f32::max)
}

#[allow(dead_code)]
pub fn hfe_world_size(hfe: &HeightfieldExport) -> [f32; 2] {
    [hfe.width as f32 * hfe.cell_size, hfe.depth as f32 * hfe.cell_size]
}

#[allow(dead_code)]
pub fn hfe_vertex_count(hfe: &HeightfieldExport) -> u32 { hfe.width * hfe.depth }

#[allow(dead_code)]
pub fn hfe_validate(hfe: &HeightfieldExport) -> bool {
    hfe.width > 0 && hfe.depth > 0 && hfe.heights.len() == (hfe.width * hfe.depth) as usize
}

#[allow(dead_code)]
pub fn hfe_to_json(hfe: &HeightfieldExport) -> String {
    format!("{{\"width\":{},\"depth\":{},\"cell_size\":{:.4},\"min_h\":{:.4},\"max_h\":{:.4}}}",
        hfe.width, hfe.depth, hfe.cell_size, hfe_min_height(hfe), hfe_max_height(hfe))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> HeightfieldExport {
        let mut h = new_heightfield_export(4, 4, 1.0);
        hfe_set_height(&mut h, 1, 1, 5.0);
        hfe_set_height(&mut h, 2, 2, 10.0);
        h
    }

    #[test] fn test_new() { let h = new_heightfield_export(8, 8, 0.5); assert_eq!(h.heights.len(), 64); }
    #[test] fn test_set_get() { let h = sample(); assert!((hfe_get_height(&h, 1, 1) - 5.0).abs() < 1e-5); }
    #[test] fn test_min() { let h = sample(); assert!((hfe_min_height(&h)).abs() < 1e-5); }
    #[test] fn test_max() { let h = sample(); assert!((hfe_max_height(&h) - 10.0).abs() < 1e-5); }
    #[test] fn test_world_size() { let h = sample(); let ws = hfe_world_size(&h); assert!((ws[0] - 4.0).abs() < 1e-5); }
    #[test] fn test_vertex_count() { assert_eq!(hfe_vertex_count(&sample()), 16); }
    #[test] fn test_validate() { assert!(hfe_validate(&sample())); }
    #[test] fn test_to_json() { assert!(hfe_to_json(&sample()).contains("cell_size")); }
    #[test] fn test_out_of_bounds() { let h = sample(); assert!((hfe_get_height(&h, 100, 100)).abs() < 1e-6); }
    #[test] fn test_cell_size() { let h = sample(); assert!((h.cell_size - 1.0).abs() < 1e-6); }
}
