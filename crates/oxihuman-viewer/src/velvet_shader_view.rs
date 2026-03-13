// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct VelvetView {
    pub show_sheen: bool,
    pub sheen_roughness: f32,
}

pub fn new_velvet_view() -> VelvetView {
    VelvetView {
        show_sheen: true,
        sheen_roughness: 0.5,
    }
}

pub fn velvet_sheen_color(cos_theta: f32, roughness: f32) -> [f32; 3] {
    let sheen = velvet_rim_highlight(cos_theta, roughness);
    [sheen, sheen, sheen]
}

pub fn velvet_retroreflection(cos_theta: f32) -> f32 {
    let sin2 = (1.0 - cos_theta * cos_theta).max(0.0);
    sin2 * sin2
}

pub fn velvet_rim_highlight(cos_theta: f32, roughness: f32) -> f32 {
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    (sin_theta.powi(4) / (roughness * roughness).max(1e-4)).min(1.0)
}

pub fn velvet_silhouette_boost(cos_theta: f32) -> f32 {
    (1.0 - cos_theta.abs()).max(0.0).powi(3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_velvet_view() {
        /* sheen roughness defaults to 0.5 */
        let v = new_velvet_view();
        assert!((v.sheen_roughness - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_velvet_retroreflection_at_normal() {
        /* at cos_theta=1 (normal), retro = 0 */
        let r = velvet_retroreflection(1.0);
        assert!(r < 1e-6);
    }

    #[test]
    fn test_velvet_rim_highlight_at_grazing() {
        /* at grazing angle cos_theta~0, rim highlight is maximal */
        let r = velvet_rim_highlight(0.01, 0.5);
        assert!(r > 0.0);
    }

    #[test]
    fn test_velvet_silhouette_boost_at_grazing() {
        /* at cos_theta=0, boost is 1 */
        let b = velvet_silhouette_boost(0.0);
        assert!((b - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_velvet_sheen_color_gray() {
        /* sheen color should be gray (r=g=b) */
        let c = velvet_sheen_color(0.5, 0.5);
        assert!((c[0] - c[1]).abs() < 1e-6);
        assert!((c[1] - c[2]).abs() < 1e-6);
    }
}
