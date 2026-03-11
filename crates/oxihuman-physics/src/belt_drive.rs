// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Belt drive with slip and pre-tension modeling.

#![allow(dead_code)]

/// A belt drive (V-belt or flat belt).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BeltDrive {
    /// Radius of driver pulley (m).
    pub r_driver: f32,
    /// Radius of driven pulley (m).
    pub r_driven: f32,
    /// Pre-tension of the belt (N).
    pub pretension: f32,
    /// Coefficient of friction (belt vs pulley).
    pub mu: f32,
    /// Wrap angle of belt on driver pulley (radians).
    pub wrap_angle: f32,
    /// Slip coefficient (0 = no slip, 1 = full slip).
    pub slip: f32,
    /// Belt speed (m/s).
    pub belt_speed: f32,
    /// Driver angular velocity (rad/s).
    pub omega_driver: f32,
    /// Driven angular velocity (rad/s).
    pub omega_driven: f32,
    /// Mechanical efficiency.
    pub efficiency: f32,
}

/// Create a new belt drive.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn new_belt_drive(
    r_driver: f32,
    r_driven: f32,
    pretension: f32,
    mu: f32,
    wrap_angle: f32,
    efficiency: f32,
) -> BeltDrive {
    BeltDrive {
        r_driver: r_driver.abs().max(1e-6),
        r_driven: r_driven.abs().max(1e-6),
        pretension: pretension.abs(),
        mu,
        wrap_angle: wrap_angle.abs(),
        slip: 0.0,
        belt_speed: 0.0,
        omega_driver: 0.0,
        omega_driven: 0.0,
        efficiency: efficiency.clamp(0.0, 1.0),
    }
}

/// Gear ratio: driven/driver = r_driver/r_driven.
#[allow(dead_code)]
pub fn belt_gear_ratio(bd: &BeltDrive) -> f32 {
    bd.r_driver / bd.r_driven
}

/// Maximum transmissible force using Euler-Eytelwein: F_tight/F_slack = e^(mu*alpha).
#[allow(dead_code)]
pub fn belt_max_force_ratio(bd: &BeltDrive) -> f32 {
    (bd.mu * bd.wrap_angle).exp()
}

/// Maximum transmissible tension (tight side) given pretension.
#[allow(dead_code)]
pub fn belt_tight_tension(bd: &BeltDrive) -> f32 {
    let ratio = belt_max_force_ratio(bd);
    bd.pretension * ratio / (ratio - 1.0)
}

/// Slack side tension.
#[allow(dead_code)]
pub fn belt_slack_tension(bd: &BeltDrive) -> f32 {
    let ratio = belt_max_force_ratio(bd);
    bd.pretension / (ratio - 1.0)
}

/// Maximum transmissible power given belt speed.
#[allow(dead_code)]
pub fn belt_max_power(bd: &BeltDrive) -> f32 {
    let f_max = belt_tight_tension(bd) - belt_slack_tension(bd);
    f_max * bd.belt_speed.abs()
}

/// Set input and compute output.
/// Returns output torque.
#[allow(dead_code)]
pub fn belt_set_input(bd: &mut BeltDrive, omega_driver: f32, torque_in: f32) -> f32 {
    bd.omega_driver = omega_driver;
    bd.belt_speed = omega_driver * bd.r_driver * (1.0 - bd.slip);
    bd.omega_driven = bd.belt_speed / bd.r_driven;
    let ratio = belt_gear_ratio(bd);
    if ratio.abs() > 1e-10 {
        torque_in / ratio * bd.efficiency
    } else {
        0.0
    }
}

/// Apply slip: reduce effective belt speed by slip fraction.
#[allow(dead_code)]
pub fn belt_set_slip(bd: &mut BeltDrive, slip: f32) {
    bd.slip = slip.clamp(0.0, 1.0);
}

/// Power loss due to slip and friction.
#[allow(dead_code)]
pub fn belt_power_loss(bd: &BeltDrive) -> f32 {
    let p_in = bd.omega_driver * bd.r_driver * (belt_tight_tension(bd) - belt_slack_tension(bd));
    let p_out = bd.omega_driven * bd.r_driven * (belt_tight_tension(bd) - belt_slack_tension(bd));
    (p_in - p_out).max(0.0)
}

/// Reset belt drive state.
#[allow(dead_code)]
pub fn belt_reset(bd: &mut BeltDrive) {
    bd.omega_driver = 0.0;
    bd.omega_driven = 0.0;
    bd.belt_speed = 0.0;
    bd.slip = 0.0;
}

/// Center distance between pulleys.
#[allow(dead_code)]
pub fn belt_center_distance(bd: &BeltDrive, belt_length: f32) -> f32 {
    let sum_r = bd.r_driver + bd.r_driven;
    let diff_r = bd.r_driver - bd.r_driven;
    let arg = belt_length - std::f32::consts::PI * sum_r;
    if arg < diff_r.abs() * 2.0 {
        sum_r
    } else {
        (arg * arg - 4.0 * diff_r * diff_r * 0.25).sqrt() * 0.125 / 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bd() -> BeltDrive {
        new_belt_drive(0.1, 0.2, 100.0, 0.3, std::f32::consts::PI, 0.97)
    }

    #[test]
    fn test_gear_ratio() {
        let bd = make_bd();
        assert!((belt_gear_ratio(&bd) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_max_force_ratio_positive() {
        let bd = make_bd();
        assert!(belt_max_force_ratio(&bd) > 1.0);
    }

    #[test]
    fn test_tight_greater_than_slack() {
        let bd = make_bd();
        assert!(belt_tight_tension(&bd) > belt_slack_tension(&bd));
    }

    #[test]
    fn test_belt_speed() {
        let mut bd = make_bd();
        belt_set_input(&mut bd, 10.0, 5.0);
        assert!(bd.belt_speed > 0.0);
    }

    #[test]
    fn test_driven_omega_reduced() {
        let mut bd = make_bd();
        belt_set_input(&mut bd, 10.0, 5.0);
        assert!(bd.omega_driven < bd.omega_driver);
    }

    #[test]
    fn test_slip_reduces_belt_speed() {
        let mut bd = make_bd();
        belt_set_slip(&mut bd, 0.1);
        belt_set_input(&mut bd, 10.0, 5.0);
        let speed_with_slip = bd.belt_speed;
        bd.slip = 0.0;
        belt_set_input(&mut bd, 10.0, 5.0);
        let speed_no_slip = bd.belt_speed;
        assert!(speed_with_slip < speed_no_slip);
    }

    #[test]
    fn test_reset() {
        let mut bd = make_bd();
        belt_set_input(&mut bd, 10.0, 5.0);
        belt_reset(&mut bd);
        assert_eq!(bd.omega_driver, 0.0);
        assert_eq!(bd.belt_speed, 0.0);
    }

    #[test]
    fn test_efficiency_clamped() {
        let bd = new_belt_drive(0.1, 0.2, 100.0, 0.3, std::f32::consts::PI, 1.5);
        assert!(bd.efficiency <= 1.0);
    }

    #[test]
    fn test_max_power_at_rest_is_zero() {
        let bd = make_bd();
        assert_eq!(belt_max_power(&bd), 0.0);
    }
}
