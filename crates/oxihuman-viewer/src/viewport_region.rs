// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ViewportRegion — rectangular viewport region utilities.

#![allow(dead_code)]

/// A rectangular region in pixel coordinates.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ViewportRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Create a new viewport region.
#[allow(dead_code)]
pub fn new_viewport_region(x: f32, y: f32, width: f32, height: f32) -> ViewportRegion {
    ViewportRegion { x, y, width, height }
}

/// Width of the region.
#[allow(dead_code)]
pub fn region_width(region: &ViewportRegion) -> f32 {
    region.width
}

/// Height of the region.
#[allow(dead_code)]
pub fn region_height(region: &ViewportRegion) -> f32 {
    region.height
}

/// Aspect ratio (width / height). Returns 0.0 if height is zero.
#[allow(dead_code)]
pub fn region_aspect(region: &ViewportRegion) -> f32 {
    if region.height.abs() < 1e-9 {
        return 0.0;
    }
    region.width / region.height
}

/// Test whether a point `(px, py)` lies within the region.
#[allow(dead_code)]
pub fn region_contains_point(region: &ViewportRegion, px: f32, py: f32) -> bool {
    let x_range = region.x..=(region.x + region.width);
    let y_range = region.y..=(region.y + region.height);
    x_range.contains(&px) && y_range.contains(&py)
}

/// Return the center of the region.
#[allow(dead_code)]
pub fn region_center(region: &ViewportRegion) -> [f32; 2] {
    [region.x + region.width * 0.5, region.y + region.height * 0.5]
}

/// Convert pixel coordinate `(px, py)` to NDC `[-1, 1]` relative to this region.
#[allow(dead_code)]
pub fn region_to_ndc(region: &ViewportRegion, px: f32, py: f32) -> [f32; 2] {
    if region.width.abs() < 1e-9 || region.height.abs() < 1e-9 {
        return [0.0, 0.0];
    }
    let nx = ((px - region.x) / region.width) * 2.0 - 1.0;
    let ny = ((py - region.y) / region.height) * 2.0 - 1.0;
    [nx, ny]
}

/// Compute the overlapping area between two regions. Returns 0.0 if no overlap.
#[allow(dead_code)]
pub fn region_overlap(a: &ViewportRegion, b: &ViewportRegion) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    let w = (x2 - x1).max(0.0);
    let h = (y2 - y1).max(0.0);
    w * h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_viewport_region() {
        let r = new_viewport_region(10.0, 20.0, 100.0, 200.0);
        assert!((r.x - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_width() {
        let r = new_viewport_region(0.0, 0.0, 800.0, 600.0);
        assert!((region_width(&r) - 800.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_height() {
        let r = new_viewport_region(0.0, 0.0, 800.0, 600.0);
        assert!((region_height(&r) - 600.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_aspect() {
        let r = new_viewport_region(0.0, 0.0, 800.0, 400.0);
        assert!((region_aspect(&r) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_aspect_zero_height() {
        let r = new_viewport_region(0.0, 0.0, 800.0, 0.0);
        assert!((region_aspect(&r) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_contains_point_inside() {
        let r = new_viewport_region(0.0, 0.0, 100.0, 100.0);
        assert!(region_contains_point(&r, 50.0, 50.0));
    }

    #[test]
    fn test_region_contains_point_outside() {
        let r = new_viewport_region(0.0, 0.0, 100.0, 100.0);
        assert!(!region_contains_point(&r, 150.0, 50.0));
    }

    #[test]
    fn test_region_center() {
        let r = new_viewport_region(0.0, 0.0, 100.0, 200.0);
        let c = region_center(&r);
        assert!((c[0] - 50.0).abs() < 1e-6);
        assert!((c[1] - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_to_ndc_center() {
        let r = new_viewport_region(0.0, 0.0, 100.0, 100.0);
        let ndc = region_to_ndc(&r, 50.0, 50.0);
        assert!((ndc[0] - 0.0).abs() < 1e-6);
        assert!((ndc[1] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_region_overlap() {
        let a = new_viewport_region(0.0, 0.0, 100.0, 100.0);
        let b = new_viewport_region(50.0, 50.0, 100.0, 100.0);
        let area = region_overlap(&a, &b);
        assert!((area - 2500.0).abs() < 1e-3);
    }
}
