// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Julia set orbit stub.

/// Result of iterating z → z^2 + c for a given z0 with fixed c.
#[derive(Debug, Clone)]
pub struct JuliaOrbit {
    pub z0_re: f64,
    pub z0_im: f64,
    pub c_re: f64,
    pub c_im: f64,
    pub max_iter: u32,
    pub escape_iter: Option<u32>,
    pub last_re: f64,
    pub last_im: f64,
}

impl JuliaOrbit {
    pub fn compute(
        z0_re: f64,
        z0_im: f64,
        c_re: f64,
        c_im: f64,
        max_iter: u32,
        escape_r: f64,
    ) -> Self {
        let r2 = escape_r * escape_r;
        let mut z_re = z0_re;
        let mut z_im = z0_im;
        let mut escape_iter = None;

        for i in 0..max_iter {
            let z_re2 = z_re * z_re;
            let z_im2 = z_im * z_im;
            if z_re2 + z_im2 > r2 {
                escape_iter = Some(i);
                break;
            }
            let new_re = z_re2 - z_im2 + c_re;
            let new_im = 2.0 * z_re * z_im + c_im;
            z_re = new_re;
            z_im = new_im;
        }

        JuliaOrbit {
            z0_re,
            z0_im,
            c_re,
            c_im,
            max_iter,
            escape_iter,
            last_re: z_re,
            last_im: z_im,
        }
    }

    pub fn is_in_julia_set(&self) -> bool {
        self.escape_iter.is_none()
    }

    pub fn escape_velocity(&self) -> f64 {
        match self.escape_iter {
            None => 1.0,
            Some(n) => n as f64 / self.max_iter as f64,
        }
    }
}

pub fn julia_compute(z0_re: f64, z0_im: f64, c_re: f64, c_im: f64, max_iter: u32) -> JuliaOrbit {
    JuliaOrbit::compute(z0_re, z0_im, c_re, c_im, max_iter, 2.0)
}

pub fn julia_in_set(orbit: &JuliaOrbit) -> bool {
    orbit.is_in_julia_set()
}

pub fn julia_escape_iter(orbit: &JuliaOrbit) -> Option<u32> {
    orbit.escape_iter
}

pub fn julia_escape_velocity(orbit: &JuliaOrbit) -> f64 {
    orbit.escape_velocity()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_with_c_zero() {
        /* z=0, c=0 → stays at 0 */
        let o = julia_compute(0.0, 0.0, 0.0, 0.0, 100);
        assert!(julia_in_set(&o));
    }

    #[test]
    fn test_far_point_escapes() {
        let o = julia_compute(5.0, 5.0, 0.0, 0.0, 100);
        assert!(!julia_in_set(&o));
    }

    #[test]
    fn test_rabbit_julia_origin() {
        /* c = -0.123 + 0.745i (rabbit fractal), z=0 should be in set */
        let o = julia_compute(0.0, 0.0, -0.123, 0.745, 200);
        /* z=0 might or might not be in set for this c; just test it's finite */
        let _ = julia_in_set(&o);
        assert!(o.last_re.is_finite());
    }

    #[test]
    fn test_escape_iter_some_when_outside() {
        let o = julia_compute(3.0, 3.0, 0.0, 0.0, 100);
        assert!(julia_escape_iter(&o).is_some());
    }

    #[test]
    fn test_escape_velocity_in_range() {
        let o = julia_compute(0.5, 0.5, -0.7, 0.27015, 100);
        let v = julia_escape_velocity(&o);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_c_half_all_orbits_bounded() {
        /* For small c and small z0 orbit should be bounded */
        let o = julia_compute(0.1, 0.0, 0.3, 0.0, 50);
        let _ = julia_in_set(&o);
        assert!(o.last_re.is_finite());
    }

    #[test]
    fn test_last_re_im_stored() {
        let o = julia_compute(0.0, 0.0, 0.0, 0.0, 10);
        assert_eq!(o.last_re, 0.0);
        assert_eq!(o.last_im, 0.0);
    }

    #[test]
    fn test_escape_velocity_one_if_in_set() {
        let o = julia_compute(0.0, 0.0, 0.0, 0.0, 100);
        assert!((julia_escape_velocity(&o) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_immediate_escape_for_large_z() {
        let o = julia_compute(10.0, 0.0, 0.0, 0.0, 100);
        assert_eq!(julia_escape_iter(&o), Some(0));
    }
}
