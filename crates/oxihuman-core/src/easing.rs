// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f64::consts::PI;

#[allow(dead_code)]
pub fn ease_in_quad(t: f64) -> f64 {
    t * t
}

#[allow(dead_code)]
pub fn ease_out_quad(t: f64) -> f64 {
    t * (2.0 - t)
}

#[allow(dead_code)]
pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

#[allow(dead_code)]
pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}

#[allow(dead_code)]
pub fn ease_out_cubic(t: f64) -> f64 {
    let t1 = t - 1.0;
    t1 * t1 * t1 + 1.0
}

#[allow(dead_code)]
pub fn ease_in_expo(t: f64) -> f64 {
    if t == 0.0 {
        0.0
    } else {
        (2.0f64).powf(10.0 * t - 10.0)
    }
}

#[allow(dead_code)]
pub fn ease_in_elastic(t: f64) -> f64 {
    if t == 0.0 {
        return 0.0;
    }
    if t == 1.0 {
        return 1.0;
    }
    let c4 = (2.0 * PI) / 3.0;
    -(2.0f64).powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_in_quad_zero() {
        assert!((ease_in_quad(0.0)).abs() < 1e-10);
    }

    #[test]
    fn test_ease_in_quad_one() {
        assert!((ease_in_quad(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_out_quad_endpoints() {
        assert!((ease_out_quad(0.0)).abs() < 1e-10);
        assert!((ease_out_quad(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_in_out_quad_endpoints() {
        assert!((ease_in_out_quad(0.0)).abs() < 1e-10);
        assert!((ease_in_out_quad(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_in_cubic_endpoints() {
        assert!((ease_in_cubic(0.0)).abs() < 1e-10);
        assert!((ease_in_cubic(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_out_cubic_endpoints() {
        assert!((ease_out_cubic(0.0)).abs() < 1e-10);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_in_expo_endpoints() {
        assert!((ease_in_expo(0.0)).abs() < 1e-10);
        assert!((ease_in_expo(1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ease_in_quad_monotone() {
        let v1 = ease_in_quad(0.3);
        let v2 = ease_in_quad(0.6);
        assert!(v2 > v1);
    }

    #[test]
    fn test_ease_in_out_quad_midpoint() {
        assert!((ease_in_out_quad(0.5) - 0.5).abs() < 1e-10);
    }
}
