// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export emission/emissive material properties.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmissionExport {
    pub material_name: String,
    pub color: [f32; 3],
    pub intensity: f32,
    pub texture_path: Option<String>,
}

#[allow(dead_code)]
pub fn new_emission_export(material: &str, color: [f32; 3], intensity: f32) -> EmissionExport {
    EmissionExport { material_name: material.to_string(), color, intensity: intensity.max(0.0), texture_path: None }
}

#[allow(dead_code)]
pub fn ee_set_texture(ee: &mut EmissionExport, path: &str) {
    ee.texture_path = Some(path.to_string());
}

#[allow(dead_code)]
pub fn ee_luminance(ee: &EmissionExport) -> f32 {
    (0.2126 * ee.color[0] + 0.7152 * ee.color[1] + 0.0722 * ee.color[2]) * ee.intensity
}

#[allow(dead_code)]
pub fn ee_has_texture(ee: &EmissionExport) -> bool { ee.texture_path.is_some() }

#[allow(dead_code)]
pub fn ee_validate(ee: &EmissionExport) -> bool {
    !ee.material_name.is_empty() && ee.intensity >= 0.0 && ee.color.iter().all(|&c| (0.0..=1.0).contains(&c))
}

#[allow(dead_code)]
pub fn ee_is_black(ee: &EmissionExport) -> bool {
    ee.color[0].abs() < 1e-6 && ee.color[1].abs() < 1e-6 && ee.color[2].abs() < 1e-6
}

#[allow(dead_code)]
pub fn ee_scale_intensity(ee: &mut EmissionExport, factor: f32) {
    ee.intensity = (ee.intensity * factor).max(0.0);
}

#[allow(dead_code)]
pub fn ee_to_json(ee: &EmissionExport) -> String {
    format!("{{\"material\":\"{}\",\"color\":[{:.3},{:.3},{:.3}],\"intensity\":{:.4}}}", ee.material_name, ee.color[0], ee.color[1], ee.color[2], ee.intensity)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> EmissionExport { new_emission_export("glow", [1.0, 0.5, 0.0], 2.0) }

    #[test] fn test_new() { let e = sample(); assert_eq!(e.material_name, "glow"); }
    #[test] fn test_luminance() { let e = sample(); assert!(ee_luminance(&e) > 0.0); }
    #[test] fn test_no_texture() { assert!(!ee_has_texture(&sample())); }
    #[test] fn test_set_texture() { let mut e = sample(); ee_set_texture(&mut e, "glow.png"); assert!(ee_has_texture(&e)); }
    #[test] fn test_validate() { assert!(ee_validate(&sample())); }
    #[test] fn test_not_black() { assert!(!ee_is_black(&sample())); }
    #[test] fn test_black() { let e = new_emission_export("m", [0.0,0.0,0.0], 1.0); assert!(ee_is_black(&e)); }
    #[test] fn test_scale() { let mut e = sample(); ee_scale_intensity(&mut e, 0.5); assert!((e.intensity - 1.0).abs() < 1e-5); }
    #[test] fn test_to_json() { assert!(ee_to_json(&sample()).contains("glow")); }
    #[test] fn test_negative_intensity() { let e = new_emission_export("m", [1.0,1.0,1.0], -5.0); assert!((e.intensity).abs() < 1e-6); }
}
