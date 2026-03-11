#![allow(dead_code)]
//! Skin detail parameters — pores, wrinkles, freckles.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum DetailLevel {
    Low,
    Medium,
    High,
    Custom(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SkinDetail {
    pub pore_intensity: f32,
    pub wrinkle_depth: f32,
    pub freckle_density: f32,
    pub level: DetailLevel,
}

#[allow(dead_code)]
pub fn new_skin_detail(level: DetailLevel) -> SkinDetail {
    let (pore, wrinkle, freckle) = match &level {
        DetailLevel::Low => (0.1, 0.1, 0.05),
        DetailLevel::Medium => (0.5, 0.5, 0.3),
        DetailLevel::High => (0.9, 0.8, 0.6),
        DetailLevel::Custom(_) => (0.5, 0.5, 0.3),
    };
    SkinDetail {
        pore_intensity: pore,
        wrinkle_depth: wrinkle,
        freckle_density: freckle,
        level,
    }
}

#[allow(dead_code)]
pub fn set_pore_intensity(sd: &mut SkinDetail, v: f32) {
    sd.pore_intensity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_wrinkle_depth(sd: &mut SkinDetail, v: f32) {
    sd.wrinkle_depth = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_freckle_density(sd: &mut SkinDetail, v: f32) {
    sd.freckle_density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn detail_to_params(sd: &SkinDetail) -> HashMap<String, f32> {
    let mut m = HashMap::new();
    m.insert("pore_intensity".to_string(), sd.pore_intensity);
    m.insert("wrinkle_depth".to_string(), sd.wrinkle_depth);
    m.insert("freckle_density".to_string(), sd.freckle_density);
    m
}

#[allow(dead_code)]
pub fn detail_level_name(level: &DetailLevel) -> &str {
    match level {
        DetailLevel::Low => "low",
        DetailLevel::Medium => "medium",
        DetailLevel::High => "high",
        DetailLevel::Custom(n) => n.as_str(),
    }
}

#[allow(dead_code)]
pub fn detail_to_json(sd: &SkinDetail) -> String {
    format!(
        "{{\"level\":\"{}\",\"pore\":{},\"wrinkle\":{},\"freckle\":{}}}",
        detail_level_name(&sd.level),
        sd.pore_intensity,
        sd.wrinkle_depth,
        sd.freckle_density
    )
}

#[allow(dead_code)]
pub fn default_skin_detail() -> SkinDetail {
    new_skin_detail(DetailLevel::Medium)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_skin_detail() {
        let sd = new_skin_detail(DetailLevel::Medium);
        assert!((sd.pore_intensity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_pore_intensity() {
        let mut sd = default_skin_detail();
        set_pore_intensity(&mut sd, 0.8);
        assert!((sd.pore_intensity - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_wrinkle_depth() {
        let mut sd = default_skin_detail();
        set_wrinkle_depth(&mut sd, 0.3);
        assert!((sd.wrinkle_depth - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_freckle_density() {
        let mut sd = default_skin_detail();
        set_freckle_density(&mut sd, 0.7);
        assert!((sd.freckle_density - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_detail_to_params() {
        let sd = default_skin_detail();
        let params = detail_to_params(&sd);
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_detail_level_name() {
        assert_eq!(detail_level_name(&DetailLevel::High), "high");
        assert_eq!(detail_level_name(&DetailLevel::Custom("x".into())), "x");
    }

    #[test]
    fn test_detail_to_json() {
        let sd = default_skin_detail();
        let json = detail_to_json(&sd);
        assert!(json.contains("\"level\":\"medium\""));
    }

    #[test]
    fn test_default_skin_detail() {
        let sd = default_skin_detail();
        assert_eq!(sd.level, DetailLevel::Medium);
    }

    #[test]
    fn test_clamp_values() {
        let mut sd = default_skin_detail();
        set_pore_intensity(&mut sd, 5.0);
        assert!((sd.pore_intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_low_detail() {
        let sd = new_skin_detail(DetailLevel::Low);
        assert!(sd.pore_intensity < 0.2);
    }
}
