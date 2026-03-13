// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Animation easing functions.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Linear easing: t passes through unchanged.
#[allow(dead_code)]
pub fn ease_linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

/// Quadratic ease-in.
#[allow(dead_code)]
pub fn ease_in_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

/// Quadratic ease-out.
#[allow(dead_code)]
pub fn ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Quadratic ease-in-out.
#[allow(dead_code)]
pub fn ease_in_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) * 0.5
    }
}

/// Cubic ease-in.
#[allow(dead_code)]
pub fn ease_in_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

/// Cubic ease-out.
#[allow(dead_code)]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Sinusoidal ease-in-out.
#[allow(dead_code)]
pub fn ease_in_out_sine(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    -((PI * t).cos() - 1.0) * 0.5
}

/// Bounce ease-out.
#[allow(dead_code)]
pub fn ease_bounce_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let n1 = 7.5625_f32;
    let d1 = 2.75_f32;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

/// Elastic ease-in (spec name: ease_in_elastic).
#[allow(dead_code)]
pub fn ease_in_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return 0.0;
    }
    if (t - 1.0).abs() < 1e-9 {
        return 1.0;
    }
    let c4 = 2.0 * PI / 3.0;
    -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin()
}

/// Bounce ease-out (spec name: ease_out_bounce, alias for ease_bounce_out).
#[allow(dead_code)]
pub fn ease_out_bounce(t: f32) -> f32 {
    ease_bounce_out(t)
}

/// Elastic ease-out.
#[allow(dead_code)]
pub fn ease_elastic_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return 0.0;
    }
    if (t - 1.0).abs() < 1e-9 {
        return 1.0;
    }
    let c4 = 2.0 * PI / 3.0;
    2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn test_linear_endpoints() {
        assert!((ease_linear(0.0)).abs() < EPS);
        assert!((ease_linear(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_in_quad_endpoints() {
        assert!((ease_in_quad(0.0)).abs() < EPS);
        assert!((ease_in_quad(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_out_quad_endpoints() {
        assert!((ease_out_quad(0.0)).abs() < EPS);
        assert!((ease_out_quad(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_in_out_quad_midpoint() {
        // At t=0.5 should equal 0.5 for symmetric ease
        assert!((ease_in_out_quad(0.5) - 0.5).abs() < EPS);
    }

    #[test]
    fn test_in_cubic_monotone() {
        let a = ease_in_cubic(0.3);
        let b = ease_in_cubic(0.6);
        assert!(a < b);
    }

    #[test]
    fn test_out_cubic_endpoints() {
        assert!((ease_out_cubic(0.0)).abs() < EPS);
        assert!((ease_out_cubic(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_sine_endpoints() {
        assert!((ease_in_out_sine(0.0)).abs() < EPS);
        assert!((ease_in_out_sine(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_bounce_endpoints() {
        assert!((ease_bounce_out(0.0)).abs() < EPS);
        assert!((ease_bounce_out(1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_elastic_endpoints() {
        assert!((ease_elastic_out(0.0)).abs() < EPS);
        assert!((ease_elastic_out(1.0) - 1.0).abs() < EPS);
    }
}
