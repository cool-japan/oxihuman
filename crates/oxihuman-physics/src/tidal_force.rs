// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Differential gravity (tidal) deformation forces.

/// Compute the tidal acceleration on a test mass at position `r` relative to
/// the primary body's center, due to a perturbing body at position `r_perturber`.
///
/// a_tidal = G * M_p * (r / |r|³ - r_relative / |r_relative|³)
/// where r_relative = r - r_perturber.
pub fn tidal_acceleration(
    r: [f64; 3],
    r_perturber: [f64; 3],
    gm_perturber: f64,
) -> [f64; 3] {
    /* direct term: force toward perturber from test mass position */
    let d1 = [r_perturber[0] - r[0], r_perturber[1] - r[1], r_perturber[2] - r[2]];
    let mag1 = mag3(d1);

    /* reference term: force toward perturber from primary center */
    let d2 = r_perturber;
    let mag2 = mag3(d2);

    if mag1 < 1e-20 || mag2 < 1e-20 {
        return [0.0; 3];
    }

    let mut a = [0.0f64; 3];
    for k in 0..3 {
        a[k] = gm_perturber * (d1[k] / (mag1 * mag1 * mag1) - d2[k] / (mag2 * mag2 * mag2));
    }
    a
}

/// Tidal deformation scale for a body of radius `R` at distance `D` from perturber.
///
/// Dimensionless tidal factor = (R/D)³.
pub fn tidal_deformation_factor(radius: f64, distance: f64) -> f64 {
    if distance < 1e-30 { return 0.0; }
    (radius / distance).powi(3)
}

/// Roche limit: minimum distance at which a satellite survives tidal forces.
///
/// d_Roche ≈ R_M * (2 * ρ_M / ρ_m)^(1/3)
pub fn roche_limit(primary_radius: f64, density_primary: f64, density_satellite: f64) -> f64 {
    if density_satellite < 1e-30 { return f64::INFINITY; }
    primary_radius * (2.0 * density_primary / density_satellite).powf(1.0 / 3.0)
}

/// Tidal heating rate estimate (simplified, W/m³).
///
/// Q_tidal ∝ e² * n⁵ * R⁵ / (G * Q_factor)
pub fn tidal_heating_rate(
    eccentricity: f64,
    mean_motion: f64,
    radius: f64,
    rigidity: f64,
    q_factor: f64,
) -> f64 {
    if q_factor < 1e-30 || rigidity < 1e-30 { return 0.0; }
    /* simplified expression */
    (21.0 / 2.0)
        * eccentricity.powi(2)
        * mean_motion.powi(5)
        * radius.powi(5)
        / (rigidity * q_factor)
}

/// Tidal locking timescale estimate (seconds).
pub fn tidal_lock_timescale(
    a: f64,
    mass_primary: f64,
    mass_satellite: f64,
    radius_satellite: f64,
    q_satellite: f64,
) -> f64 {
    if mass_primary < 1e-30 || radius_satellite < 1e-30 {
        return f64::INFINITY;
    }
    /* T_lock ∝ ω_0 * a^6 * Q / (G * M * R^5) */
    let g = 6.674e-11;
    a.powi(6) * q_satellite * mass_satellite / (g * mass_primary * radius_satellite.powi(5))
}

fn mag3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute tidal acceleration.
pub fn tf_tidal_acceleration(r: [f64; 3], r_perturber: [f64; 3], gm: f64) -> [f64; 3] {
    tidal_acceleration(r, r_perturber, gm)
}

/// Tidal deformation factor.
pub fn tf_deformation_factor(radius: f64, distance: f64) -> f64 {
    tidal_deformation_factor(radius, distance)
}

/// Roche limit.
pub fn tf_roche_limit(primary_radius: f64, density_primary: f64, density_satellite: f64) -> f64 {
    roche_limit(primary_radius, density_primary, density_satellite)
}

/// Tidal heating rate.
#[allow(clippy::too_many_arguments)]
pub fn tf_tidal_heating(e: f64, n: f64, r: f64, mu: f64, q: f64) -> f64 {
    tidal_heating_rate(e, n, r, mu, q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tidal_accel_zero_at_primary() {
        /* test mass at same position as primary center → degenerate */
        let a = tf_tidal_acceleration([0.0; 3], [1e9, 0.0, 0.0], 1e14);
        /* both terms use different distances, but r=0 gives d1=r_perturber, d2=r_perturber → zero */
        assert!(a.iter().all(|&x| x == 0.0) /* at primary center: tidal = 0 */);
    }

    #[test]
    fn test_tidal_accel_nonzero_off_center() {
        let a = tf_tidal_acceleration([1e6, 0.0, 0.0], [1e9, 0.0, 0.0], 1e14);
        assert!(a.iter().any(|&x| x.abs() > 0.0) /* off-center: nonzero tidal */);
    }

    #[test]
    fn test_deformation_factor_decreases_with_distance() {
        let f1 = tf_deformation_factor(1e6, 1e8);
        let f2 = tf_deformation_factor(1e6, 1e9);
        assert!(f1 > f2 /* closer → larger deformation */);
    }

    #[test]
    fn test_deformation_factor_at_zero_distance() {
        let f = tf_deformation_factor(1e6, 0.0);
        assert_eq!(f, 0.0 /* degenerate */);
    }

    #[test]
    fn test_roche_limit_equal_densities() {
        let r = tf_roche_limit(1e6, 1000.0, 1000.0);
        assert!((r - 1e6 * 2.0_f64.powf(1.0 / 3.0)).abs() < 1.0 /* 2^(1/3) * R */);
    }

    #[test]
    fn test_roche_limit_higher_density_satellite_smaller() {
        let r_low = tf_roche_limit(1e6, 1000.0, 500.0);
        let r_high = tf_roche_limit(1e6, 1000.0, 2000.0);
        assert!(r_low > r_high /* denser satellite → smaller Roche limit */);
    }

    #[test]
    fn test_roche_limit_zero_density_satellite_infinity() {
        let r = tf_roche_limit(1e6, 1000.0, 0.0);
        assert!(r.is_infinite() /* zero density → infinite Roche limit */);
    }

    #[test]
    fn test_tidal_heating_zero_eccentricity() {
        let h = tf_tidal_heating(0.0, 1e-4, 1e6, 1e10, 100.0);
        assert_eq!(h, 0.0 /* circular orbit → no tidal heating */);
    }

    #[test]
    fn test_tidal_heating_nonzero() {
        let h = tf_tidal_heating(0.1, 1e-4, 1e6, 1e10, 100.0);
        assert!(h > 0.0 /* eccentric orbit → tidal heating */);
    }

    #[test]
    fn test_tidal_lock_timescale_positive() {
        let t = tidal_lock_timescale(4e8, 1e26, 1e22, 1e6, 100.0);
        assert!(t > 0.0 /* positive timescale */);
    }
}
