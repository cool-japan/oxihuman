// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Mitchell-Netravali reconstruction filter.
pub fn mitchell_netravali(x: f32, b: f32, c: f32) -> f32 {
    let ax = x.abs();
    if ax < 1.0 {
        ((12.0 - 9.0 * b - 6.0 * c) * ax * ax * ax
            + (-18.0 + 12.0 * b + 6.0 * c) * ax * ax
            + (6.0 - 2.0 * b))
            / 6.0
    } else if ax < 2.0 {
        ((-b - 6.0 * c) * ax * ax * ax
            + (6.0 * b + 30.0 * c) * ax * ax
            + (-12.0 * b - 48.0 * c) * ax
            + (8.0 * b + 24.0 * c))
            / 6.0
    } else {
        0.0
    }
}

/// Lanczos sinc reconstruction filter.
pub fn lanczos(x: f32, lobes: f32) -> f32 {
    use std::f32::consts::PI;
    if x.abs() < 1e-6 {
        return 1.0;
    }
    if x.abs() >= lobes {
        return 0.0;
    }
    let px = PI * x;
    (px.sin() / px) * ((px / lobes).sin() / (px / lobes))
}

/// Box filter (nearest-neighbour equivalent).
pub fn box_filter(x: f32) -> f32 {
    if x.abs() <= 0.5 {
        1.0
    } else {
        0.0
    }
}

/// Gaussian filter.
pub fn gaussian_filter(x: f32, sigma: f32) -> f32 {
    use std::f32::consts::PI;
    (-(x * x) / (2.0 * sigma * sigma)).exp() / (sigma * (2.0 * PI).sqrt())
}

pub fn filter_is_separable(_name: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mitchell_center() {
        /* mitchell at x=0 should be (6-2b)/6 for default b=1/3, c=1/3 */
        let v = mitchell_netravali(0.0, 1.0 / 3.0, 1.0 / 3.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_mitchell_zero_outside() {
        /* zero for |x| >= 2 */
        let v = mitchell_netravali(3.0, 1.0 / 3.0, 1.0 / 3.0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_lanczos_center() {
        /* lanczos at x=0 is 1 */
        let v = lanczos(0.0, 3.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_box_filter() {
        /* box filter at center */
        assert!((box_filter(0.0) - 1.0).abs() < 1e-6);
        assert!(box_filter(1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gaussian_positive() {
        /* gaussian is always positive */
        let g = gaussian_filter(0.5, 0.5);
        assert!(g > 0.0);
    }
}
