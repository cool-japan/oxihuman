// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub fn option_or_default_f32(opt: Option<f32>, default: f32) -> f32 {
    opt.unwrap_or(default)
}

pub fn option_map_or_zero(opt: Option<f32>) -> f32 {
    opt.unwrap_or(0.0)
}

pub fn option_filter_positive(opt: Option<f32>) -> Option<f32> {
    opt.filter(|&v| v > 0.0)
}

pub fn option_zip_with(a: Option<f32>, b: Option<f32>, f: impl Fn(f32, f32) -> f32) -> Option<f32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(f(x, y)),
        _ => None,
    }
}

pub fn option_to_result(opt: Option<f32>, msg: &str) -> Result<f32, String> {
    opt.ok_or_else(|| msg.to_string())
}

pub fn option_count(opts: &[Option<f32>]) -> usize {
    opts.iter().filter(|o| o.is_some()).count()
}

pub fn option_sum(opts: &[Option<f32>]) -> f32 {
    opts.iter().filter_map(|o| *o).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_or_default_f32() {
        /* Some returns value, None returns default */
        assert_eq!(option_or_default_f32(Some(3.0), 0.0), 3.0);
        assert_eq!(option_or_default_f32(None, 5.0), 5.0);
    }

    #[test]
    fn test_map_or_zero() {
        /* None maps to 0 */
        assert_eq!(option_map_or_zero(None), 0.0);
        assert_eq!(option_map_or_zero(Some(7.0)), 7.0);
    }

    #[test]
    fn test_filter_positive() {
        /* negative/zero filtered out */
        assert_eq!(option_filter_positive(Some(1.0)), Some(1.0));
        assert_eq!(option_filter_positive(Some(-1.0)), None);
        assert_eq!(option_filter_positive(None), None);
    }

    #[test]
    fn test_zip_with() {
        /* combines two Somes */
        assert_eq!(
            option_zip_with(Some(2.0), Some(3.0), |a, b| a + b),
            Some(5.0)
        );
        assert_eq!(option_zip_with(None, Some(3.0), |a, b| a + b), None);
    }

    #[test]
    fn test_to_result() {
        /* Some gives Ok, None gives Err */
        assert_eq!(option_to_result(Some(1.0), "err"), Ok(1.0));
        assert!(option_to_result(None, "err").is_err());
    }

    #[test]
    fn test_count() {
        /* counts Some values */
        let opts = vec![Some(1.0), None, Some(2.0)];
        assert_eq!(option_count(&opts), 2);
    }

    #[test]
    fn test_sum() {
        /* sums Some values, skips None */
        let opts = vec![Some(1.0), None, Some(2.0), Some(3.0)];
        assert!((option_sum(&opts) - 6.0).abs() < 1e-6);
    }
}
