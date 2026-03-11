#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionBlendMask {
    weights: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_expr_blend_mask(vertex_count: usize) -> ExpressionBlendMask {
    ExpressionBlendMask { weights: vec![1.0; vertex_count] }
}

#[allow(dead_code)]
pub fn set_blend_region(mask: &mut ExpressionBlendMask, start: usize, end: usize, weight: f32) {
    let w = weight.clamp(0.0, 1.0);
    let end = end.min(mask.weights.len());
    for i in start..end { mask.weights[i] = w; }
}

#[allow(dead_code)]
pub fn blend_region_weight(mask: &ExpressionBlendMask, idx: usize) -> f32 {
    if idx < mask.weights.len() { mask.weights[idx] } else { 0.0 }
}

#[allow(dead_code)]
pub fn blend_mask_vertex_count(mask: &ExpressionBlendMask) -> usize { mask.weights.len() }

#[allow(dead_code)]
pub fn apply_expr_blend_mask(mask: &ExpressionBlendMask, deltas: &mut [[f32; 3]]) {
    for (i, d) in deltas.iter_mut().enumerate() {
        let w = if i < mask.weights.len() { mask.weights[i] } else { 0.0 };
        d[0] *= w; d[1] *= w; d[2] *= w;
    }
}

#[allow(dead_code)]
pub fn blend_mask_to_json_ebm(mask: &ExpressionBlendMask) -> String {
    format!("{{\"vertex_count\":{}}}", mask.weights.len())
}

#[allow(dead_code)]
pub fn invert_blend_mask_ebm(mask: &mut ExpressionBlendMask) {
    for w in mask.weights.iter_mut() { *w = 1.0 - *w; }
}

#[allow(dead_code)]
pub fn clear_blend_mask_ebm(mask: &mut ExpressionBlendMask) {
    for w in mask.weights.iter_mut() { *w = 1.0; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let m = new_expr_blend_mask(5); assert_eq!(blend_mask_vertex_count(&m), 5); }
    #[test] fn test_default_weight() { let m = new_expr_blend_mask(3); assert!((blend_region_weight(&m, 0) - 1.0).abs() < 1e-6); }
    #[test] fn test_set_region() { let mut m = new_expr_blend_mask(10); set_blend_region(&mut m, 2, 5, 0.5); assert!((blend_region_weight(&m, 3) - 0.5).abs() < 1e-6); }
    #[test] fn test_oob() { let m = new_expr_blend_mask(2); assert!((blend_region_weight(&m, 10)).abs() < 1e-6); }
    #[test] fn test_apply() { let mut m = new_expr_blend_mask(1); set_blend_region(&mut m, 0, 1, 0.5); let mut d = [[2.0, 4.0, 6.0]]; apply_expr_blend_mask(&m, &mut d); assert!((d[0][0] - 1.0).abs() < 1e-6); }
    #[test] fn test_json() { let m = new_expr_blend_mask(3); assert!(blend_mask_to_json_ebm(&m).contains("vertex_count")); }
    #[test] fn test_invert() { let mut m = new_expr_blend_mask(2); set_blend_region(&mut m, 0, 2, 0.3); invert_blend_mask_ebm(&mut m); assert!((blend_region_weight(&m, 0) - 0.7).abs() < 1e-6); }
    #[test] fn test_clear() { let mut m = new_expr_blend_mask(3); set_blend_region(&mut m, 0, 3, 0.2); clear_blend_mask_ebm(&mut m); assert!((blend_region_weight(&m, 0) - 1.0).abs() < 1e-6); }
    #[test] fn test_empty() { let m = new_expr_blend_mask(0); assert_eq!(blend_mask_vertex_count(&m), 0); }
    #[test] fn test_clamp_weight() { let mut m = new_expr_blend_mask(1); set_blend_region(&mut m, 0, 1, 5.0); assert!((blend_region_weight(&m, 0) - 1.0).abs() < 1e-6); }
}
