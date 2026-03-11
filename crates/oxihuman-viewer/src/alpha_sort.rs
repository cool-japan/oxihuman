// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Alpha-transparent draw call sorting (back-to-front).

use std::f32::consts::FRAC_PI_4;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlphaSortEntry {
    pub id: u32,
    pub depth: f32,
    pub alpha: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AlphaSortBuffer {
    pub entries: Vec<AlphaSortEntry>,
}

#[allow(dead_code)]
pub fn new_alpha_sort_buffer() -> AlphaSortBuffer {
    AlphaSortBuffer::default()
}

#[allow(dead_code)]
pub fn as_push(buf: &mut AlphaSortBuffer, id: u32, depth: f32, alpha: f32) {
    let alpha = alpha.clamp(0.0, 1.0);
    buf.entries.push(AlphaSortEntry { id, depth, alpha });
}

#[allow(dead_code)]
pub fn as_sort_back_to_front(buf: &mut AlphaSortBuffer) {
    buf.entries.sort_by(|a, b| {
        b.depth
            .partial_cmp(&a.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn as_sort_front_to_back(buf: &mut AlphaSortBuffer) {
    buf.entries.sort_by(|a, b| {
        a.depth
            .partial_cmp(&b.depth)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn as_clear(buf: &mut AlphaSortBuffer) {
    buf.entries.clear();
}

#[allow(dead_code)]
pub fn as_count(buf: &AlphaSortBuffer) -> usize {
    buf.entries.len()
}

#[allow(dead_code)]
pub fn as_is_empty(buf: &AlphaSortBuffer) -> bool {
    buf.entries.is_empty()
}

#[allow(dead_code)]
pub fn as_average_alpha(buf: &AlphaSortBuffer) -> f32 {
    if buf.entries.is_empty() {
        return 0.0;
    }
    buf.entries.iter().map(|e| e.alpha).sum::<f32>() / buf.entries.len() as f32
}

#[allow(dead_code)]
pub fn as_max_depth(buf: &AlphaSortBuffer) -> f32 {
    buf.entries
        .iter()
        .map(|e| e.depth)
        .fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn as_depth_angle_rad(buf: &AlphaSortBuffer) -> f32 {
    as_average_alpha(buf) * FRAC_PI_4
}

#[allow(dead_code)]
pub fn as_to_json(buf: &AlphaSortBuffer) -> String {
    format!(
        "{{\"count\":{},\"avg_alpha\":{:.4}}}",
        as_count(buf),
        as_average_alpha(buf)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_is_empty() {
        assert!(as_is_empty(&new_alpha_sort_buffer()));
    }
    #[test]
    fn push_increments_count() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 0, 1.0, 0.5);
        assert_eq!(as_count(&b), 1);
    }
    #[test]
    fn alpha_clamps_to_one() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 0, 1.0, 5.0);
        assert!((0.0..=1.0).contains(&b.entries[0].alpha));
    }
    #[test]
    fn clear_empties() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 0, 1.0, 0.5);
        as_clear(&mut b);
        assert!(as_is_empty(&b));
    }
    #[test]
    fn back_to_front_order() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 1, 1.0, 0.5);
        as_push(&mut b, 2, 3.0, 0.5);
        as_sort_back_to_front(&mut b);
        assert!(b.entries[0].depth >= b.entries[1].depth);
    }
    #[test]
    fn front_to_back_order() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 1, 3.0, 0.5);
        as_push(&mut b, 2, 1.0, 0.5);
        as_sort_front_to_back(&mut b);
        assert!(b.entries[0].depth <= b.entries[1].depth);
    }
    #[test]
    fn average_alpha_empty_is_zero() {
        assert!(as_average_alpha(&new_alpha_sort_buffer()).abs() < 1e-6);
    }
    #[test]
    fn max_depth_after_push() {
        let mut b = new_alpha_sort_buffer();
        as_push(&mut b, 0, 7.5, 0.5);
        assert!((as_max_depth(&b) - 7.5).abs() < 1e-5);
    }
    #[test]
    fn depth_angle_nonneg() {
        assert!(as_depth_angle_rad(&new_alpha_sort_buffer()) >= 0.0);
    }
    #[test]
    fn to_json_has_count() {
        assert!(as_to_json(&new_alpha_sort_buffer()).contains("\"count\""));
    }
}
