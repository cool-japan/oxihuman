// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 2-body Kepler orbit mechanics.

use std::f64::consts::PI;

/// Keplerian orbital elements.
#[derive(Debug, Clone)]
pub struct KeplerOrbit {
    /// Semi-major axis (m).
    pub semi_major_axis: f64,
    /// Eccentricity (0 = circle, <1 = ellipse).
    pub eccentricity: f64,
    /// Gravitational parameter μ = G * (M + m).
    pub mu: f64,
}

impl KeplerOrbit {
    /// Create a new Kepler orbit.
    pub fn new(semi_major_axis: f64, eccentricity: f64, mu: f64) -> Self {
        KeplerOrbit { semi_major_axis, eccentricity: eccentricity.clamp(0.0, 0.9999), mu }
    }

    /// Orbital period (s).
    pub fn period(&self) -> f64 {
        2.0 * PI * (self.semi_major_axis.powi(3) / self.mu).sqrt()
    }

    /// Orbital speed at distance `r` from focus (vis-viva equation).
    pub fn speed_at(&self, r: f64) -> f64 {
        (self.mu * (2.0 / r - 1.0 / self.semi_major_axis)).max(0.0).sqrt()
    }

    /// Periapsis distance (closest approach).
    pub fn periapsis(&self) -> f64 {
        self.semi_major_axis * (1.0 - self.eccentricity)
    }

    /// Apoapsis distance (farthest point).
    pub fn apoapsis(&self) -> f64 {
        self.semi_major_axis * (1.0 + self.eccentricity)
    }

    /// Solve Kepler's equation M = E - e * sin(E) for eccentric anomaly E.
    pub fn eccentric_anomaly(&self, mean_anomaly: f64) -> f64 {
        let mut e = mean_anomaly;
        for _ in 0..100 {
            let de = (mean_anomaly - (e - self.eccentricity * e.sin()))
                / (1.0 - self.eccentricity * e.cos());
            e += de;
            if de.abs() < 1e-12 { break; }
        }
        e
    }

    /// Cartesian position at mean anomaly `m_anom` in the orbital plane.
    pub fn position_at(&self, m_anom: f64) -> [f64; 2] {
        let ea = self.eccentric_anomaly(m_anom);
        let x = self.semi_major_axis * (ea.cos() - self.eccentricity);
        let b = self.semi_major_axis * (1.0 - self.eccentricity * self.eccentricity).sqrt();
        let y = b * ea.sin();
        [x, y]
    }

    /// Mean motion (rad/s).
    pub fn mean_motion(&self) -> f64 {
        (self.mu / self.semi_major_axis.powi(3)).sqrt()
    }
}

/// Create a new Kepler orbit.
pub fn new_kepler_orbit(a: f64, e: f64, mu: f64) -> KeplerOrbit {
    KeplerOrbit::new(a, e, mu)
}

/// Orbital period.
pub fn ko_period(o: &KeplerOrbit) -> f64 { o.period() }

/// Speed at radius.
pub fn ko_speed_at(o: &KeplerOrbit, r: f64) -> f64 { o.speed_at(r) }

/// Periapsis.
pub fn ko_periapsis(o: &KeplerOrbit) -> f64 { o.periapsis() }

/// Apoapsis.
pub fn ko_apoapsis(o: &KeplerOrbit) -> f64 { o.apoapsis() }

/// Position in orbital plane at mean anomaly.
pub fn ko_position_at(o: &KeplerOrbit, m_anom: f64) -> [f64; 2] { o.position_at(m_anom) }

/// Mean motion.
pub fn ko_mean_motion(o: &KeplerOrbit) -> f64 { o.mean_motion() }

#[cfg(test)]
mod tests {
    use super::*;

    const MU_EARTH: f64 = 3.986e14; /* Earth's gravitational parameter */

    #[test]
    fn test_circular_orbit_period() {
        let r = 7_000_000.0f64; /* LEO ~7000 km */
        let o = new_kepler_orbit(r, 0.0, MU_EARTH);
        let t = ko_period(&o);
        assert!((t - 5825.0).abs() < 50.0 /* roughly 97 min */);
    }

    #[test]
    fn test_periapsis_circle() {
        let o = new_kepler_orbit(7e6, 0.0, MU_EARTH);
        assert!((ko_periapsis(&o) - 7e6).abs() < 1.0 /* circle: peri = a */);
    }

    #[test]
    fn test_apoapsis_circle() {
        let o = new_kepler_orbit(7e6, 0.0, MU_EARTH);
        assert!((ko_apoapsis(&o) - 7e6).abs() < 1.0 /* circle: apo = a */);
    }

    #[test]
    fn test_periapsis_ellipse() {
        let o = new_kepler_orbit(10e6, 0.3, MU_EARTH);
        assert!((ko_periapsis(&o) - 7e6).abs() < 1.0 /* 10e6*(1-0.3) */);
    }

    #[test]
    fn test_speed_at_periapsis_is_max() {
        let o = new_kepler_orbit(10e6, 0.3, MU_EARTH);
        let v_peri = ko_speed_at(&o, ko_periapsis(&o));
        let v_apo = ko_speed_at(&o, ko_apoapsis(&o));
        assert!(v_peri > v_apo /* faster at periapsis */);
    }

    #[test]
    fn test_eccentric_anomaly_zero() {
        let o = new_kepler_orbit(7e6, 0.1, MU_EARTH);
        let ea = o.eccentric_anomaly(0.0);
        assert!(ea.abs() < 1e-10 /* M=0 → E=0 */);
    }

    #[test]
    fn test_position_at_zero_anomaly() {
        let o = new_kepler_orbit(7e6, 0.0, MU_EARTH);
        let [x, y] = ko_position_at(&o, 0.0);
        /* circular orbit, M=0 → x=a, y=0 */
        assert!((x - 7e6).abs() < 1.0 /* at periapsis */);
        assert!(y.abs() < 1.0);
    }

    #[test]
    fn test_mean_motion() {
        let o = new_kepler_orbit(7e6, 0.0, MU_EARTH);
        let n = ko_mean_motion(&o);
        assert!((n - 2.0 * PI / ko_period(&o)).abs() < 1e-9 /* n = 2π/T */);
    }

    #[test]
    fn test_eccentricity_clamped() {
        let o = new_kepler_orbit(7e6, 1.5, MU_EARTH);
        assert!(o.eccentricity <= 0.9999 /* clamped */);
    }

    #[test]
    fn test_position_at_half_period() {
        let o = new_kepler_orbit(7e6, 0.0, MU_EARTH);
        let [x, _y] = ko_position_at(&o, PI);
        /* M=π → opposite side of circle: x ≈ -a */
        assert!(x < 0.0 /* on far side */);
    }
}
