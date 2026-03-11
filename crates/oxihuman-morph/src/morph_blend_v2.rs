#![allow(dead_code)]

//! Multi-target blend with normalized weights.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphBlendTarget {
    pub name: String,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphBlendV2 {
    pub targets: Vec<MorphBlendTarget>,
    pub normalize: bool,
    pub total_weight: f32,
}

#[allow(dead_code)]
pub fn new_morph_blend_v2(normalize: bool) -> MorphBlendV2 {
    MorphBlendV2 {
        targets: Vec::new(),
        normalize,
        total_weight: 0.0,
    }
}

#[allow(dead_code)]
pub fn mbv2_add_target(blend: &mut MorphBlendV2, name: &str, weight: f32) {
    blend.targets.push(MorphBlendTarget {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
    });
    blend.total_weight = blend.targets.iter().map(|t| t.weight).sum();
}

#[allow(dead_code)]
pub fn mbv2_set_weight(blend: &mut MorphBlendV2, name: &str, weight: f32) {
    if let Some(t) = blend.targets.iter_mut().find(|t| t.name == name) {
        t.weight = weight.clamp(0.0, 1.0);
    }
    blend.total_weight = blend.targets.iter().map(|t| t.weight).sum();
}

#[allow(dead_code)]
pub fn mbv2_effective_weight(blend: &MorphBlendV2, name: &str) -> f32 {
    let t = blend.targets.iter().find(|t| t.name == name);
    match t {
        None => 0.0,
        Some(target) => {
            if blend.normalize && blend.total_weight > 0.0 {
                target.weight / blend.total_weight
            } else {
                target.weight
            }
        }
    }
}

#[allow(dead_code)]
pub fn mbv2_target_count(blend: &MorphBlendV2) -> usize {
    blend.targets.len()
}

#[allow(dead_code)]
pub fn mbv2_clear(blend: &mut MorphBlendV2) {
    blend.targets.clear();
    blend.total_weight = 0.0;
}

#[allow(dead_code)]
pub fn mbv2_remove(blend: &mut MorphBlendV2, name: &str) {
    blend.targets.retain(|t| t.name != name);
    blend.total_weight = blend.targets.iter().map(|t| t.weight).sum();
}

#[allow(dead_code)]
pub fn mbv2_normalize(blend: &mut MorphBlendV2) {
    let total: f32 = blend.targets.iter().map(|t| t.weight).sum();
    if total > 0.0 {
        for t in &mut blend.targets {
            t.weight /= total;
        }
        blend.total_weight = 1.0;
    }
}

#[allow(dead_code)]
pub fn mbv2_to_json(blend: &MorphBlendV2) -> String {
    format!(
        "{{\"target_count\":{},\"normalize\":{},\"total_weight\":{}}}",
        blend.targets.len(),
        blend.normalize,
        blend.total_weight
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blend() {
        let b = new_morph_blend_v2(false);
        assert_eq!(mbv2_target_count(&b), 0);
    }

    #[test]
    fn test_add_target() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "smile", 0.5);
        assert_eq!(mbv2_target_count(&b), 1);
    }

    #[test]
    fn test_effective_weight_no_normalize() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "smile", 0.6);
        assert!((mbv2_effective_weight(&b, "smile") - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_effective_weight_normalized() {
        let mut b = new_morph_blend_v2(true);
        mbv2_add_target(&mut b, "a", 0.5);
        mbv2_add_target(&mut b, "b", 0.5);
        assert!((mbv2_effective_weight(&b, "a") - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "jaw", 0.3);
        mbv2_set_weight(&mut b, "jaw", 0.8);
        assert!((mbv2_effective_weight(&b, "jaw") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_remove() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "blink", 0.5);
        mbv2_remove(&mut b, "blink");
        assert_eq!(mbv2_target_count(&b), 0);
    }

    #[test]
    fn test_clear() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "x", 0.5);
        mbv2_add_target(&mut b, "y", 0.5);
        mbv2_clear(&mut b);
        assert_eq!(mbv2_target_count(&b), 0);
    }

    #[test]
    fn test_normalize_function() {
        let mut b = new_morph_blend_v2(false);
        mbv2_add_target(&mut b, "a", 2.0);
        mbv2_normalize(&mut b);
        assert!((b.total_weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let b = new_morph_blend_v2(true);
        let json = mbv2_to_json(&b);
        assert!(json.contains("normalize"));
    }

    #[test]
    fn test_unknown_target_returns_zero() {
        let b = new_morph_blend_v2(false);
        assert!((mbv2_effective_weight(&b, "nonexistent") - 0.0).abs() < 1e-6);
    }
}
