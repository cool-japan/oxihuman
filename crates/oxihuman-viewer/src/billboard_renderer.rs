// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Billboard rendering utilities for 3D viewer.

/// Billboard alignment mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BillboardAlign {
    ScreenAligned,
    AxisAligned,
    ViewPlane,
}

/// A billboard instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Billboard {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub align: BillboardAlign,
}

/// Billboard batch.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BillboardBatch {
    pub billboards: Vec<Billboard>,
}

/// Create a new billboard.
#[allow(dead_code)]
pub fn new_billboard(position: [f32; 3], size: [f32; 2]) -> Billboard {
    Billboard {
        position,
        size,
        color: [1.0, 1.0, 1.0, 1.0],
        align: BillboardAlign::ScreenAligned,
    }
}

/// Create an empty batch.
#[allow(dead_code)]
pub fn new_billboard_batch() -> BillboardBatch {
    BillboardBatch {
        billboards: Vec::new(),
    }
}

/// Add a billboard to the batch.
#[allow(dead_code)]
pub fn add_billboard(batch: &mut BillboardBatch, billboard: Billboard) {
    batch.billboards.push(billboard);
}

/// Clear the batch.
#[allow(dead_code)]
pub fn clear_billboard_batch(batch: &mut BillboardBatch) {
    batch.billboards.clear();
}

/// Count billboards.
#[allow(dead_code)]
pub fn billboard_count(batch: &BillboardBatch) -> usize {
    batch.billboards.len()
}

/// Compute billboard screen area.
#[allow(dead_code)]
pub fn billboard_area(b: &Billboard) -> f32 {
    b.size[0] * b.size[1]
}

/// Sort billboards by distance from camera (back to front).
#[allow(dead_code)]
pub fn sort_billboards_by_distance(batch: &mut BillboardBatch, camera_pos: [f32; 3]) {
    batch.billboards.sort_by(|a, b| {
        let da = dist_sq(a.position, camera_pos);
        let db = dist_sq(b.position, camera_pos);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_billboard() {
        let b = new_billboard([0.0, 0.0, 0.0], [1.0, 1.0]);
        assert!((b.size[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_batch() {
        let batch = new_billboard_batch();
        assert!(batch.billboards.is_empty());
    }

    #[test]
    fn test_add_billboard() {
        let mut batch = new_billboard_batch();
        add_billboard(&mut batch, new_billboard([0.0, 0.0, 0.0], [1.0, 1.0]));
        assert_eq!(billboard_count(&batch), 1);
    }

    #[test]
    fn test_clear() {
        let mut batch = new_billboard_batch();
        add_billboard(&mut batch, new_billboard([0.0, 0.0, 0.0], [1.0, 1.0]));
        clear_billboard_batch(&mut batch);
        assert_eq!(billboard_count(&batch), 0);
    }

    #[test]
    fn test_area() {
        let b = new_billboard([0.0, 0.0, 0.0], [2.0, 3.0]);
        assert!((billboard_area(&b) - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_sort() {
        let mut batch = new_billboard_batch();
        add_billboard(&mut batch, new_billboard([0.0, 0.0, 1.0], [1.0, 1.0]));
        add_billboard(&mut batch, new_billboard([0.0, 0.0, 5.0], [1.0, 1.0]));
        sort_billboards_by_distance(&mut batch, [0.0, 0.0, 0.0]);
        assert!(batch.billboards[0].position[2] > batch.billboards[1].position[2]);
    }

    #[test]
    fn test_default_color() {
        let b = new_billboard([0.0, 0.0, 0.0], [1.0, 1.0]);
        assert!((b.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_default_align() {
        let b = new_billboard([0.0, 0.0, 0.0], [1.0, 1.0]);
        assert_eq!(b.align, BillboardAlign::ScreenAligned);
    }

    #[test]
    fn test_dist_sq() {
        let d = dist_sq([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((d - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_add() {
        let mut batch = new_billboard_batch();
        for i in 0..5 {
            add_billboard(&mut batch, new_billboard([i as f32, 0.0, 0.0], [1.0, 1.0]));
        }
        assert_eq!(billboard_count(&batch), 5);
    }
}
