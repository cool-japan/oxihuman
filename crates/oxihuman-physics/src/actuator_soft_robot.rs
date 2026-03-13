// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Soft robot pneumatic stub — bellows/chamber-based soft actuator.

/// Soft robot actuator (pneumatic bellows) parameters.
#[derive(Clone, Debug)]
pub struct SoftRobotParams {
    /// Number of pneumatic chambers.
    pub num_chambers: usize,
    /// Rest length of each chamber segment (m).
    pub chamber_rest_length: f32,
    /// Maximum elongation per chamber (m).
    pub max_elongation: f32,
    /// Chamber stiffness (N/m).
    pub stiffness: f32,
    /// Damping coefficient (N·s/m).
    pub damping: f32,
    /// Maximum pressure per chamber (Pa).
    pub max_pressure: f32,
    /// Effective cross-sectional area of bellows (m²).
    pub effective_area: f32,
}

impl Default for SoftRobotParams {
    fn default() -> Self {
        Self {
            num_chambers: 3,
            chamber_rest_length: 0.05,
            max_elongation: 0.03,
            stiffness: 200.0,
            damping: 5.0,
            max_pressure: 100_000.0,
            effective_area: 0.001,
        }
    }
}

/// State of a soft robot actuator.
#[derive(Clone, Debug)]
pub struct SoftRobotState {
    /// Pressure in each chamber (Pa).
    pub pressures: Vec<f32>,
    /// Elongation of each chamber (m).
    pub elongations: Vec<f32>,
    /// Velocity of each chamber (m/s).
    pub velocities: Vec<f32>,
    /// Tip position (simplified 2D: x, z).
    pub tip_x: f32,
    pub tip_z: f32,
}

impl SoftRobotState {
    /// Creates a new state for n chambers.
    pub fn new(n: usize) -> Self {
        Self {
            pressures: vec![0.0; n],
            elongations: vec![0.0; n],
            velocities: vec![0.0; n],
            tip_x: 0.0,
            tip_z: 0.0,
        }
    }
}

/// Creates a new soft robot state.
pub fn new_soft_robot_state(params: &SoftRobotParams) -> SoftRobotState {
    SoftRobotState::new(params.num_chambers)
}

/// Sets the pressure for a specific chamber.
pub fn set_chamber_pressure(
    params: &SoftRobotParams,
    state: &mut SoftRobotState,
    chamber: usize,
    pressure: f32,
) {
    if chamber < state.pressures.len() {
        state.pressures[chamber] = pressure.clamp(0.0, params.max_pressure);
    }
}

/// Computes the pneumatic force for a chamber.
pub fn chamber_force(params: &SoftRobotParams, pressure: f32) -> f32 {
    pressure * params.effective_area
}

/// Steps all chambers by dt seconds.
pub fn step_soft_robot(
    params: &SoftRobotParams,
    state: &mut SoftRobotState,
    load_forces: &[f32],
    dt: f32,
) {
    for i in 0..params.num_chambers {
        let pressure = state.pressures[i];
        let load = if i < load_forces.len() {
            load_forces[i]
        } else {
            0.0
        };
        let pneumatic_force = chamber_force(params, pressure);
        let elastic_force = params.stiffness * state.elongations[i];
        let damping_force = params.damping * state.velocities[i];
        /* stub mass = 0.01 kg per chamber */
        let net_force = pneumatic_force - elastic_force - damping_force - load;
        state.velocities[i] += (net_force / 0.01) * dt;
        state.elongations[i] =
            (state.elongations[i] + state.velocities[i] * dt).clamp(0.0, params.max_elongation);
    }
    update_tip_position(params, state);
}

fn update_tip_position(params: &SoftRobotParams, state: &mut SoftRobotState) {
    /* simplified: tip z = rest length * n + sum of elongations */
    let total_length = params.num_chambers as f32 * params.chamber_rest_length
        + state.elongations.iter().sum::<f32>();
    /* bending: difference between chamber 0 and chamber 1 elongation creates curvature */
    let bend = if state.elongations.len() >= 2 {
        state.elongations[0] - state.elongations.last().copied().unwrap_or(0.0)
    } else {
        0.0
    };
    state.tip_z = total_length;
    state.tip_x = bend * 2.0;
}

/// Returns the total elongation of the actuator.
pub fn total_elongation(state: &SoftRobotState) -> f32 {
    state.elongations.iter().sum()
}

/// Returns whether any chamber is at maximum pressure.
pub fn any_at_max_pressure(params: &SoftRobotParams, state: &SoftRobotState) -> bool {
    state
        .pressures
        .iter()
        .any(|&p| (p - params.max_pressure).abs() < 1.0)
}

/// Soft robot actuator stub struct.
pub struct SoftRobotActuator {
    pub params: SoftRobotParams,
    pub state: SoftRobotState,
}

impl SoftRobotActuator {
    /// Creates a new soft robot actuator with default params.
    pub fn new(params: SoftRobotParams) -> Self {
        let state = new_soft_robot_state(&params);
        Self { state, params }
    }

    /// Sets pressure for a chamber and steps simulation.
    pub fn actuate(&mut self, chamber: usize, pressure: f32, load_forces: &[f32], dt: f32) {
        set_chamber_pressure(&self.params, &mut self.state, chamber, pressure);
        step_soft_robot(&self.params, &mut self.state, load_forces, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_robot() -> SoftRobotActuator {
        SoftRobotActuator::new(SoftRobotParams::default())
    }

    #[test]
    fn test_initial_elongations_zero() {
        let r = default_robot();
        assert!(r.state.elongations.iter().all(|&e| e.abs() < 1e-9));
    }

    #[test]
    fn test_set_pressure_clamped_to_max() {
        let mut r = default_robot();
        set_chamber_pressure(&r.params, &mut r.state, 0, 1e9);
        assert!(r.state.pressures[0] <= r.params.max_pressure);
    }

    #[test]
    fn test_chamber_force_positive() {
        let p = SoftRobotParams::default();
        assert!(chamber_force(&p, 50_000.0) > 0.0);
    }

    #[test]
    fn test_pressurized_chamber_elongates() {
        let mut r = default_robot();
        for _ in 0..50 {
            r.actuate(0, 80_000.0, &[], 0.001);
        }
        assert!(r.state.elongations[0] > 0.0);
    }

    #[test]
    fn test_elongation_clamped_to_max() {
        /* use small dt to avoid numerical blow-up; clamp must hold */
        let mut r = default_robot();
        for _ in 0..500 {
            r.actuate(0, 100_000.0, &[], 0.001);
        }
        assert!(r.state.elongations[0] <= r.params.max_elongation + 1e-5);
    }

    #[test]
    fn test_total_elongation_increases_under_pressure() {
        let mut r = default_robot();
        for _ in 0..50 {
            r.actuate(0, 80_000.0, &[], 0.001);
        }
        assert!(total_elongation(&r.state) > 0.0);
    }

    #[test]
    fn test_tip_z_increases_with_elongation() {
        let mut r = default_robot();
        let z0 = r.state.tip_z;
        for _ in 0..50 {
            r.actuate(0, 80_000.0, &[], 0.001);
        }
        assert!(r.state.tip_z >= z0);
    }

    #[test]
    fn test_any_at_max_pressure_detection() {
        let mut r = default_robot();
        set_chamber_pressure(&r.params, &mut r.state, 1, r.params.max_pressure);
        assert!(any_at_max_pressure(&r.params, &r.state));
    }

    #[test]
    fn test_state_num_chambers_correct() {
        let r = default_robot();
        assert_eq!(r.state.pressures.len(), r.params.num_chambers);
    }
}
