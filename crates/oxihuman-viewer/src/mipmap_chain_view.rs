// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mipmap chain visualization — overlays mip level selection on screen.

/// Mipmap chain view configuration.
#[derive(Debug, Clone)]
pub struct MipmapChainView {
    pub enabled: bool,
    pub base_width: u32,
    pub base_height: u32,
    pub mip_levels: u32,
    pub show_level_borders: bool,
}

impl MipmapChainView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            base_width: 1024,
            base_height: 1024,
            mip_levels: 0,
            show_level_borders: false,
        }
    }
}

impl Default for MipmapChainView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new mipmap chain view.
pub fn new_mipmap_chain_view() -> MipmapChainView {
    MipmapChainView::new()
}

/// Enable or disable mipmap chain overlay.
pub fn mmv_set_enabled(v: &mut MipmapChainView, enabled: bool) {
    v.enabled = enabled;
}

/// Set base texture dimensions and auto-compute mip level count.
pub fn mmv_set_base_dimensions(v: &mut MipmapChainView, width: u32, height: u32) {
    v.base_width = width.max(1);
    v.base_height = height.max(1);
    let max_dim = v.base_width.max(v.base_height);
    v.mip_levels = (max_dim as f32).log2().floor() as u32 + 1;
}

/// Toggle mip level border display.
pub fn mmv_set_show_level_borders(v: &mut MipmapChainView, show: bool) {
    v.show_level_borders = show;
}

/// Compute dimensions at a given mip level.
pub fn mmv_level_dimensions(v: &MipmapChainView, level: u32) -> (u32, u32) {
    let shift = level.min(31);
    let w = (v.base_width >> shift).max(1);
    let h = (v.base_height >> shift).max(1);
    (w, h)
}

/// Total texel count across all mip levels.
pub fn mmv_total_texels(v: &MipmapChainView) -> u64 {
    let mut total: u64 = 0;
    for lvl in 0..v.mip_levels {
        let (w, h) = mmv_level_dimensions(v, lvl);
        total = total.saturating_add(w as u64 * h as u64);
    }
    total
}

/// Serialize to JSON-like string.
pub fn mipmap_chain_view_to_json(v: &MipmapChainView) -> String {
    format!(
        r#"{{"enabled":{},"base_width":{},"base_height":{},"mip_levels":{},"show_level_borders":{}}}"#,
        v.enabled, v.base_width, v.base_height, v.mip_levels, v.show_level_borders
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_mipmap_chain_view();
        assert!(!v.enabled);
        assert_eq!(v.base_width, 1024);
    }

    #[test]
    fn test_enable() {
        let mut v = new_mipmap_chain_view();
        mmv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_set_base_dimensions_mip_count() {
        let mut v = new_mipmap_chain_view();
        mmv_set_base_dimensions(&mut v, 512, 512);
        assert_eq!(v.mip_levels, 10); /* 512 = 2^9, so 10 levels */
    }

    #[test]
    fn test_set_dimensions_min() {
        let mut v = new_mipmap_chain_view();
        mmv_set_base_dimensions(&mut v, 0, 0);
        assert_eq!(v.base_width, 1);
    }

    #[test]
    fn test_show_level_borders() {
        let mut v = new_mipmap_chain_view();
        mmv_set_show_level_borders(&mut v, true);
        assert!(v.show_level_borders);
    }

    #[test]
    fn test_level_dimensions_level_0() {
        let mut v = new_mipmap_chain_view();
        mmv_set_base_dimensions(&mut v, 256, 128);
        let (w, h) = mmv_level_dimensions(&v, 0);
        assert_eq!(w, 256);
        assert_eq!(h, 128);
    }

    #[test]
    fn test_level_dimensions_halve() {
        let mut v = new_mipmap_chain_view();
        mmv_set_base_dimensions(&mut v, 256, 256);
        let (w, _h) = mmv_level_dimensions(&v, 1);
        assert_eq!(w, 128);
    }

    #[test]
    fn test_total_texels_positive() {
        let mut v = new_mipmap_chain_view();
        mmv_set_base_dimensions(&mut v, 64, 64);
        assert!(mmv_total_texels(&v) > 64 * 64);
    }

    #[test]
    fn test_json_keys() {
        let v = new_mipmap_chain_view();
        let s = mipmap_chain_view_to_json(&v);
        assert!(s.contains("mip_levels"));
    }

    #[test]
    fn test_clone() {
        let v = new_mipmap_chain_view();
        let v2 = v.clone();
        assert_eq!(v2.base_width, v.base_width);
    }
}
