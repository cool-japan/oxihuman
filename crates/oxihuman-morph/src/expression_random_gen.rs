#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionRandomGen {
    seed: u64,
    min_intensity: f32,
    max_intensity: f32,
    count: usize,
    last: Vec<f32>,
}

fn lcg(s: u64) -> u64 { s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) }
fn lcg_f32(s: &mut u64) -> f32 { *s = lcg(*s); ((*s >> 33) as f32) / (u32::MAX as f32) }

#[allow(dead_code)]
pub fn new_expression_random_gen(seed: u64, count: usize, min_i: f32, max_i: f32) -> ExpressionRandomGen {
    ExpressionRandomGen { seed, min_intensity: min_i, max_intensity: max_i, count, last: vec![0.0; count] }
}

#[allow(dead_code)]
pub fn generate_expression(gen: &mut ExpressionRandomGen) -> Vec<f32> {
    let mut result = Vec::with_capacity(gen.count);
    let range = gen.max_intensity - gen.min_intensity;
    for _ in 0..gen.count {
        let v = gen.min_intensity + lcg_f32(&mut gen.seed) * range;
        result.push(v);
    }
    gen.last = result.clone();
    result
}

#[allow(dead_code)]
pub fn gen_seed(gen: &ExpressionRandomGen) -> u64 { gen.seed }

#[allow(dead_code)]
pub fn gen_intensity_range(gen: &ExpressionRandomGen) -> (f32, f32) { (gen.min_intensity, gen.max_intensity) }

#[allow(dead_code)]
pub fn gen_count(gen: &ExpressionRandomGen) -> usize { gen.count }

#[allow(dead_code)]
pub fn gen_to_json(gen: &ExpressionRandomGen) -> String {
    format!("{{\"seed\":{},\"count\":{}}}", gen.seed, gen.count)
}

#[allow(dead_code)]
pub fn gen_reset(gen: &mut ExpressionRandomGen) {
    gen.last = vec![0.0; gen.count];
}

#[allow(dead_code)]
pub fn gen_last_expression(gen: &ExpressionRandomGen) -> &[f32] { &gen.last }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let g = new_expression_random_gen(42, 5, 0.0, 1.0); assert_eq!(gen_count(&g), 5); }
    #[test] fn test_generate() { let mut g = new_expression_random_gen(42, 3, 0.0, 1.0); let v = generate_expression(&mut g); assert_eq!(v.len(), 3); }
    #[test] fn test_range() { let mut g = new_expression_random_gen(42, 10, 0.2, 0.8); let v = generate_expression(&mut g); for x in &v { assert!((0.0..=1.0).contains(x)); } }
    #[test] fn test_seed() { let g = new_expression_random_gen(123, 1, 0.0, 1.0); assert_eq!(gen_seed(&g), 123); }
    #[test] fn test_intensity_range() { let g = new_expression_random_gen(1, 1, 0.1, 0.9); assert!((gen_intensity_range(&g).0 - 0.1).abs() < 1e-6); }
    #[test] fn test_json() { let g = new_expression_random_gen(1, 1, 0.0, 1.0); assert!(gen_to_json(&g).contains("seed")); }
    #[test] fn test_reset() { let mut g = new_expression_random_gen(1, 3, 0.0, 1.0); generate_expression(&mut g); gen_reset(&mut g); assert!((gen_last_expression(&g)[0]).abs() < 1e-6); }
    #[test] fn test_last() { let mut g = new_expression_random_gen(42, 2, 0.0, 1.0); generate_expression(&mut g); assert_eq!(gen_last_expression(&g).len(), 2); }
    #[test] fn test_deterministic() { let mut g1 = new_expression_random_gen(42, 3, 0.0, 1.0); let mut g2 = new_expression_random_gen(42, 3, 0.0, 1.0); assert_eq!(generate_expression(&mut g1), generate_expression(&mut g2)); }
    #[test] fn test_count_zero() { let g = new_expression_random_gen(1, 0, 0.0, 1.0); assert_eq!(gen_count(&g), 0); }
}
