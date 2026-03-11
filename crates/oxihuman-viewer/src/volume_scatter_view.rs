// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Volume scattering debug view.

/// Volume scatter debug state.
#[derive(Debug, Clone)]
pub struct VolumeScatterView {
    pub density: f32,
    pub anisotropy: f32,
    pub scatter_color: [f32; 3],
    pub show_bounds: bool,
    pub step_size: f32,
}

impl Default for VolumeScatterView {
    fn default() -> Self {
        Self {
            density: 1.0,
            anisotropy: 0.0,
            scatter_color: [1.0, 1.0, 1.0],
            show_bounds: false,
            step_size: 0.1,
        }
    }
}

/// Create a new VolumeScatterView.
pub fn new_volume_scatter_view() -> VolumeScatterView {
    VolumeScatterView::default()
}

/// Set scattering density.
pub fn volume_scatter_set_density(view: &mut VolumeScatterView, v: f32) {
    view.density = v.clamp(0.0, 100.0);
}

/// Set Henyey-Greenstein anisotropy (-1.0 to 1.0).
pub fn volume_scatter_set_anisotropy(view: &mut VolumeScatterView, v: f32) {
    view.anisotropy = v.clamp(-1.0, 1.0);
}

/// Set step size for ray marching.
pub fn volume_scatter_set_step_size(view: &mut VolumeScatterView, v: f32) {
    view.step_size = v.clamp(1e-4, 10.0);
}

/// Toggle bounds visualization.
pub fn volume_scatter_show_bounds(view: &mut VolumeScatterView, show: bool) {
    view.show_bounds = show;
}

/// Compute approximate optical depth for a ray length.
pub fn volume_scatter_optical_depth(view: &VolumeScatterView, length: f32) -> f32 {
    view.density * length
}

/// Serialize to JSON.
pub fn volume_scatter_to_json(view: &VolumeScatterView) -> String {
    format!(
        r#"{{"density":{},"anisotropy":{},"step_size":{},"show_bounds":{}}}"#,
        view.density, view.anisotropy, view.step_size, view.show_bounds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_volume_scatter_view();
        assert!((v.density - 1.0).abs() < 1e-6 /* default */);
    }

    #[test]
    fn test_density_clamp() {
        let mut v = new_volume_scatter_view();
        volume_scatter_set_density(&mut v, 200.0);
        assert!((v.density - 100.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_anisotropy_clamp() {
        let mut v = new_volume_scatter_view();
        volume_scatter_set_anisotropy(&mut v, 5.0);
        assert!((v.anisotropy - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_anisotropy_negative() {
        let mut v = new_volume_scatter_view();
        volume_scatter_set_anisotropy(&mut v, -0.5);
        assert!((v.anisotropy - (-0.5)).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_step_size() {
        let mut v = new_volume_scatter_view();
        volume_scatter_set_step_size(&mut v, 0.05);
        assert!((v.step_size - 0.05).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_show_bounds() {
        let mut v = new_volume_scatter_view();
        volume_scatter_show_bounds(&mut v, true);
        assert!(v.show_bounds /* enabled */);
    }

    #[test]
    fn test_optical_depth() {
        let v = new_volume_scatter_view();
        let od = volume_scatter_optical_depth(&v, 2.0);
        assert!((od - 2.0).abs() < 1e-6 /* density 1 * length 2 */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_volume_scatter_view();
        let j = volume_scatter_to_json(&v);
        assert!(j.contains("anisotropy") /* key */);
    }

    #[test]
    fn test_default_color() {
        let v = VolumeScatterView::default();
        assert!((v.scatter_color[0] - 1.0).abs() < 1e-6 /* white */);
    }
}

/* ── Wave-148C additions ─────────────────────────────────────────────────── */

/// Debug view config (show_density / show_albedo / show_anisotropy).
pub struct VolumeScatterDebugView {
    pub show_density: bool,
    pub show_albedo: bool,
    pub show_anisotropy: bool,
}

pub fn new_volume_scatter_debug_view() -> VolumeScatterDebugView {
    VolumeScatterDebugView {
        show_density: true,
        show_albedo: false,
        show_anisotropy: false,
    }
}

pub fn volume_density_color(density: f32) -> [f32; 3] {
    let d = density.clamp(0.0, 1.0);
    [d, d * 0.5, 0.0]
}

pub fn volume_albedo_color(absorption: f32, scattering: f32) -> [f32; 3] {
    let total = absorption + scattering;
    if total < 1e-9 {
        return [0.0, 0.0, 0.0];
    }
    let albedo = scattering / total;
    [albedo, albedo, albedo]
}

pub fn volume_phase_function_hg(cos_theta: f32, g: f32) -> f32 {
    /* Henyey-Greenstein phase function */
    let g2 = g * g;
    let denom = (1.0 + g2 - 2.0 * g * cos_theta).powf(1.5);
    (1.0 - g2) / (4.0 * std::f32::consts::PI * denom.max(1e-9))
}

pub fn volume_mean_free_path(extinction: f32) -> f32 {
    if extinction < 1e-9 {
        f32::INFINITY
    } else {
        1.0 / extinction
    }
}

#[cfg(test)]
mod wave148c_tests {
    use super::*;

    #[test]
    fn test_volume_density_color_zero() {
        /* zero density -> black */
        let c = volume_density_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume_albedo_color_pure_scatter() {
        /* pure scattering -> albedo=1 -> white */
        let c = volume_albedo_color(0.0, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_volume_phase_function_hg_isotropic() {
        /* g=0 -> isotropic: p = 1/(4*pi) */
        let p = volume_phase_function_hg(1.0, 0.0);
        let expected = 1.0 / (4.0 * std::f32::consts::PI);
        assert!((p - expected).abs() < 1e-5);
    }

    #[test]
    fn test_volume_mean_free_path() {
        /* extinction=2 -> mfp=0.5 */
        let mfp = volume_mean_free_path(2.0);
        assert!((mfp - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_volume_scatter_debug_view() {
        /* show_density defaults to true */
        let v = new_volume_scatter_debug_view();
        assert!(v.show_density);
    }
}
