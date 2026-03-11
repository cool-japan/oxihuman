#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct LodBias {
    pub value: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureLod {
    bias: f32,
    max_level: u32,
    min_level: u32,
}

#[allow(dead_code)]
pub fn new_texture_lod(max_level: u32) -> TextureLod {
    TextureLod { bias: 0.0, max_level, min_level: 0 }
}

#[allow(dead_code)]
pub fn set_lod_bias(lod: &mut TextureLod, bias: f32) { lod.bias = bias; }

#[allow(dead_code)]
pub fn lod_bias_value(lod: &TextureLod) -> f32 { lod.bias }

#[allow(dead_code)]
pub fn compute_mip_level(lod: &TextureLod, distance: f32) -> u32 {
    let base = (distance.log2().max(0.0) + lod.bias).max(0.0) as u32;
    base.clamp(lod.min_level, lod.max_level)
}

#[allow(dead_code)]
pub fn texture_lod_to_json(lod: &TextureLod) -> String {
    format!("{{\"bias\":{:.2},\"max\":{},\"min\":{}}}", lod.bias, lod.max_level, lod.min_level)
}

#[allow(dead_code)]
pub fn lod_max_level(lod: &TextureLod) -> u32 { lod.max_level }

#[allow(dead_code)]
pub fn lod_min_level(lod: &TextureLod) -> u32 { lod.min_level }

#[allow(dead_code)]
pub fn lod_reset(lod: &mut TextureLod) { lod.bias = 0.0; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let l = new_texture_lod(10); assert_eq!(lod_max_level(&l), 10); }
    #[test] fn test_bias() { let mut l = new_texture_lod(8); set_lod_bias(&mut l, 1.5); assert!((lod_bias_value(&l) - 1.5).abs() < 1e-6); }
    #[test] fn test_mip_near() { let l = new_texture_lod(10); assert_eq!(compute_mip_level(&l, 1.0), 0); }
    #[test] fn test_mip_far() { let l = new_texture_lod(10); assert!(compute_mip_level(&l, 1024.0) > 0); }
    #[test] fn test_mip_clamp() { let l = new_texture_lod(3); assert!(compute_mip_level(&l, 100000.0) <= 3); }
    #[test] fn test_json() { let l = new_texture_lod(5); assert!(texture_lod_to_json(&l).contains("bias")); }
    #[test] fn test_min_level() { let l = new_texture_lod(5); assert_eq!(lod_min_level(&l), 0); }
    #[test] fn test_reset() { let mut l = new_texture_lod(5); set_lod_bias(&mut l, 3.0); lod_reset(&mut l); assert!((lod_bias_value(&l)).abs() < 1e-6); }
    #[test] fn test_bias_negative() { let mut l = new_texture_lod(5); set_lod_bias(&mut l, -1.0); assert!((lod_bias_value(&l) - (-1.0)).abs() < 1e-6); }
    #[test] fn test_mip_with_bias() { let mut l = new_texture_lod(10); set_lod_bias(&mut l, 2.0); let m1 = compute_mip_level(&l, 4.0); set_lod_bias(&mut l, 0.0); let m2 = compute_mip_level(&l, 4.0); assert!(m1 >= m2); }
}
