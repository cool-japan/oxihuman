#![allow(dead_code)]

/// A parameter range with min/max bounds.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ParamRange {
    pub min: f32,
    pub max: f32,
}

#[allow(dead_code)]
pub fn new_param_range(min: f32, max: f32) -> ParamRange {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    ParamRange { min: lo, max: hi }
}

#[allow(dead_code)]
pub fn range_min(r: &ParamRange) -> f32 { r.min }

#[allow(dead_code)]
pub fn range_max(r: &ParamRange) -> f32 { r.max }

#[allow(dead_code)]
pub fn range_contains(r: &ParamRange, val: f32) -> bool {
    (r.min..=r.max).contains(&val)
}

#[allow(dead_code)]
pub fn range_clamp(r: &ParamRange, val: f32) -> f32 {
    val.clamp(r.min, r.max)
}

#[allow(dead_code)]
pub fn range_normalize(r: &ParamRange, val: f32) -> f32 {
    let span = r.max - r.min;
    if span.abs() < 1e-9 { return 0.0; }
    (val - r.min) / span
}

#[allow(dead_code)]
pub fn range_lerp(r: &ParamRange, t: f32) -> f32 {
    r.min + (r.max - r.min) * t
}

#[allow(dead_code)]
pub fn range_to_json(r: &ParamRange) -> String {
    format!("{{\"min\":{:.4},\"max\":{:.4}}}", r.min, r.max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = new_param_range(0.0, 1.0);
        assert!((r.min).abs() < 1e-6);
        assert!((r.max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_reversed() {
        let r = new_param_range(5.0, 2.0);
        assert!((r.min - 2.0).abs() < 1e-6);
        assert!((r.max - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_contains() {
        let r = new_param_range(0.0, 1.0);
        assert!(range_contains(&r, 0.5));
        assert!(range_contains(&r, 0.0));
        assert!(range_contains(&r, 1.0));
        assert!(!range_contains(&r, 1.1));
    }

    #[test]
    fn test_clamp() {
        let r = new_param_range(0.0, 1.0);
        assert!((range_clamp(&r, -0.5)).abs() < 1e-6);
        assert!((range_clamp(&r, 1.5) - 1.0).abs() < 1e-6);
        assert!((range_clamp(&r, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let r = new_param_range(0.0, 10.0);
        assert!((range_normalize(&r, 5.0) - 0.5).abs() < 1e-6);
        assert!((range_normalize(&r, 0.0)).abs() < 1e-6);
        assert!((range_normalize(&r, 10.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lerp() {
        let r = new_param_range(0.0, 10.0);
        assert!((range_lerp(&r, 0.0)).abs() < 1e-6);
        assert!((range_lerp(&r, 0.5) - 5.0).abs() < 1e-6);
        assert!((range_lerp(&r, 1.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = new_param_range(0.0, 1.0);
        let j = range_to_json(&r);
        assert!(j.contains("min"));
        assert!(j.contains("max"));
    }

    #[test]
    fn test_min_max_accessors() {
        let r = new_param_range(3.0, 7.0);
        assert!((range_min(&r) - 3.0).abs() < 1e-6);
        assert!((range_max(&r) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_span() {
        let r = new_param_range(5.0, 5.0);
        assert!((range_normalize(&r, 5.0)).abs() < 1e-6);
    }

    #[test]
    fn test_lerp_endpoints() {
        let r = new_param_range(-1.0, 1.0);
        assert!((range_lerp(&r, 0.0) - (-1.0)).abs() < 1e-6);
        assert!((range_lerp(&r, 1.0) - 1.0).abs() < 1e-6);
    }
}
