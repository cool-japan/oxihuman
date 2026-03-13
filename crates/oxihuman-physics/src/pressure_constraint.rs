// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pressure/inflation constraint for cloth/soft bodies.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PressureConfig {
    pub target_pressure: f32,
    pub compliance: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PressureConstraint {
    pub surface_area: f32,
    pub current_volume: f32,
    pub lambda: f32,
    pub config: PressureConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PressureResult {
    pub correction_scale: f32,
    pub volume_error: f32,
    pub satisfied: bool,
}

#[allow(dead_code)]
pub fn default_pressure_config() -> PressureConfig {
    PressureConfig { target_pressure: 1.0, compliance: 0.001 }
}

#[allow(dead_code)]
pub fn new_pressure_constraint(
    surface_area: f32,
    current_volume: f32,
    config: PressureConfig,
) -> PressureConstraint {
    PressureConstraint { surface_area, current_volume, lambda: 0.0, config }
}

#[allow(dead_code)]
pub fn pressure_solve(pc: &mut PressureConstraint, dt: f32) -> PressureResult {
    // Constraint: C = current_volume - target_volume
    // target_volume derived from pressure: P = force / area (simplified model)
    let target_volume = pc.config.target_pressure * pc.surface_area;
    let error = pc.current_volume - target_volume;
    let compliance = pc.config.compliance / (dt * dt);
    let denom = 1.0 + compliance;
    let correction_scale = if denom.abs() > 1e-12 {
        (-error - compliance * pc.lambda) / denom
    } else {
        0.0
    };
    pc.lambda += correction_scale;
    pc.current_volume += correction_scale;
    let satisfied = error.abs() < 1e-4;
    PressureResult { correction_scale, volume_error: error, satisfied }
}

#[allow(dead_code)]
pub fn pressure_error(pc: &PressureConstraint) -> f32 {
    let target_volume = pc.config.target_pressure * pc.surface_area;
    (pc.current_volume - target_volume).abs()
}

#[allow(dead_code)]
pub fn pressure_reset(pc: &mut PressureConstraint) {
    pc.lambda = 0.0;
}

#[allow(dead_code)]
pub fn pressure_set_target(pc: &mut PressureConstraint, target: f32) {
    pc.config.target_pressure = target;
}

#[allow(dead_code)]
pub fn pressure_is_satisfied(pc: &PressureConstraint, tol: f32) -> bool {
    pressure_error(pc) < tol
}

#[allow(dead_code)]
pub fn pressure_stiffness(pc: &PressureConstraint) -> f32 {
    if pc.config.compliance > 1e-12 {
        1.0 / pc.config.compliance
    } else {
        f32::INFINITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_constraint() {
        let cfg = default_pressure_config();
        let pc = new_pressure_constraint(1.0, 1.0, cfg);
        assert!((pc.surface_area - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_error_at_equilibrium() {
        let cfg = PressureConfig { target_pressure: 1.0, compliance: 0.001 };
        // target_volume = 1.0 * 1.0 = 1.0; current_volume = 1.0 => error = 0
        let pc = new_pressure_constraint(1.0, 1.0, cfg);
        assert!(pressure_error(&pc) < 1e-5);
    }

    #[test]
    fn test_error_when_deflated() {
        let cfg = PressureConfig { target_pressure: 1.0, compliance: 0.001 };
        let pc = new_pressure_constraint(1.0, 0.5, cfg);
        assert!(pressure_error(&pc) > 0.0);
    }

    #[test]
    fn test_solve_returns_result() {
        let cfg = default_pressure_config();
        let mut pc = new_pressure_constraint(1.0, 0.5, cfg);
        let res = pressure_solve(&mut pc, 0.016);
        let _ = res.correction_scale;
    }

    #[test]
    fn test_reset_lambda() {
        let cfg = default_pressure_config();
        let mut pc = new_pressure_constraint(1.0, 1.0, cfg);
        pc.lambda = 5.0;
        pressure_reset(&mut pc);
        assert!(pc.lambda.abs() < 1e-9);
    }

    #[test]
    fn test_set_target() {
        let cfg = default_pressure_config();
        let mut pc = new_pressure_constraint(1.0, 1.0, cfg);
        pressure_set_target(&mut pc, 2.0);
        assert!((pc.config.target_pressure - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_satisfied_at_equilibrium() {
        let cfg = PressureConfig { target_pressure: 1.0, compliance: 0.001 };
        let pc = new_pressure_constraint(1.0, 1.0, cfg);
        assert!(pressure_is_satisfied(&pc, 1e-4));
    }

    #[test]
    fn test_stiffness_finite() {
        let cfg = PressureConfig { target_pressure: 1.0, compliance: 0.01 };
        let pc = new_pressure_constraint(1.0, 1.0, cfg);
        assert!(pressure_stiffness(&pc).is_finite());
    }

    #[test]
    fn test_stiffness_infinite_for_rigid() {
        let cfg = PressureConfig { target_pressure: 1.0, compliance: 0.0 };
        let pc = new_pressure_constraint(1.0, 1.0, cfg);
        assert!(pressure_stiffness(&pc).is_infinite());
    }
}
