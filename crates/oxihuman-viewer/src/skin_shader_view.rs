// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SkinShaderView {
    pub show_diffuse: bool,
    pub show_specular: bool,
    pub show_sss: bool,
    pub show_layers: bool,
}

pub fn new_skin_shader_view() -> SkinShaderView {
    SkinShaderView {
        show_diffuse: true,
        show_specular: true,
        show_sss: true,
        show_layers: false,
    }
}

/// Schlick Fresnel specular.
pub fn skin_specular_color(roughness: f32, ior: f32, cos_theta: f32) -> [f32; 3] {
    let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
    let f = f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5);
    let intensity = f * (1.0 - roughness);
    [intensity, intensity, intensity]
}

pub fn skin_diffuse_color(albedo: [f32; 3]) -> [f32; 3] {
    albedo
}

/// Epidermis = reddish, dermis = orange-red, hypodermis = yellow.
pub fn skin_sss_radius_color(layer: u8) -> [f32; 3] {
    match layer {
        0 => [1.0, 0.4, 0.3], // epidermis
        1 => [0.9, 0.2, 0.1], // dermis
        _ => [0.9, 0.8, 0.1], // hypodermis
    }
}

pub fn skin_layer_weight(layer: u8) -> f32 {
    match layer {
        0 => 0.2,
        1 => 0.6,
        _ => 0.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_shader_view() {
        /* show_diffuse defaults to true */
        let v = new_skin_shader_view();
        assert!(v.show_diffuse);
    }

    #[test]
    fn test_skin_specular_at_normal() {
        /* at cos_theta=1 (normal incidence), specular should be near f0 */
        let c = skin_specular_color(0.0, 1.4, 1.0);
        assert!(c[0] >= 0.0);
    }

    #[test]
    fn test_skin_diffuse_returns_albedo() {
        /* diffuse returns albedo unchanged */
        let albedo = [0.9, 0.7, 0.6];
        let d = skin_diffuse_color(albedo);
        assert_eq!(d, albedo);
    }

    #[test]
    fn test_skin_sss_radius_color_layers() {
        /* each layer has different color */
        let ep = skin_sss_radius_color(0);
        let de = skin_sss_radius_color(1);
        assert_ne!(ep, de);
    }

    #[test]
    fn test_skin_layer_weights_sum() {
        /* layer weights sum to 1 */
        let sum = skin_layer_weight(0) + skin_layer_weight(1) + skin_layer_weight(2);
        assert!((sum - 1.0).abs() < 1e-6);
    }
}
