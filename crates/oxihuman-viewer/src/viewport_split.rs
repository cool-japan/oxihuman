// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Direction of a viewport split.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// A viewport region.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct ViewportRegionSplit {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Manages viewport splitting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ViewportSplit {
    regions: Vec<ViewportRegionSplit>,
}

/// Create a new single-viewport split.
#[allow(dead_code)]
pub fn new_viewport_split(width: f32, height: f32) -> ViewportSplit {
    ViewportSplit {
        regions: vec![ViewportRegionSplit {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }],
    }
}

/// Split the first region horizontally (top/bottom).
#[allow(dead_code)]
pub fn split_horizontal(vs: &mut ViewportSplit, ratio: f32) {
    if vs.regions.is_empty() {
        return;
    }
    let r = vs.regions[0];
    let ratio = ratio.clamp(0.1, 0.9);
    let h1 = r.height * ratio;
    let h2 = r.height - h1;
    vs.regions[0] = ViewportRegionSplit {
        x: r.x,
        y: r.y,
        width: r.width,
        height: h1,
    };
    vs.regions.push(ViewportRegionSplit {
        x: r.x,
        y: r.y + h1,
        width: r.width,
        height: h2,
    });
}

/// Split the first region vertically (left/right).
#[allow(dead_code)]
pub fn split_vertical(vs: &mut ViewportSplit, ratio: f32) {
    if vs.regions.is_empty() {
        return;
    }
    let r = vs.regions[0];
    let ratio = ratio.clamp(0.1, 0.9);
    let w1 = r.width * ratio;
    let w2 = r.width - w1;
    vs.regions[0] = ViewportRegionSplit {
        x: r.x,
        y: r.y,
        width: w1,
        height: r.height,
    };
    vs.regions.push(ViewportRegionSplit {
        x: r.x + w1,
        y: r.y,
        width: w2,
        height: r.height,
    });
}

/// Return the number of viewport regions.
#[allow(dead_code)]
pub fn split_count(vs: &ViewportSplit) -> usize {
    vs.regions.len()
}

/// Return the region at the given index as (x, y, w, h).
#[allow(dead_code)]
pub fn viewport_at(vs: &ViewportSplit, index: usize) -> (f32, f32, f32, f32) {
    vs.regions
        .get(index)
        .map_or((0.0, 0.0, 0.0, 0.0), |r| (r.x, r.y, r.width, r.height))
}

/// Resize the entire viewport.
#[allow(dead_code)]
pub fn viewport_resize(vs: &mut ViewportSplit, width: f32, height: f32) {
    if vs.regions.is_empty() {
        return;
    }
    let old_w = vs.regions.iter().map(|r| r.x + r.width).fold(0.0_f32, f32::max);
    let old_h = vs.regions.iter().map(|r| r.y + r.height).fold(0.0_f32, f32::max);
    if old_w < 1e-9 || old_h < 1e-9 {
        return;
    }
    let sx = width / old_w;
    let sy = height / old_h;
    for r in &mut vs.regions {
        r.x *= sx;
        r.y *= sy;
        r.width *= sx;
        r.height *= sy;
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn split_to_json(vs: &ViewportSplit) -> String {
    let regions: Vec<String> = vs
        .regions
        .iter()
        .map(|r| {
            format!(
                "{{\"x\":{:.2},\"y\":{:.2},\"w\":{:.2},\"h\":{:.2}}}",
                r.x, r.y, r.width, r.height
            )
        })
        .collect();
    format!("{{\"regions\":[{}]}}", regions.join(","))
}

/// Reset to a single viewport.
#[allow(dead_code)]
pub fn split_reset(vs: &mut ViewportSplit, width: f32, height: f32) {
    vs.regions = vec![ViewportRegionSplit {
        x: 0.0,
        y: 0.0,
        width,
        height,
    }];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_single() {
        let vs = new_viewport_split(800.0, 600.0);
        assert_eq!(split_count(&vs), 1);
    }

    #[test]
    fn split_h() {
        let mut vs = new_viewport_split(800.0, 600.0);
        split_horizontal(&mut vs, 0.5);
        assert_eq!(split_count(&vs), 2);
    }

    #[test]
    fn split_v() {
        let mut vs = new_viewport_split(800.0, 600.0);
        split_vertical(&mut vs, 0.5);
        assert_eq!(split_count(&vs), 2);
    }

    #[test]
    fn viewport_at_first() {
        let vs = new_viewport_split(800.0, 600.0);
        let (x, y, w, h) = viewport_at(&vs, 0);
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
        assert!((w - 800.0).abs() < 1e-4);
        assert!((h - 600.0).abs() < 1e-4);
    }

    #[test]
    fn viewport_at_invalid() {
        let vs = new_viewport_split(800.0, 600.0);
        let (_, _, w, h) = viewport_at(&vs, 99);
        assert!(w.abs() < 1e-6);
        assert!(h.abs() < 1e-6);
    }

    #[test]
    fn resize() {
        let mut vs = new_viewport_split(800.0, 600.0);
        viewport_resize(&mut vs, 1600.0, 1200.0);
        let (_, _, w, h) = viewport_at(&vs, 0);
        assert!((w - 1600.0).abs() < 1e-2);
        assert!((h - 1200.0).abs() < 1e-2);
    }

    #[test]
    fn to_json() {
        let vs = new_viewport_split(100.0, 100.0);
        let j = split_to_json(&vs);
        assert!(j.contains("\"regions\""));
    }

    #[test]
    fn reset_to_single() {
        let mut vs = new_viewport_split(800.0, 600.0);
        split_horizontal(&mut vs, 0.5);
        split_reset(&mut vs, 1024.0, 768.0);
        assert_eq!(split_count(&vs), 1);
    }

    #[test]
    fn split_h_preserves_width() {
        let mut vs = new_viewport_split(800.0, 600.0);
        split_horizontal(&mut vs, 0.5);
        let (_, _, w1, _) = viewport_at(&vs, 0);
        let (_, _, w2, _) = viewport_at(&vs, 1);
        assert!((w1 - 800.0).abs() < 1e-4);
        assert!((w2 - 800.0).abs() < 1e-4);
    }

    #[test]
    fn split_v_preserves_height() {
        let mut vs = new_viewport_split(800.0, 600.0);
        split_vertical(&mut vs, 0.5);
        let (_, _, _, h1) = viewport_at(&vs, 0);
        let (_, _, _, h2) = viewport_at(&vs, 1);
        assert!((h1 - 600.0).abs() < 1e-4);
        assert!((h2 - 600.0).abs() < 1e-4);
    }
}
