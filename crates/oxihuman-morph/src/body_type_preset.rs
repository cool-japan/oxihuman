#![allow(dead_code)]
//! Body type presets — ectomorph, mesomorph, endomorph with blending.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BodyType {
    Ectomorph,
    Mesomorph,
    Endomorph,
    Custom(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct BodyTypePreset {
    pub body_type: BodyType,
    pub params: HashMap<String, f32>,
}

#[allow(dead_code)]
pub fn new_body_type_preset(body_type: BodyType) -> BodyTypePreset {
    let params = body_type_to_params(&body_type);
    BodyTypePreset { body_type, params }
}

#[allow(dead_code)]
pub fn ectomorph_preset() -> BodyTypePreset {
    new_body_type_preset(BodyType::Ectomorph)
}

#[allow(dead_code)]
pub fn mesomorph_preset() -> BodyTypePreset {
    new_body_type_preset(BodyType::Mesomorph)
}

#[allow(dead_code)]
pub fn endomorph_preset() -> BodyTypePreset {
    new_body_type_preset(BodyType::Endomorph)
}

#[allow(dead_code)]
pub fn body_type_name(bt: &BodyType) -> &str {
    match bt {
        BodyType::Ectomorph => "ectomorph",
        BodyType::Mesomorph => "mesomorph",
        BodyType::Endomorph => "endomorph",
        BodyType::Custom(n) => n.as_str(),
    }
}

#[allow(dead_code)]
pub fn body_type_to_params(bt: &BodyType) -> HashMap<String, f32> {
    let mut m = HashMap::new();
    match bt {
        BodyType::Ectomorph => {
            m.insert("muscle".to_string(), 0.2);
            m.insert("fat".to_string(), 0.1);
            m.insert("height".to_string(), 0.7);
        }
        BodyType::Mesomorph => {
            m.insert("muscle".to_string(), 0.7);
            m.insert("fat".to_string(), 0.3);
            m.insert("height".to_string(), 0.5);
        }
        BodyType::Endomorph => {
            m.insert("muscle".to_string(), 0.4);
            m.insert("fat".to_string(), 0.7);
            m.insert("height".to_string(), 0.4);
        }
        BodyType::Custom(_) => {}
    }
    m
}

#[allow(dead_code)]
pub fn blend_body_types(a: &BodyTypePreset, b: &BodyTypePreset, t: f32) -> HashMap<String, f32> {
    let t = t.clamp(0.0, 1.0);
    let mut result = HashMap::new();
    for (k, va) in &a.params {
        let vb = b.params.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), va * (1.0 - t) + vb * t);
    }
    for (k, vb) in &b.params {
        if !result.contains_key(k) {
            result.insert(k.clone(), vb * t);
        }
    }
    result
}

#[allow(dead_code)]
pub fn body_type_to_json(preset: &BodyTypePreset) -> String {
    let params: Vec<String> = preset
        .params
        .iter()
        .map(|(k, v)| format!("\"{k}\":{v}"))
        .collect();
    format!(
        "{{\"type\":\"{}\",\"params\":{{{}}}}}",
        body_type_name(&preset.body_type),
        params.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_type_preset() {
        let p = new_body_type_preset(BodyType::Ectomorph);
        assert!(!p.params.is_empty());
    }

    #[test]
    fn test_ectomorph_preset() {
        let p = ectomorph_preset();
        assert!(p.params.contains_key("muscle"));
    }

    #[test]
    fn test_mesomorph_preset() {
        let p = mesomorph_preset();
        assert!(*p.params.get("muscle").unwrap() > 0.5);
    }

    #[test]
    fn test_endomorph_preset() {
        let p = endomorph_preset();
        assert!(*p.params.get("fat").unwrap() > 0.5);
    }

    #[test]
    fn test_body_type_name() {
        assert_eq!(body_type_name(&BodyType::Ectomorph), "ectomorph");
        assert_eq!(body_type_name(&BodyType::Custom("x".into())), "x");
    }

    #[test]
    fn test_blend_body_types() {
        let a = ectomorph_preset();
        let b = endomorph_preset();
        let blended = blend_body_types(&a, &b, 0.5);
        assert!(blended.contains_key("muscle"));
    }

    #[test]
    fn test_blend_at_zero() {
        let a = ectomorph_preset();
        let b = endomorph_preset();
        let blended = blend_body_types(&a, &b, 0.0);
        let av = a.params.get("muscle").unwrap();
        let bv = blended.get("muscle").unwrap();
        assert!((av - bv).abs() < 1e-6);
    }

    #[test]
    fn test_body_type_to_json() {
        let p = mesomorph_preset();
        let json = body_type_to_json(&p);
        assert!(json.contains("\"type\":\"mesomorph\""));
    }

    #[test]
    fn test_custom_body_type() {
        let p = new_body_type_preset(BodyType::Custom("alien".into()));
        assert!(p.params.is_empty());
    }

    #[test]
    fn test_body_type_to_params() {
        let params = body_type_to_params(&BodyType::Mesomorph);
        assert_eq!(params.len(), 3);
    }
}
