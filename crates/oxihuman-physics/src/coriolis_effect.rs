// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Coriolis acceleration in a rotating reference frame.

/// Coriolis acceleration: a_cor = -2 * ω × v.
pub fn coriolis_accel(omega: [f64; 3], velocity: [f64; 3]) -> [f64; 3] {
    let w_cross_v = cross(omega, velocity);
    [-2.0 * w_cross_v[0], -2.0 * w_cross_v[1], -2.0 * w_cross_v[2]]
}

/// Coriolis force: F_cor = m * a_cor.
pub fn coriolis_force(omega: [f64; 3], velocity: [f64; 3], mass: f64) -> [f64; 3] {
    let a = coriolis_accel(omega, velocity);
    [a[0] * mass, a[1] * mass, a[2] * mass]
}

/// Deflection distance for a particle travelling at `speed` for `time` in a frame
/// rotating at `omega_z` rad/s (2D approximation).
pub fn coriolis_deflection_2d(speed: f64, time: f64, omega_z: f64) -> f64 {
    /* Δ ≈ omega_z * speed * time² */
    omega_z.abs() * speed * time * time
}

/// Effective rotation vector at latitude `lat_rad` on a body rotating at `omega`.
pub fn effective_omega_at_latitude(omega: f64, lat_rad: f64) -> [f64; 3] {
    /* horizontal: omega * cos(lat), vertical: omega * sin(lat) */
    [0.0, omega * lat_rad.sin(), omega * lat_rad.cos()]
}

/// True if the Coriolis force deflects motion to the right in the Northern Hemisphere.
/// Returns true for omega_z > 0 (Northern Hemisphere convention).
pub fn deflects_right(omega_z: f64) -> bool {
    omega_z > 0.0
}

/// Magnitude of the Coriolis acceleration.
pub fn coriolis_accel_magnitude(omega: [f64; 3], velocity: [f64; 3]) -> f64 {
    let a = coriolis_accel(omega, velocity);
    mag3(a)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn mag3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_zero_omega_zero_coriolis() {
        let a = coriolis_accel([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(a.iter().all(|&x| x == 0.0) /* no rotation → no Coriolis */);
    }

    #[test]
    fn test_zero_velocity_zero_coriolis() {
        let a = coriolis_accel([0.0, 0.0, 1.0], [0.0; 3]);
        assert!(a.iter().all(|&x| x == 0.0) /* no motion → no Coriolis */);
    }

    #[test]
    fn test_deflection_right_northern() {
        /* ω = [0,0,1] (N. hemisphere), v = [1,0,0] (east) */
        /* a_cor = -2*[0,0,1]×[1,0,0] = -2*[0,1,0] × wait — let's compute */
        let a = coriolis_accel([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        /* [0,0,1]×[1,0,0] = [0*0-1*0, 1*1-0*0, 0*0-0*1] = [0,1,0] */
        /* a_cor = -2*[0,1,0] = [0,-2,0] — deflects south (right in N. hemisphere) */
        assert!(a[1] < 0.0 /* deflects to the right (southward) */);
    }

    #[test]
    fn test_coriolis_force_scaled() {
        let omega = [0.0, 0.0, 1.0];
        let vel = [1.0, 0.0, 0.0];
        let a = coriolis_accel(omega, vel);
        let f = coriolis_force(omega, vel, 2.0);
        assert!((f[0] - a[0] * 2.0).abs() < 1e-9 /* F = m * a */);
    }

    #[test]
    fn test_deflects_right_function() {
        assert!(deflects_right(1.0) /* positive ω → Northern Hemisphere */);
        assert!(!deflects_right(-1.0) /* negative → Southern */);
    }

    #[test]
    fn test_deflection_2d_positive() {
        let d = coriolis_deflection_2d(10.0, 2.0, 0.1);
        assert!(d > 0.0 /* non-zero deflection */);
    }

    #[test]
    fn test_deflection_zero_no_motion() {
        let d = coriolis_deflection_2d(0.0, 10.0, 1.0);
        assert_eq!(d, 0.0 /* no velocity → no deflection */);
    }

    #[test]
    fn test_effective_omega_equator() {
        let eo = effective_omega_at_latitude(7.27e-5, 0.0);
        /* at equator: vertical = 0, horizontal = omega */
        assert!(eo[1].abs() < 1e-12 /* zero vertical at equator */);
    }

    #[test]
    fn test_effective_omega_pole() {
        let eo = effective_omega_at_latitude(7.27e-5, PI / 2.0);
        assert!((eo[1] - 7.27e-5).abs() < 1e-10 /* full omega vertical at pole */);
    }

    #[test]
    fn test_coriolis_magnitude_nonzero() {
        let mag = coriolis_accel_magnitude([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        assert!((mag - 2.0).abs() < 1e-9 /* |a| = 2*|ω|*|v|*sin(90°) = 2 */);
    }
}
