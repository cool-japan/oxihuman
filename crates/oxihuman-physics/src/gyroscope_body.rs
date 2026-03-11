// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gyroscopic precession body simulation.

/// A rigid body with gyroscopic precession.
#[derive(Debug, Clone)]
pub struct GyroscopeBody {
    /// Angular momentum vector L.
    pub angular_momentum: [f64; 3],
    /// Principal moments of inertia [Ixx, Iyy, Izz].
    pub inertia: [f64; 3],
    /// Angular velocity.
    pub omega: [f64; 3],
}

impl GyroscopeBody {
    /// Create a new gyroscope body.
    pub fn new(inertia: [f64; 3]) -> Self {
        GyroscopeBody {
            angular_momentum: [0.0; 3],
            inertia,
            omega: [0.0; 3],
        }
    }

    /// Set the spin about the body's z-axis.
    pub fn set_spin(&mut self, spin_rate: f64) {
        self.omega[2] = spin_rate;
        self.update_angular_momentum();
    }

    /// Update angular momentum from omega (L = I * omega).
    pub fn update_angular_momentum(&mut self) {
        for k in 0..3 {
            self.angular_momentum[k] = self.inertia[k] * self.omega[k];
        }
    }

    /// Compute gyroscopic torque = omega × L.
    pub fn gyroscopic_torque(&self) -> [f64; 3] {
        cross(self.omega, self.angular_momentum)
    }

    /// Precession rate given an applied torque vector.
    ///
    /// dL/dt = torque, so omega_precession = torque / |L|
    pub fn precession_rate(&self, torque: [f64; 3]) -> f64 {
        let l_mag = mag3(self.angular_momentum);
        if l_mag < 1e-12 { return 0.0; }
        mag3(torque) / l_mag
    }

    /// Step the gyroscope under an external torque for `dt` seconds.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, torque: [f64; 3], dt: f64) {
        for k in 0..3 {
            self.angular_momentum[k] += torque[k] * dt;
        }
        /* recompute omega from L (omega = L / I) */
        for k in 0..3 {
            if self.inertia[k].abs() > 1e-30 {
                self.omega[k] = self.angular_momentum[k] / self.inertia[k];
            }
        }
    }

    /// Spin speed (magnitude of omega).
    pub fn spin_speed(&self) -> f64 {
        mag3(self.omega)
    }

    /// Magnitude of angular momentum.
    pub fn angular_momentum_mag(&self) -> f64 {
        mag3(self.angular_momentum)
    }
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

/// Create a new gyroscope body.
pub fn new_gyroscope_body(inertia: [f64; 3]) -> GyroscopeBody {
    GyroscopeBody::new(inertia)
}

/// Set spin rate about z.
pub fn gb_set_spin(g: &mut GyroscopeBody, rate: f64) {
    g.set_spin(rate);
}

/// Gyroscopic torque.
pub fn gb_gyroscopic_torque(g: &GyroscopeBody) -> [f64; 3] {
    g.gyroscopic_torque()
}

/// Precession rate.
pub fn gb_precession_rate(g: &GyroscopeBody, torque: [f64; 3]) -> f64 {
    g.precession_rate(torque)
}

/// Step.
pub fn gb_step(g: &mut GyroscopeBody, torque: [f64; 3], dt: f64) {
    g.step(torque, dt);
}

/// Spin speed.
pub fn gb_spin_speed(g: &GyroscopeBody) -> f64 {
    g.spin_speed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_zero_spin() {
        let g = new_gyroscope_body([1.0, 1.0, 1.0]);
        assert_eq!(gb_spin_speed(&g), 0.0 /* starts at rest */);
    }

    #[test]
    fn test_set_spin() {
        let mut g = new_gyroscope_body([1.0, 1.0, 2.0]);
        gb_set_spin(&mut g, 10.0);
        assert!((g.omega[2] - 10.0).abs() < 1e-9 /* spin set */);
    }

    #[test]
    fn test_angular_momentum_from_spin() {
        let mut g = new_gyroscope_body([1.0, 1.0, 5.0]);
        gb_set_spin(&mut g, 2.0);
        assert!((g.angular_momentum[2] - 10.0).abs() < 1e-9 /* L = I * omega */);
    }

    #[test]
    fn test_gyroscopic_torque_zero_when_aligned() {
        let mut g = new_gyroscope_body([1.0, 1.0, 1.0]);
        gb_set_spin(&mut g, 5.0);
        let tau = gb_gyroscopic_torque(&g);
        /* omega = [0,0,5], L = [0,0,5] → cross = [0,0,0] */
        assert!(tau.iter().all(|&x| x.abs() < 1e-9) /* no torque when aligned */);
    }

    #[test]
    fn test_step_changes_angular_momentum() {
        let mut g = new_gyroscope_body([1.0, 1.0, 1.0]);
        gb_set_spin(&mut g, 5.0);
        let torque = [1.0, 0.0, 0.0];
        gb_step(&mut g, torque, 0.1);
        assert!((g.angular_momentum[0] - 0.1).abs() < 1e-9 /* torque applied */);
    }

    #[test]
    fn test_precession_rate_zero_no_torque() {
        let mut g = new_gyroscope_body([1.0, 1.0, 2.0]);
        gb_set_spin(&mut g, 10.0);
        let rate = gb_precession_rate(&g, [0.0; 3]);
        assert_eq!(rate, 0.0 /* no torque → no precession */);
    }

    #[test]
    fn test_precession_rate_nonzero() {
        let mut g = new_gyroscope_body([1.0, 1.0, 1.0]);
        gb_set_spin(&mut g, 10.0);
        let rate = gb_precession_rate(&g, [1.0, 0.0, 0.0]);
        assert!(rate > 0.0 /* positive precession */);
    }

    #[test]
    fn test_angular_momentum_mag() {
        let mut g = new_gyroscope_body([2.0, 2.0, 2.0]);
        gb_set_spin(&mut g, 3.0);
        assert!((g.angular_momentum_mag() - 6.0).abs() < 1e-9 /* |L| = 2*3 */);
    }

    #[test]
    fn test_spin_speed() {
        let mut g = new_gyroscope_body([1.0, 1.0, 1.0]);
        gb_set_spin(&mut g, 7.0);
        assert!((gb_spin_speed(&g) - 7.0).abs() < 1e-9 /* speed = 7 */);
    }

    #[test]
    fn test_inertia_stored() {
        let g = new_gyroscope_body([3.0, 4.0, 5.0]);
        assert_eq!(g.inertia, [3.0, 4.0, 5.0] /* inertia stored */);
    }
}
