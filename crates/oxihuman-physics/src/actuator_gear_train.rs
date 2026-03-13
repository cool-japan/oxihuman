// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gear train reduction model stub — multi-stage gear reduction.

/// Parameters for a single gear stage.
#[derive(Clone, Debug)]
pub struct GearStageParams {
    /// Number of teeth on the driving gear.
    pub driver_teeth: u32,
    /// Number of teeth on the driven gear.
    pub driven_teeth: u32,
    /// Efficiency of this stage (0–1).
    pub efficiency: f32,
}

impl GearStageParams {
    /// Returns the gear ratio (driven/driver).
    pub fn ratio(&self) -> f32 {
        self.driven_teeth as f32 / self.driver_teeth as f32
    }
}

/// A multi-stage gear train.
#[derive(Clone, Debug)]
pub struct GearTrain {
    pub stages: Vec<GearStageParams>,
}

/// State of the gear train.
#[derive(Clone, Debug, Default)]
pub struct GearTrainState {
    /// Input (motor) angular velocity (rad/s).
    pub input_omega: f32,
    /// Output angular velocity (rad/s).
    pub output_omega: f32,
    /// Input torque (N·m).
    pub input_torque: f32,
    /// Output torque (N·m).
    pub output_torque: f32,
}

/// Creates a simple single-stage gear train.
pub fn single_stage_gear(driver: u32, driven: u32, efficiency: f32) -> GearTrain {
    GearTrain {
        stages: vec![GearStageParams {
            driver_teeth: driver,
            driven_teeth: driven,
            efficiency,
        }],
    }
}

/// Computes the total gear ratio of the train (product of stage ratios).
pub fn total_ratio(train: &GearTrain) -> f32 {
    train.stages.iter().fold(1.0, |acc, s| acc * s.ratio())
}

/// Computes the overall efficiency (product of stage efficiencies).
pub fn total_efficiency(train: &GearTrain) -> f32 {
    train.stages.iter().fold(1.0, |acc, s| acc * s.efficiency)
}

/// Updates the gear train state from input speed and torque.
pub fn update_gear_train(train: &GearTrain, state: &mut GearTrainState) {
    let ratio = total_ratio(train);
    let eff = total_efficiency(train);
    state.output_omega = state.input_omega / ratio;
    state.output_torque = state.input_torque * ratio * eff;
}

/// Returns the gear ratio of a single stage by index.
pub fn stage_ratio(train: &GearTrain, index: usize) -> Option<f32> {
    train.stages.get(index).map(|s| s.ratio())
}

/// Adds a stage to the gear train and returns a new instance.
pub fn add_stage(train: &GearTrain, stage: GearStageParams) -> GearTrain {
    let mut new = train.clone();
    new.stages.push(stage);
    new
}

/// Returns the reflected inertia seen at the input given output inertia.
pub fn reflected_inertia(train: &GearTrain, output_inertia: f32) -> f32 {
    let r = total_ratio(train);
    output_inertia / (r * r)
}

/// Gear train stub struct with state.
pub struct GearTrainActuator {
    pub train: GearTrain,
    pub state: GearTrainState,
}

impl GearTrainActuator {
    /// Creates a new gear train actuator.
    pub fn new(train: GearTrain) -> Self {
        Self {
            train,
            state: GearTrainState::default(),
        }
    }

    /// Applies input and updates output state.
    pub fn apply_input(&mut self, input_omega: f32, input_torque: f32) {
        self.state.input_omega = input_omega;
        self.state.input_torque = input_torque;
        update_gear_train(&self.train, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_2x_gear() -> GearTrain {
        single_stage_gear(10, 20, 0.95)
    }

    #[test]
    fn test_single_stage_ratio() {
        let g = make_2x_gear();
        let r = stage_ratio(&g, 0).expect("should succeed");
        assert!((r - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_ratio_single_stage() {
        let g = make_2x_gear();
        assert!((total_ratio(&g) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_efficiency_single_stage() {
        let g = make_2x_gear();
        assert!((total_efficiency(&g) - 0.95).abs() < 1e-5);
    }

    #[test]
    fn test_output_omega_reduced() {
        let g = make_2x_gear();
        let mut state = GearTrainState {
            input_omega: 100.0,
            input_torque: 1.0,
            ..Default::default()
        };
        update_gear_train(&g, &mut state);
        assert!((state.output_omega - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_output_torque_increased() {
        let g = make_2x_gear();
        let mut state = GearTrainState {
            input_omega: 100.0,
            input_torque: 1.0,
            ..Default::default()
        };
        update_gear_train(&g, &mut state);
        /* torque multiplied by ratio * efficiency = 2 * 0.95 = 1.9 */
        assert!((state.output_torque - 1.9).abs() < 1e-4);
    }

    #[test]
    fn test_add_stage_increases_total_ratio() {
        let g = make_2x_gear();
        let stage2 = GearStageParams {
            driver_teeth: 10,
            driven_teeth: 10,
            efficiency: 1.0,
        };
        let g2 = add_stage(&g, stage2);
        assert!((total_ratio(&g2) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_reflected_inertia_decreases_with_ratio() {
        let g = make_2x_gear();
        let ri = reflected_inertia(&g, 1.0);
        assert!(ri < 1.0);
    }

    #[test]
    fn test_stage_ratio_out_of_bounds_returns_none() {
        let g = make_2x_gear();
        assert!(stage_ratio(&g, 10).is_none());
    }

    #[test]
    fn test_gear_train_actuator_apply_input() {
        let mut gta = GearTrainActuator::new(make_2x_gear());
        gta.apply_input(200.0, 0.5);
        assert!((gta.state.output_omega - 100.0).abs() < 1e-4);
    }
}
