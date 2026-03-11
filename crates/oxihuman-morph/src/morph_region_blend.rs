// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Region-based morph blending with spatial falloff.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphRegion {
    pub center: [f32; 3],
    pub radius: f32,
    pub morph_target: usize,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphRegionBlend {
    pub regions: Vec<MorphRegion>,
}

#[allow(dead_code)]
pub fn new_morph_region_blend() -> MorphRegionBlend {
    MorphRegionBlend { regions: Vec::new() }
}

#[allow(dead_code)]
pub fn mrb_add_region(
    blend: &mut MorphRegionBlend,
    center: [f32; 3],
    radius: f32,
    target: usize,
    weight: f32,
) {
    blend.regions.push(MorphRegion { center, radius, morph_target: target, weight });
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn mrb_weight_at(blend: &MorphRegionBlend, pos: [f32; 3]) -> f32 {
    let mut max_w = 0.0_f32;
    for region in &blend.regions {
        let d = dist3(pos, region.center);
        if d < region.radius {
            /* linear falloff */
            let t = 1.0 - d / region.radius;
            let w = t * region.weight;
            if w > max_w {
                max_w = w;
            }
        }
    }
    max_w
}

#[allow(dead_code)]
pub fn mrb_region_count(blend: &MorphRegionBlend) -> usize {
    blend.regions.len()
}

#[allow(dead_code)]
pub fn mrb_target_at(blend: &MorphRegionBlend, pos: [f32; 3]) -> Option<usize> {
    let mut best_w = 0.0_f32;
    let mut best_target = None;
    for region in &blend.regions {
        let d = dist3(pos, region.center);
        if d < region.radius {
            let t = 1.0 - d / region.radius;
            let w = t * region.weight;
            if w > best_w {
                best_w = w;
                best_target = Some(region.morph_target);
            }
        }
    }
    best_target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_region_count() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 1.0, 0, 1.0);
        assert_eq!(mrb_region_count(&b), 1);
    }

    #[test]
    fn test_weight_at_center_is_max() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 1.0, 0, 1.0);
        let w = mrb_weight_at(&b, [0.0, 0.0, 0.0]);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_weight_at_outside_is_zero() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 1.0, 0, 1.0);
        let w = mrb_weight_at(&b, [2.0, 0.0, 0.0]);
        assert_eq!(w, 0.0);
    }

    #[test]
    fn test_weight_at_edge_nonzero() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 2.0, 0, 1.0);
        let w = mrb_weight_at(&b, [1.0, 0.0, 0.0]);
        assert!(w > 0.0 && w < 1.0);
    }

    #[test]
    fn test_target_at_inside() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 1.0, 42, 1.0);
        let t = mrb_target_at(&b, [0.0, 0.0, 0.0]);
        assert_eq!(t, Some(42));
    }

    #[test]
    fn test_target_at_outside_none() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 1.0, 0, 1.0);
        let t = mrb_target_at(&b, [5.0, 0.0, 0.0]);
        assert_eq!(t, None);
    }

    #[test]
    fn test_multiple_regions_max_weight() {
        let mut b = new_morph_region_blend();
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 2.0, 0, 0.5);
        mrb_add_region(&mut b, [0.0, 0.0, 0.0], 2.0, 1, 0.9);
        let w = mrb_weight_at(&b, [0.0, 0.0, 0.0]);
        assert!((w - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_empty_blend_zero_weight() {
        let b = new_morph_region_blend();
        assert_eq!(mrb_weight_at(&b, [0.0, 0.0, 0.0]), 0.0);
    }
}
