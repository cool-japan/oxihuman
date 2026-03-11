// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! UV atlas utility v2: pack multiple UV charts.

#[allow(dead_code)]
pub struct UVChart {
    pub id: usize,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

#[allow(dead_code)]
pub struct UVAtlasV2 {
    pub charts: Vec<UVChart>,
    pub atlas_size: u32,
}

#[allow(dead_code)]
pub fn new_uv_atlas_v2(atlas_size: u32) -> UVAtlasV2 {
    UVAtlasV2 { charts: Vec::new(), atlas_size }
}

#[allow(dead_code)]
pub fn uva2_add_chart(atlas: &mut UVAtlasV2, width: f32, height: f32) -> usize {
    let s = atlas.atlas_size as f32;
    let w = width.min(s);
    let h = height.min(s);
    /* Simple row packing: place next to the last chart in normalized coords */
    let x_offset: f32 = atlas.charts.iter().map(|c| c.uv_max[0]).fold(0.0f32, f32::max);
    let (x0, y0) = if x_offset + w / s <= 1.0 {
        (x_offset, 0.0f32)
    } else {
        let y_used: f32 = atlas.charts.iter().map(|c| c.uv_max[1]).fold(0.0f32, f32::max);
        (0.0, y_used)
    };
    let id = atlas.charts.len();
    atlas.charts.push(UVChart {
        id,
        uv_min: [x0, y0],
        uv_max: [x0 + w / s, y0 + h / s],
    });
    id
}

#[allow(dead_code)]
pub fn uva2_chart_count(atlas: &UVAtlasV2) -> usize {
    atlas.charts.len()
}

#[allow(dead_code)]
pub fn uva2_utilization(atlas: &UVAtlasV2) -> f32 {
    atlas.charts.iter().map(|c| {
        let dw = c.uv_max[0] - c.uv_min[0];
        let dh = c.uv_max[1] - c.uv_min[1];
        dw * dh
    }).sum()
}

#[allow(dead_code)]
pub fn uva2_has_overlap(atlas: &UVAtlasV2) -> bool {
    let charts = &atlas.charts;
    for i in 0..charts.len() {
        for j in (i + 1)..charts.len() {
            let a = &charts[i];
            let b = &charts[j];
            let overlap_x = a.uv_max[0] > b.uv_min[0] + 1e-6 && b.uv_max[0] > a.uv_min[0] + 1e-6;
            let overlap_y = a.uv_max[1] > b.uv_min[1] + 1e-6 && b.uv_max[1] > a.uv_min[1] + 1e-6;
            if overlap_x && overlap_y {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let a = new_uv_atlas_v2(1024);
        assert_eq!(uva2_chart_count(&a), 0);
    }

    #[test]
    fn test_add_chart() {
        let mut a = new_uv_atlas_v2(512);
        let id = uva2_add_chart(&mut a, 100.0, 100.0);
        assert_eq!(id, 0);
        assert_eq!(uva2_chart_count(&a), 1);
    }

    #[test]
    fn test_utilization_positive() {
        let mut a = new_uv_atlas_v2(512);
        uva2_add_chart(&mut a, 100.0, 100.0);
        assert!(uva2_utilization(&a) > 0.0);
    }

    #[test]
    fn test_no_overlap_single() {
        let mut a = new_uv_atlas_v2(512);
        uva2_add_chart(&mut a, 100.0, 100.0);
        assert!(!uva2_has_overlap(&a));
    }

    #[test]
    fn test_multiple_charts_count() {
        let mut a = new_uv_atlas_v2(1024);
        for _ in 0..5 {
            uva2_add_chart(&mut a, 100.0, 100.0);
        }
        assert_eq!(uva2_chart_count(&a), 5);
    }

    #[test]
    fn test_utilization_zero_on_empty() {
        let a = new_uv_atlas_v2(512);
        assert_eq!(uva2_utilization(&a), 0.0);
    }

    #[test]
    fn test_overlap_check_two_side_by_side() {
        let mut a = new_uv_atlas_v2(1024);
        uva2_add_chart(&mut a, 100.0, 100.0);
        uva2_add_chart(&mut a, 100.0, 100.0);
        /* Two charts placed side by side should not overlap */
        let overlap = uva2_has_overlap(&a);
        assert!(!overlap);
    }

    #[test]
    fn test_chart_id_increments() {
        let mut a = new_uv_atlas_v2(1024);
        let id0 = uva2_add_chart(&mut a, 50.0, 50.0);
        let id1 = uva2_add_chart(&mut a, 50.0, 50.0);
        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
    }
}
