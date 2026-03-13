// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV chart packing into a texture atlas.

/// An axis-aligned rectangle in UV space.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ChartRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// A packed chart with its final atlas position.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PackedChart {
    pub original_index: usize,
    pub rect: ChartRect,
}

/// Result of a chart packing pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChartPackResult {
    pub packed: Vec<PackedChart>,
    pub atlas_size: f32,
    pub utilization: f32,
}

/// Check if two rects overlap.
#[allow(dead_code)]
pub fn rects_overlap_cp(a: ChartRect, b: ChartRect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

/// Compute total area of a list of rects.
#[allow(dead_code)]
pub fn total_chart_area(charts: &[ChartRect]) -> f32 {
    charts.iter().map(|c| c.w * c.h).sum()
}

/// Pack charts using a shelf-first-fit algorithm into a square atlas.
/// Charts are sorted by height descending.
#[allow(dead_code)]
pub fn pack_charts(charts: &[ChartRect], padding: f32) -> ChartPackResult {
    if charts.is_empty() {
        return ChartPackResult {
            packed: vec![],
            atlas_size: 0.0,
            utilization: 0.0,
        };
    }
    // Sort by height desc
    let mut order: Vec<usize> = (0..charts.len()).collect();
    order.sort_by(|&a, &b| {
        charts[b]
            .h
            .partial_cmp(&charts[a].h)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Estimate atlas size
    let total_area = total_chart_area(charts);
    let mut atlas = (total_area.sqrt() * 1.2).max(1.0);
    // Ensure atlas is at least as large as the widest chart
    let max_w = charts.iter().map(|c| c.w + padding).fold(0.0f32, f32::max);
    atlas = atlas.max(max_w);

    let mut packed = Vec::with_capacity(charts.len());
    let mut cursor_x = padding;
    let mut cursor_y = padding;
    let mut shelf_h = 0.0f32;

    for &oi in &order {
        let c = charts[oi];
        let pw = c.w + padding;
        let ph = c.h + padding;
        if cursor_x + pw > atlas {
            cursor_x = padding;
            cursor_y += shelf_h + padding;
            shelf_h = 0.0;
        }
        packed.push(PackedChart {
            original_index: oi,
            rect: ChartRect {
                x: cursor_x,
                y: cursor_y,
                w: c.w,
                h: c.h,
            },
        });
        cursor_x += pw;
        shelf_h = shelf_h.max(ph);
    }

    let atlas_area = atlas * atlas;
    let used_area = total_chart_area(charts);
    let utilization = if atlas_area > 0.0 {
        used_area / atlas_area
    } else {
        0.0
    };
    ChartPackResult {
        packed,
        atlas_size: atlas,
        utilization,
    }
}

/// Compute utilization given a result.
#[allow(dead_code)]
pub fn chart_utilization(result: &ChartPackResult) -> f32 {
    result.utilization
}

/// Serialize chart pack result to JSON.
#[allow(dead_code)]
pub fn chart_pack_to_json(result: &ChartPackResult) -> String {
    format!(
        "{{\"charts\":{},\"atlas_size\":{:.2},\"utilization\":{:.4}}}",
        result.packed.len(),
        result.atlas_size,
        result.utilization
    )
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct UvChart {
    pub uvs: Vec<[f32; 2]>,
    pub chart_id: usize,
}

pub struct PackingResult {
    pub charts: Vec<UvChart>,
    pub atlas_utilization: f32,
}

pub fn new_uv_chart(id: usize, uvs: Vec<[f32; 2]>) -> UvChart {
    UvChart { uvs, chart_id: id }
}

pub fn chart_bounding_box(chart: &UvChart) -> ([f32; 2], [f32; 2]) {
    if chart.uvs.is_empty() {
        return ([0.0; 2], [0.0; 2]);
    }
    let mut min = chart.uvs[0];
    let mut max = chart.uvs[0];
    for &uv in &chart.uvs {
        min[0] = min[0].min(uv[0]);
        min[1] = min[1].min(uv[1]);
        max[0] = max[0].max(uv[0]);
        max[1] = max[1].max(uv[1]);
    }
    (min, max)
}

pub fn chart_area(chart: &UvChart) -> f32 {
    let (min, max) = chart_bounding_box(chart);
    (max[0] - min[0]) * (max[1] - min[1])
}

pub fn pack_charts_stub(charts: Vec<UvChart>, atlas_size: f32) -> PackingResult {
    /* naive row packing: place charts in sequence */
    let total_area: f32 = charts.iter().map(chart_area).sum();
    let atlas_area = atlas_size * atlas_size;
    let utilization = if atlas_area > 0.0 {
        (total_area / atlas_area).min(1.0)
    } else {
        0.0
    };
    PackingResult {
        charts,
        atlas_utilization: utilization,
    }
}

pub fn packing_utilization(result: &PackingResult) -> f32 {
    result.atlas_utilization
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_charts() -> Vec<ChartRect> {
        vec![
            ChartRect {
                x: 0.0,
                y: 0.0,
                w: 0.2,
                h: 0.2,
            },
            ChartRect {
                x: 0.0,
                y: 0.0,
                w: 0.3,
                h: 0.1,
            },
        ]
    }

    #[test]
    fn test_rects_overlap() {
        let a = ChartRect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        };
        let b = ChartRect {
            x: 0.5,
            y: 0.5,
            w: 1.0,
            h: 1.0,
        };
        assert!(rects_overlap_cp(a, b));
    }

    #[test]
    fn test_rects_no_overlap() {
        let a = ChartRect {
            x: 0.0,
            y: 0.0,
            w: 0.4,
            h: 0.4,
        };
        let b = ChartRect {
            x: 0.5,
            y: 0.5,
            w: 0.4,
            h: 0.4,
        };
        assert!(!rects_overlap_cp(a, b));
    }

    #[test]
    fn test_total_chart_area() {
        let charts = small_charts();
        let area = total_chart_area(&charts);
        assert!((area - 0.04 - 0.03).abs() < 1e-5);
    }

    #[test]
    fn test_pack_charts_count() {
        let charts = small_charts();
        let result = pack_charts(&charts, 0.01);
        assert_eq!(result.packed.len(), 2);
    }

    #[test]
    fn test_pack_charts_utilization_positive() {
        let charts = small_charts();
        let result = pack_charts(&charts, 0.01);
        assert!(chart_utilization(&result) > 0.0);
    }

    #[test]
    fn test_pack_charts_empty() {
        let result = pack_charts(&[], 0.01);
        assert_eq!(result.packed.len(), 0);
    }

    #[test]
    fn test_pack_charts_atlas_size_positive() {
        let charts = small_charts();
        let result = pack_charts(&charts, 0.01);
        assert!(result.atlas_size > 0.0);
    }

    #[test]
    fn test_chart_pack_to_json() {
        let charts = small_charts();
        let result = pack_charts(&charts, 0.01);
        let j = chart_pack_to_json(&result);
        assert!(j.contains("atlas_size"));
    }

    #[test]
    fn test_packed_indices_valid() {
        let charts = small_charts();
        let result = pack_charts(&charts, 0.01);
        for p in &result.packed {
            assert!(p.original_index < charts.len());
        }
    }

    #[test]
    fn test_utilization_leq_one() {
        let charts = vec![ChartRect {
            x: 0.0,
            y: 0.0,
            w: 0.5,
            h: 0.5,
        }];
        let result = pack_charts(&charts, 0.0);
        assert!(chart_utilization(&result) <= 1.01); // allow slight fp slack
    }
}
