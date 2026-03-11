#![allow(dead_code)]

/// Blends multiple skin textures by weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinTextureBlend {
    names: Vec<String>,
    weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_skin_texture_blend() -> SkinTextureBlend {
    SkinTextureBlend { names: Vec::new(), weights: Vec::new() }
}

#[allow(dead_code)]
pub fn blend_textures(stb: &SkinTextureBlend) -> Vec<(String, f32)> {
    stb.names.iter().zip(stb.weights.iter()).map(|(n, w)| (n.clone(), *w)).collect()
}

#[allow(dead_code)]
pub fn texture_weight(stb: &SkinTextureBlend, idx: usize) -> f32 {
    stb.weights.get(idx).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn texture_count_stb(stb: &SkinTextureBlend) -> usize { stb.names.len() }

#[allow(dead_code)]
pub fn set_texture_weight(stb: &mut SkinTextureBlend, name: &str, weight: f32) {
    if let Some(pos) = stb.names.iter().position(|n| n == name) {
        stb.weights[pos] = weight;
    } else {
        stb.names.push(name.to_string());
        stb.weights.push(weight);
    }
}

#[allow(dead_code)]
pub fn texture_blend_to_json(stb: &SkinTextureBlend) -> String {
    let e: Vec<String> = stb.names.iter().zip(stb.weights.iter())
        .map(|(n, w)| format!("\"{}\":{:.4}", n, w)).collect();
    format!("{{{}}}", e.join(","))
}

#[allow(dead_code)]
pub fn normalize_texture_weights(stb: &mut SkinTextureBlend) {
    let sum: f32 = stb.weights.iter().sum();
    if sum > 1e-9 { for w in stb.weights.iter_mut() { *w /= sum; } }
}

#[allow(dead_code)]
pub fn clear_texture_blend(stb: &mut SkinTextureBlend) {
    stb.names.clear();
    stb.weights.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { assert_eq!(texture_count_stb(&new_skin_texture_blend()), 0); }
    #[test] fn test_set_get() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "base", 0.5);
        assert!((texture_weight(&s, 0) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_count() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "a", 1.0);
        set_texture_weight(&mut s, "b", 0.5);
        assert_eq!(texture_count_stb(&s), 2);
    }
    #[test] fn test_overwrite() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "a", 1.0);
        set_texture_weight(&mut s, "a", 0.5);
        assert_eq!(texture_count_stb(&s), 1);
        assert!((texture_weight(&s, 0) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_blend() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "x", 0.7);
        let b = blend_textures(&s);
        assert_eq!(b[0].0, "x");
    }
    #[test] fn test_normalize() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "a", 2.0);
        set_texture_weight(&mut s, "b", 2.0);
        normalize_texture_weights(&mut s);
        assert!((texture_weight(&s, 0) - 0.5).abs() < 1e-6);
    }
    #[test] fn test_clear() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "x", 1.0);
        clear_texture_blend(&mut s);
        assert_eq!(texture_count_stb(&s), 0);
    }
    #[test] fn test_to_json() {
        let mut s = new_skin_texture_blend();
        set_texture_weight(&mut s, "t", 0.5);
        assert!(texture_blend_to_json(&s).contains("t"));
    }
    #[test] fn test_weight_oob() { assert!((texture_weight(&new_skin_texture_blend(), 99)).abs() < 1e-6); }
    #[test] fn test_normalize_empty() {
        let mut s = new_skin_texture_blend();
        normalize_texture_weights(&mut s);
        assert_eq!(texture_count_stb(&s), 0);
    }
}
