// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct MeshDensityView {
    pub low_threshold: f32,
    pub high_threshold: f32,
    pub enabled: bool,
}

pub fn new_mesh_density_view() -> MeshDensityView {
    MeshDensityView {
        low_threshold: 0.3,
        high_threshold: 0.7,
        enabled: false,
    }
}

pub fn mdv_set_thresholds(v: &mut MeshDensityView, low: f32, high: f32) {
    v.low_threshold = low.clamp(0.0, 1.0);
    v.high_threshold = high.clamp(0.0, 1.0).max(v.low_threshold);
}

pub fn mdv_density_color(v: &MeshDensityView, density: f32) -> [f32; 3] {
    if density < v.low_threshold {
        [0.0, 0.5, 1.0] /* blue for low density */
    } else if density > v.high_threshold {
        [1.0, 0.0, 0.0] /* red for high density */
    } else {
        [0.0, 1.0, 0.0] /* green for medium density */
    }
}

pub fn mdv_enable(v: &mut MeshDensityView) {
    v.enabled = true;
}

pub fn mdv_is_enabled(v: &MeshDensityView) -> bool {
    v.enabled
}

pub fn mdv_to_json(v: &MeshDensityView) -> String {
    format!(
        r#"{{"low_threshold":{:.4},"high_threshold":{:.4},"enabled":{}}}"#,
        v.low_threshold, v.high_threshold, v.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* thresholds sensible, disabled */
        let v = new_mesh_density_view();
        assert!(v.low_threshold < v.high_threshold);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_thresholds() {
        /* stored correctly */
        let mut v = new_mesh_density_view();
        mdv_set_thresholds(&mut v, 0.2, 0.8);
        assert!((v.low_threshold - 0.2).abs() < 1e-6);
        assert!((v.high_threshold - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_density_color_low() {
        /* below low threshold => blue */
        let v = new_mesh_density_view();
        let c = mdv_density_color(&v, 0.0);
        assert!(c[2] > c[0]);
    }

    #[test]
    fn test_density_color_high() {
        /* above high threshold => red */
        let v = new_mesh_density_view();
        let c = mdv_density_color(&v, 1.0);
        assert!(c[0] > c[1]);
    }

    #[test]
    fn test_density_color_mid() {
        /* middle density => green */
        let v = new_mesh_density_view();
        let c = mdv_density_color(&v, 0.5);
        assert!(c[1] > c[0]);
    }

    #[test]
    fn test_enable() {
        /* enable sets flag */
        let mut v = new_mesh_density_view();
        mdv_enable(&mut v);
        assert!(mdv_is_enabled(&v));
    }

    #[test]
    fn test_to_json_has_thresholds() {
        /* JSON includes thresholds */
        let v = new_mesh_density_view();
        let s = mdv_to_json(&v);
        assert!(s.contains("low_threshold"));
        assert!(s.contains("high_threshold"));
    }

    #[test]
    fn test_to_json_enabled_false() {
        /* new view JSON shows disabled */
        let v = new_mesh_density_view();
        assert!(mdv_to_json(&v).contains("false"));
    }

    #[test]
    fn test_clone() {
        /* clone independent */
        let v = new_mesh_density_view();
        let v2 = v.clone();
        assert_eq!(v.enabled, v2.enabled);
    }
}
