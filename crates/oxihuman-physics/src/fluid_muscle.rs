// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fluid-driven muscle actuator (McKibben / pneumatic artificial muscle stub).

/// A fluid-driven (pneumatic or hydraulic) muscle actuator.
#[derive(Debug, Clone)]
pub struct FluidMuscle {
    /// Braid angle at rest (radians).
    pub braid_angle: f32,
    /// Rest length (m).
    pub rest_len: f32,
    /// Current length (m).
    pub current_len: f32,
    /// Internal gauge pressure (Pa).
    pub pressure: f32,
    /// Inner tube radius (m).
    pub radius: f32,
    /// Number of braid turns.
    pub braid_turns: f32,
}

impl FluidMuscle {
    pub fn new(rest_len: f32, radius: f32) -> Self {
        FluidMuscle {
            braid_angle: 54.7_f32.to_radians(), /* ~magic angle */
            rest_len,
            current_len: rest_len,
            pressure: 0.0,
            radius,
            braid_turns: 6.0,
        }
    }
}

/// Create a new fluid muscle.
pub fn new_fluid_muscle(rest_len: f32, radius: f32) -> FluidMuscle {
    FluidMuscle::new(rest_len, radius)
}

/// Compute the McKibben contractile force (N) at given length.
/// Force = π r² P (3 cos²θ – 1) / sin²θ
pub fn fluid_muscle_force(m: &FluidMuscle) -> f32 {
    let area = std::f32::consts::PI * m.radius * m.radius;
    let theta = m.braid_angle;
    let cos2 = theta.cos().powi(2);
    let sin2 = theta.sin().powi(2);
    area * m.pressure * (3.0 * cos2 - 1.0) / sin2.max(1e-10)
}

/// Set the supply pressure (Pa).
pub fn fm_set_pressure(m: &mut FluidMuscle, pressure: f32) {
    m.pressure = pressure.max(0.0);
}

/// Update braid angle from current length (assumes constant braid length).
pub fn fm_update_angle(m: &mut FluidMuscle) {
    /* L = n * b * cos(theta); b = rest_len / (n * cos(angle0)) */
    let n = m.braid_turns;
    let b = m.rest_len / (n * m.braid_angle.cos()).max(1e-10);
    let cos_theta = m.current_len / (n * b).max(1e-10);
    m.braid_angle = cos_theta.clamp(-1.0, 1.0).acos();
}

/// Return the contraction ratio (shortening / rest length).
pub fn fm_contraction_ratio(m: &FluidMuscle) -> f32 {
    (m.rest_len - m.current_len) / m.rest_len.max(1e-10)
}

/// Return `true` if the muscle is contracting (shorter than rest).
pub fn fm_is_contracting(m: &FluidMuscle) -> bool {
    m.current_len < m.rest_len - 1e-6
}

/// Return the blocked force (maximum force at zero contraction).
pub fn fm_blocked_force(m: &FluidMuscle) -> f32 {
    let area = std::f32::consts::PI * m.radius * m.radius;
    /* simplified: F_blocked ≈ π r² P */
    area * m.pressure
}

/// Set the current length.
pub fn fm_set_length(m: &mut FluidMuscle, len: f32) {
    m.current_len = len.max(0.0);
    fm_update_angle(m);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_muscle_at_rest() {
        let m = new_fluid_muscle(0.2, 0.01);
        assert!(!fm_is_contracting(&m));
    }

    #[test]
    fn test_force_zero_at_zero_pressure() {
        let m = new_fluid_muscle(0.2, 0.01);
        assert!(fluid_muscle_force(&m).abs() < 1e-3);
    }

    #[test]
    fn test_force_positive_under_pressure() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_pressure(&mut m, 300_000.0); /* 3 bar */
        assert!(fluid_muscle_force(&m) > 0.0);
    }

    #[test]
    fn test_contraction_ratio_zero_at_rest() {
        let m = new_fluid_muscle(0.2, 0.01);
        assert!(fm_contraction_ratio(&m).abs() < 1e-5);
    }

    #[test]
    fn test_contraction_ratio_positive_when_short() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_length(&mut m, 0.18);
        assert!(fm_contraction_ratio(&m) > 0.0);
    }

    #[test]
    fn test_is_contracting() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_length(&mut m, 0.15);
        assert!(fm_is_contracting(&m));
    }

    #[test]
    fn test_blocked_force_positive() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_pressure(&mut m, 200_000.0);
        assert!(fm_blocked_force(&m) > 0.0);
    }

    #[test]
    fn test_pressure_clamp_nonnegative() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_pressure(&mut m, -1000.0);
        assert_eq!(m.pressure, 0.0);
    }

    #[test]
    fn test_update_angle_does_not_panic() {
        let mut m = new_fluid_muscle(0.2, 0.01);
        fm_set_length(&mut m, 0.19);
        assert!(m.braid_angle >= 0.0 && m.braid_angle <= std::f32::consts::PI);
    }
}
