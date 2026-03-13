// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Easing functions (linear, cubic, elastic, bounce, back, expo).

#![allow(dead_code)]

use std::f32::consts::PI;

/// Linear interpolation (no easing).
#[allow(dead_code)]
pub fn ease_linear(t: f32) -> f32 {
    t
}

/// Ease-in cubic.
#[allow(dead_code)]
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease-out cubic.
#[allow(dead_code)]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t1 = 1.0 - t;
    1.0 - t1 * t1 * t1
}

/// Ease-in-out cubic.
#[allow(dead_code)]
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t1 = -2.0 * t + 2.0;
        1.0 - t1 * t1 * t1 / 2.0
    }
}

/// Ease-in exponential.
#[allow(dead_code)]
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0f32.powf(10.0 * t - 10.0)
    }
}

/// Ease-out exponential.
#[allow(dead_code)]
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}

/// Ease-in elastic.
#[allow(dead_code)]
pub fn ease_in_elastic(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if (t - 1.0).abs() < f32::EPSILON {
        return 1.0;
    }
    let c4 = 2.0 * PI / 3.0;
    -(2.0f32.powf(10.0 * t - 10.0)) * ((10.0 * t - 10.75) * c4).sin()
}

/// Ease-out elastic.
#[allow(dead_code)]
pub fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        return 0.0;
    }
    if (t - 1.0).abs() < f32::EPSILON {
        return 1.0;
    }
    let c4 = 2.0 * PI / 3.0;
    2.0f32.powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
}

/// Ease-out bounce.
#[allow(dead_code)]
pub fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625f32;
    let d1 = 2.75f32;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t2 = t - 1.5 / d1;
        n1 * t2 * t2 + 0.75
    } else if t < 2.5 / d1 {
        let t2 = t - 2.25 / d1;
        n1 * t2 * t2 + 0.9375
    } else {
        let t2 = t - 2.625 / d1;
        n1 * t2 * t2 + 0.984375
    }
}

/// Ease-in bounce.
#[allow(dead_code)]
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Ease-in back (slight overshoot).
#[allow(dead_code)]
pub fn ease_in_back(t: f32) -> f32 {
    let c1 = 1.70158f32;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

/// Ease-out back.
#[allow(dead_code)]
pub fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158f32;
    let c3 = c1 + 1.0;
    let t1 = t - 1.0;
    1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
}

/// Evaluate any easing by name. Returns None if unknown.
#[allow(dead_code)]
pub fn ease_by_name(name: &str, t: f32) -> Option<f32> {
    match name {
        "linear" => Some(ease_linear(t)),
        "in_cubic" => Some(ease_in_cubic(t)),
        "out_cubic" => Some(ease_out_cubic(t)),
        "in_out_cubic" => Some(ease_in_out_cubic(t)),
        "in_expo" => Some(ease_in_expo(t)),
        "out_expo" => Some(ease_out_expo(t)),
        "in_elastic" => Some(ease_in_elastic(t)),
        "out_elastic" => Some(ease_out_elastic(t)),
        "in_bounce" => Some(ease_in_bounce(t)),
        "out_bounce" => Some(ease_out_bounce(t)),
        "in_back" => Some(ease_in_back(t)),
        "out_back" => Some(ease_out_back(t)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at_endpoints(f: fn(f32) -> f32) {
        assert!(f(0.0).abs() < 0.02, "f(0) should be ~0, got {}", f(0.0));
        assert!(
            (f(1.0) - 1.0).abs() < 0.02,
            "f(1) should be ~1, got {}",
            f(1.0)
        );
    }

    #[test]
    fn linear_endpoints() {
        at_endpoints(ease_linear);
    }

    #[test]
    fn cubic_endpoints() {
        at_endpoints(ease_in_cubic);
        at_endpoints(ease_out_cubic);
        at_endpoints(ease_in_out_cubic);
    }

    #[test]
    fn expo_endpoints() {
        at_endpoints(ease_in_expo);
        at_endpoints(ease_out_expo);
    }

    #[test]
    fn bounce_endpoints() {
        assert!(ease_out_bounce(0.0).abs() < 0.02);
        assert!((ease_out_bounce(1.0) - 1.0).abs() < 0.02);
    }

    #[test]
    fn ease_by_name_linear() {
        let v = ease_by_name("linear", 0.5).expect("should succeed");
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn ease_by_name_unknown_returns_none() {
        assert!(ease_by_name("banana", 0.5).is_none());
    }

    #[test]
    fn elastic_endpoints() {
        assert!(ease_in_elastic(0.0).abs() < 0.02);
        assert!((ease_in_elastic(1.0) - 1.0).abs() < 0.02);
        assert!(ease_out_elastic(0.0).abs() < 0.02);
        assert!((ease_out_elastic(1.0) - 1.0).abs() < 0.02);
    }

    #[test]
    fn back_endpoints() {
        assert!(ease_in_back(0.0).abs() < 0.02);
        assert!((ease_in_back(1.0) - 1.0).abs() < 0.02);
    }

    #[test]
    fn in_out_cubic_midpoint_near_half() {
        let v = ease_in_out_cubic(0.5);
        assert!((v - 0.5).abs() < 0.02);
    }

    #[test]
    fn all_named_easings_work() {
        let names = [
            "linear",
            "in_cubic",
            "out_cubic",
            "in_out_cubic",
            "in_expo",
            "out_expo",
            "in_elastic",
            "out_elastic",
            "in_bounce",
            "out_bounce",
            "in_back",
            "out_back",
        ];
        for name in names {
            let _v = ease_by_name(name, 0.5).expect("should succeed");
        }
    }
}
