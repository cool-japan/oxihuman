// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Index buffer debug view — visualizes IBO binding, format, and triangle counts.

/// Index buffer view configuration.
#[derive(Debug, Clone)]
pub struct IndexBufferView {
    pub enabled: bool,
    pub total_ibos: u32,
    pub total_indices: u64,
    pub index_format_bits: u8,
    pub show_primitive_count: bool,
}

impl IndexBufferView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            total_ibos: 0,
            total_indices: 0,
            index_format_bits: 32,
            show_primitive_count: true,
        }
    }
}

impl Default for IndexBufferView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new index buffer view.
pub fn new_index_buffer_view() -> IndexBufferView {
    IndexBufferView::new()
}

/// Enable or disable index buffer debug overlay.
pub fn ibv_set_enabled(v: &mut IndexBufferView, enabled: bool) {
    v.enabled = enabled;
}

/// Update IBO count and total index count.
pub fn ibv_update_stats(v: &mut IndexBufferView, ibo_count: u32, total_indices: u64) {
    v.total_ibos = ibo_count;
    v.total_indices = total_indices;
}

/// Set index format bit width (16 or 32).
pub fn ibv_set_index_format(v: &mut IndexBufferView, bits: u8) {
    v.index_format_bits = if bits <= 16 { 16 } else { 32 };
}

/// Toggle primitive count display.
pub fn ibv_set_show_primitive_count(v: &mut IndexBufferView, show: bool) {
    v.show_primitive_count = show;
}

/// Compute triangle count from index count.
pub fn ibv_triangle_count(v: &IndexBufferView) -> u64 {
    v.total_indices / 3
}

/// Compute total buffer byte size.
pub fn ibv_total_bytes(v: &IndexBufferView) -> u64 {
    v.total_indices * (v.index_format_bits as u64 / 8)
}

/// Serialize to JSON-like string.
pub fn index_buffer_view_to_json(v: &IndexBufferView) -> String {
    format!(
        r#"{{"enabled":{},"total_ibos":{},"total_indices":{},"index_format_bits":{},"show_primitive_count":{}}}"#,
        v.enabled, v.total_ibos, v.total_indices, v.index_format_bits, v.show_primitive_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_index_buffer_view();
        assert!(!v.enabled);
        assert_eq!(v.index_format_bits, 32);
    }

    #[test]
    fn test_enable() {
        let mut v = new_index_buffer_view();
        ibv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_update_stats() {
        let mut v = new_index_buffer_view();
        ibv_update_stats(&mut v, 3, 30000);
        assert_eq!(v.total_ibos, 3);
        assert_eq!(v.total_indices, 30000);
    }

    #[test]
    fn test_format_16bit() {
        let mut v = new_index_buffer_view();
        ibv_set_index_format(&mut v, 16);
        assert_eq!(v.index_format_bits, 16);
    }

    #[test]
    fn test_format_defaults_to_32_for_odd() {
        let mut v = new_index_buffer_view();
        ibv_set_index_format(&mut v, 24);
        assert_eq!(v.index_format_bits, 32);
    }

    #[test]
    fn test_show_primitive_count_toggle() {
        let mut v = new_index_buffer_view();
        ibv_set_show_primitive_count(&mut v, false);
        assert!(!v.show_primitive_count);
    }

    #[test]
    fn test_triangle_count() {
        let mut v = new_index_buffer_view();
        ibv_update_stats(&mut v, 1, 9);
        assert_eq!(ibv_triangle_count(&v), 3);
    }

    #[test]
    fn test_total_bytes_32bit() {
        let mut v = new_index_buffer_view();
        ibv_update_stats(&mut v, 1, 100);
        assert_eq!(ibv_total_bytes(&v), 400);
    }

    #[test]
    fn test_json_keys() {
        let v = new_index_buffer_view();
        let s = index_buffer_view_to_json(&v);
        assert!(s.contains("index_format_bits"));
    }

    #[test]
    fn test_clone() {
        let v = new_index_buffer_view();
        let v2 = v.clone();
        assert_eq!(v2.index_format_bits, v.index_format_bits);
    }
}
