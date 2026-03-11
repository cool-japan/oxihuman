// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GBuffer — geometry buffer for deferred shading.

#![allow(dead_code)]

/// Identifies which G-buffer slot to read/write.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GBufferSlot {
    Albedo,
    Normal,
    Roughness,
    Metallic,
}

/// A simple CPU-side G-buffer holding per-pixel RGBA data for each slot.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GBuffer {
    pub width: u32,
    pub height: u32,
    /// 4 slots × width × height × 4 channels (RGBA f32)
    pub data: Vec<Vec<f32>>,
}

/// Create a zeroed `GBuffer` with 4 slots.
#[allow(dead_code)]
pub fn new_gbuffer(width: u32, height: u32) -> GBuffer {
    let n = (width * height * 4) as usize;
    GBuffer { width, height, data: vec![vec![0.0; n]; 4] }
}

/// Write an RGBA value to the albedo slot at (x, y).
#[allow(dead_code)]
pub fn gbuffer_write_albedo(gb: &mut GBuffer, x: u32, y: u32, rgba: [f32; 4]) {
    gbuffer_write_slot(gb, GBufferSlot::Albedo, x, y, rgba);
}

/// Write an RGBA value to the normal slot at (x, y).
#[allow(dead_code)]
pub fn gbuffer_write_normal(gb: &mut GBuffer, x: u32, y: u32, rgba: [f32; 4]) {
    gbuffer_write_slot(gb, GBufferSlot::Normal, x, y, rgba);
}

/// Write a roughness value to the roughness slot at (x, y).
#[allow(dead_code)]
pub fn gbuffer_write_roughness(gb: &mut GBuffer, x: u32, y: u32, value: f32) {
    gbuffer_write_slot(gb, GBufferSlot::Roughness, x, y, [value, 0.0, 0.0, 0.0]);
}

fn gbuffer_write_slot(gb: &mut GBuffer, slot: GBufferSlot, x: u32, y: u32, rgba: [f32; 4]) {
    let idx = slot as usize;
    let base = ((y * gb.width + x) * 4) as usize;
    if idx < gb.data.len() && base + 3 < gb.data[idx].len() {
        gb.data[idx][base..base + 4].copy_from_slice(&rgba);
    }
}

/// Read an RGBA value from the given slot at (x, y).
#[allow(dead_code)]
pub fn gbuffer_read(gb: &GBuffer, slot: GBufferSlot, x: u32, y: u32) -> [f32; 4] {
    let idx = slot as usize;
    let base = ((y * gb.width + x) * 4) as usize;
    if idx < gb.data.len() && base + 3 < gb.data[idx].len() {
        [gb.data[idx][base], gb.data[idx][base + 1], gb.data[idx][base + 2], gb.data[idx][base + 3]]
    } else {
        [0.0; 4]
    }
}

/// Clear all slots to zero.
#[allow(dead_code)]
pub fn gbuffer_clear(gb: &mut GBuffer) {
    for slot in &mut gb.data {
        slot.fill(0.0);
    }
}

/// Return the number of G-buffer slots (always 4).
#[allow(dead_code)]
pub fn gbuffer_slot_count(_gb: &GBuffer) -> usize {
    4
}

/// Return (width, height).
#[allow(dead_code)]
pub fn gbuffer_dimensions(gb: &GBuffer) -> (u32, u32) {
    (gb.width, gb.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gbuffer_zeroed() {
        let gb = new_gbuffer(2, 2);
        assert!(gb.data.iter().all(|slot| slot.iter().all(|&v| v == 0.0)));
    }

    #[test]
    fn test_gbuffer_dimensions() {
        let gb = new_gbuffer(8, 6);
        assert_eq!(gbuffer_dimensions(&gb), (8, 6));
    }

    #[test]
    fn test_gbuffer_slot_count() {
        let gb = new_gbuffer(1, 1);
        assert_eq!(gbuffer_slot_count(&gb), 4);
    }

    #[test]
    fn test_write_read_albedo() {
        let mut gb = new_gbuffer(4, 4);
        gbuffer_write_albedo(&mut gb, 1, 1, [1.0, 0.5, 0.25, 1.0]);
        let v = gbuffer_read(&gb, GBufferSlot::Albedo, 1, 1);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!((v[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_write_read_normal() {
        let mut gb = new_gbuffer(4, 4);
        gbuffer_write_normal(&mut gb, 0, 0, [0.0, 1.0, 0.0, 0.0]);
        let v = gbuffer_read(&gb, GBufferSlot::Normal, 0, 0);
        assert!((v[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_write_roughness() {
        let mut gb = new_gbuffer(4, 4);
        gbuffer_write_roughness(&mut gb, 2, 2, 0.8);
        let v = gbuffer_read(&gb, GBufferSlot::Roughness, 2, 2);
        assert!((v[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_gbuffer_clear() {
        let mut gb = new_gbuffer(2, 2);
        gbuffer_write_albedo(&mut gb, 0, 0, [1.0, 1.0, 1.0, 1.0]);
        gbuffer_clear(&mut gb);
        let v = gbuffer_read(&gb, GBufferSlot::Albedo, 0, 0);
        assert!(v.iter().all(|&c| c == 0.0));
    }

    #[test]
    fn test_gbuffer_slot_enum_distinct() {
        assert_ne!(GBufferSlot::Albedo, GBufferSlot::Normal);
        assert_ne!(GBufferSlot::Roughness, GBufferSlot::Metallic);
    }

    #[test]
    fn test_gbuffer_out_of_bounds_read_returns_zero() {
        let gb = new_gbuffer(2, 2);
        let v = gbuffer_read(&gb, GBufferSlot::Albedo, 100, 100);
        assert!(v.iter().all(|&c| c == 0.0));
    }
}
