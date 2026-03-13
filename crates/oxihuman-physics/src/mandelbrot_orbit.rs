// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mandelbrot set orbit stub.

/// Result of iterating a point c in the Mandelbrot set.
#[derive(Debug, Clone)]
pub struct MandelbrotOrbit {
    pub c_re: f64,
    pub c_im: f64,
    pub max_iter: u32,
    pub escape_iter: Option<u32>,
    pub last_re: f64,
    pub last_im: f64,
}

impl MandelbrotOrbit {
    pub fn compute(c_re: f64, c_im: f64, max_iter: u32, escape_radius: f64) -> Self {
        let r2 = escape_radius * escape_radius;
        let mut z_re = 0.0_f64;
        let mut z_im = 0.0_f64;
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

        MandelbrotOrbit {
            c_re,
            c_im,
            max_iter,
            escape_iter,
            last_re: z_re,
            last_im: z_im,
        }
    }

    pub fn is_in_set(&self) -> bool {
        self.escape_iter.is_none()
    }

    pub fn escape_velocity(&self) -> f64 {
        match self.escape_iter {
            None => 1.0,
            Some(n) => n as f64 / self.max_iter as f64,
        }
    }

    pub fn modulus_sq(&self) -> f64 {
        self.last_re * self.last_re + self.last_im * self.last_im
    }
}

pub fn mandelbrot_compute(c_re: f64, c_im: f64, max_iter: u32) -> MandelbrotOrbit {
    MandelbrotOrbit::compute(c_re, c_im, max_iter, 2.0)
}

pub fn mandelbrot_in_set(orbit: &MandelbrotOrbit) -> bool {
    orbit.is_in_set()
}

pub fn mandelbrot_escape_iter(orbit: &MandelbrotOrbit) -> Option<u32> {
    orbit.escape_iter
}

pub fn mandelbrot_escape_velocity(orbit: &MandelbrotOrbit) -> f64 {
    orbit.escape_velocity()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_in_set() {
        /* c=0 → z stays at 0 forever */
        let o = mandelbrot_compute(0.0, 0.0, 100);
        assert!(mandelbrot_in_set(&o));
    }

    #[test]
    fn test_far_outside_set() {
        /* c=2+2i is clearly outside */
        let o = mandelbrot_compute(2.0, 2.0, 100);
        assert!(!mandelbrot_in_set(&o));
    }

    #[test]
    fn test_minus_one_in_set() {
        /* c = -1 is in the set */
        let o = mandelbrot_compute(-1.0, 0.0, 1000);
        assert!(mandelbrot_in_set(&o));
    }

    #[test]
    fn test_escape_iter_some_when_outside() {
        let o = mandelbrot_compute(3.0, 0.0, 100);
        assert!(mandelbrot_escape_iter(&o).is_some());
    }

    #[test]
    fn test_escape_velocity_zero_to_one() {
        let o = mandelbrot_compute(0.5, 0.5, 100);
        let v = mandelbrot_escape_velocity(&o);
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn test_in_set_escape_velocity_one() {
        let o = mandelbrot_compute(0.0, 0.0, 100);
        assert!((mandelbrot_escape_velocity(&o) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_modulus_sq_non_negative() {
        let o = mandelbrot_compute(0.5, 0.3, 50);
        assert!(o.modulus_sq() >= 0.0);
    }

    #[test]
    fn test_bulb_center_in_set() {
        /* c = -0.5 is in main cardioid */
        let o = mandelbrot_compute(-0.5, 0.0, 200);
        assert!(mandelbrot_in_set(&o));
    }

    #[test]
    fn test_escape_early_for_far_point() {
        /* c=10 is far outside; escapes at iteration 0 or 1 */
        let o = mandelbrot_compute(10.0, 0.0, 100);
        let iter = mandelbrot_escape_iter(&o);
        assert!(iter.is_some());
        assert!(iter.expect("should succeed") <= 1);
    }
}
