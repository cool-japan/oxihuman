#![allow(dead_code)]
//! Deterministic parameter randomization for morph variation.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ParamRandomizer {
    seed: u64,
    min: f32,
    max: f32,
    count: usize,
}

#[allow(dead_code)]
pub fn new_param_randomizer(min: f32, max: f32) -> ParamRandomizer {
    ParamRandomizer {
        seed: 42,
        min,
        max,
        count: 0,
    }
}

#[allow(dead_code)]
pub fn randomize_in_range(r: &mut ParamRandomizer) -> f32 {
    r.seed = r.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    r.count += 1;
    let frac = ((r.seed >> 33) as f32) / (u32::MAX as f32);
    r.min + (r.max - r.min) * frac.clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn randomize_gaussian_stub(r: &mut ParamRandomizer) -> f32 {
    // Deterministic stub: returns midpoint biased value
    let v = randomize_in_range(r);
    let mid = (r.min + r.max) * 0.5;
    mid + (v - mid) * 0.5
}

#[allow(dead_code)]
pub fn seed_randomizer(r: &mut ParamRandomizer, seed: u64) {
    r.seed = seed;
    r.count = 0;
}

#[allow(dead_code)]
pub fn param_min(r: &ParamRandomizer) -> f32 {
    r.min
}

#[allow(dead_code)]
pub fn param_max(r: &ParamRandomizer) -> f32 {
    r.max
}

#[allow(dead_code)]
pub fn randomized_count(r: &ParamRandomizer) -> usize {
    r.count
}

#[allow(dead_code)]
pub fn randomizer_to_json(r: &ParamRandomizer) -> String {
    format!(
        "{{\"seed\":{},\"min\":{},\"max\":{},\"count\":{}}}",
        r.seed, r.min, r.max, r.count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_param_randomizer() {
        let r = new_param_randomizer(0.0, 1.0);
        assert!((param_min(&r)).abs() < 1e-6);
        assert!((param_max(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_randomize_in_range() {
        let mut r = new_param_randomizer(0.0, 1.0);
        let v = randomize_in_range(&mut r);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_randomize_gaussian_stub() {
        let mut r = new_param_randomizer(0.0, 1.0);
        let v = randomize_gaussian_stub(&mut r);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_seed_randomizer() {
        let mut r = new_param_randomizer(0.0, 1.0);
        seed_randomizer(&mut r, 123);
        let v1 = randomize_in_range(&mut r);
        seed_randomizer(&mut r, 123);
        let v2 = randomize_in_range(&mut r);
        assert!((v1 - v2).abs() < 1e-6);
    }

    #[test]
    fn test_randomized_count() {
        let mut r = new_param_randomizer(0.0, 1.0);
        assert_eq!(randomized_count(&r), 0);
        randomize_in_range(&mut r);
        assert_eq!(randomized_count(&r), 1);
    }

    #[test]
    fn test_randomizer_to_json() {
        let r = new_param_randomizer(0.0, 1.0);
        let json = randomizer_to_json(&r);
        assert!(json.contains("\"seed\":"));
    }

    #[test]
    fn test_range_min_max() {
        let mut r = new_param_randomizer(5.0, 10.0);
        for _ in 0..20 {
            let v = randomize_in_range(&mut r);
            assert!((5.0..=10.0).contains(&v));
        }
    }

    #[test]
    fn test_param_min() {
        let r = new_param_randomizer(-1.0, 1.0);
        assert!((param_min(&r) - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_param_max() {
        let r = new_param_randomizer(0.0, 2.0);
        assert!((param_max(&r) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_deterministic() {
        let mut r1 = new_param_randomizer(0.0, 1.0);
        let mut r2 = new_param_randomizer(0.0, 1.0);
        for _ in 0..10 {
            assert!((randomize_in_range(&mut r1) - randomize_in_range(&mut r2)).abs() < 1e-6);
        }
    }
}
