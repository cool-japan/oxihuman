// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct LightmapDensityView {
    pub texels_per_unit: f32,
    pub enabled: bool,
    pub show_grid: bool,
}

pub fn new_lightmap_density_view() -> LightmapDensityView {
    LightmapDensityView {
        texels_per_unit: 1.0,
        enabled: false,
        show_grid: false,
    }
}

pub fn ldv_set_density(v: &mut LightmapDensityView, tpu: f32) {
    v.texels_per_unit = tpu.max(0.001);
}

pub fn ldv_enable(v: &mut LightmapDensityView) {
    v.enabled = true;
}

pub fn ldv_density_color(_v: &LightmapDensityView, ratio: f32) -> [f32; 3] {
    let r = ratio.clamp(0.0, 1.0);
    /* blue=under, green=good, red=over */
    if r < 0.5 {
        let t = r * 2.0;
        [0.0, t, 1.0 - t]
    } else {
        let t = (r - 0.5) * 2.0;
        [t, 1.0 - t, 0.0]
    }
}

pub fn ldv_toggle_grid(v: &mut LightmapDensityView) {
    v.show_grid = !v.show_grid;
}

pub fn ldv_is_enabled(v: &LightmapDensityView) -> bool {
    v.enabled
}

pub fn ldv_to_json(v: &LightmapDensityView) -> String {
    format!(
        r#"{{"texels_per_unit":{:.4},"enabled":{},"show_grid":{}}}"#,
        v.texels_per_unit, v.enabled, v.show_grid
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* tpu=1, disabled, no grid */
        let v = new_lightmap_density_view();
        assert!((v.texels_per_unit - 1.0).abs() < 1e-6);
        assert!(!v.enabled);
        assert!(!v.show_grid);
    }

    #[test]
    fn test_set_density() {
        /* valid value stored */
        let mut v = new_lightmap_density_view();
        ldv_set_density(&mut v, 4.0);
        assert!((v.texels_per_unit - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_density_clamp() {
        /* 0 clamped to 0.001 */
        let mut v = new_lightmap_density_view();
        ldv_set_density(&mut v, 0.0);
        assert!(v.texels_per_unit >= 0.001);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_lightmap_density_view();
        ldv_enable(&mut v);
        assert!(ldv_is_enabled(&v));
    }

    #[test]
    fn test_density_color_under() {
        /* ratio=0 => blueish */
        let v = new_lightmap_density_view();
        let c = ldv_density_color(&v, 0.0);
        assert!(c[2] > c[0]);
    }

    #[test]
    fn test_density_color_over() {
        /* ratio=1 => reddish */
        let v = new_lightmap_density_view();
        let c = ldv_density_color(&v, 1.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_toggle_grid() {
        /* toggle changes state */
        let mut v = new_lightmap_density_view();
        ldv_toggle_grid(&mut v);
        assert!(v.show_grid);
        ldv_toggle_grid(&mut v);
        assert!(!v.show_grid);
    }

    #[test]
    fn test_to_json() {
        /* JSON has texels_per_unit */
        let v = new_lightmap_density_view();
        assert!(ldv_to_json(&v).contains("texels_per_unit"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_lightmap_density_view();
        let v2 = v.clone();
        assert_eq!(v.show_grid, v2.show_grid);
    }
}
