// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Balance/stabilization controller stub.

/// PD gains for balance control.
#[derive(Debug, Clone)]
pub struct BalanceGains {
    pub kp_x: f32,
    pub kd_x: f32,
    pub kp_y: f32,
    pub kd_y: f32,
}

impl Default for BalanceGains {
    fn default() -> Self {
        Self {
            kp_x: 100.0,
            kd_x: 10.0,
            kp_y: 100.0,
            kd_y: 10.0,
        }
    }
}

/// Balance controller state.
#[derive(Debug, Clone, Default)]
pub struct BalanceController {
    pub gains: BalanceGains,
    pub target_x: f32,
    pub target_y: f32,
}

impl BalanceController {
    pub fn new(gains: BalanceGains) -> Self {
        Self {
            gains,
            target_x: 0.0,
            target_y: 0.0,
        }
    }

    pub fn set_target(&mut self, x: f32, y: f32) {
        self.target_x = x;
        self.target_y = y;
    }
}

/// Compute balance control torques using PD control.
pub fn compute_balance_torques(
    ctrl: &BalanceController,
    com_x: f32,
    com_y: f32,
    vel_x: f32,
    vel_y: f32,
) -> [f32; 2] {
    /* PD: τ = -kp * e - kd * ė */
    let ex = com_x - ctrl.target_x;
    let ey = com_y - ctrl.target_y;
    let tx = -ctrl.gains.kp_x * ex - ctrl.gains.kd_x * vel_x;
    let ty = -ctrl.gains.kp_y * ey - ctrl.gains.kd_y * vel_y;
    [tx, ty]
}

/// Clamp torques to a maximum magnitude.
pub fn clamp_torques(torques: [f32; 2], max_torque: f32) -> [f32; 2] {
    [
        torques[0].clamp(-max_torque, max_torque),
        torques[1].clamp(-max_torque, max_torque),
    ]
}

/// Return whether the balance error is within tolerance.
pub fn balance_error_within_tolerance(
    com_x: f32,
    com_y: f32,
    target_x: f32,
    target_y: f32,
    tol: f32,
) -> bool {
    let dx = com_x - target_x;
    let dy = com_y - target_y;
    (dx * dx + dy * dy).sqrt() < tol
}

/// Compute the balance error magnitude.
pub fn balance_error(com_x: f32, com_y: f32, target_x: f32, target_y: f32) -> f32 {
    let dx = com_x - target_x;
    let dy = com_y - target_y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gains() {
        /* default Kp is 100 */
        let g = BalanceGains::default();
        assert_eq!(g.kp_x, 100.0);
    }

    #[test]
    fn test_zero_error_zero_torque() {
        /* at target, torques are zero (with zero velocity) */
        let ctrl = BalanceController::new(BalanceGains::default());
        let t = compute_balance_torques(&ctrl, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(t, [0.0, 0.0]);
    }

    #[test]
    fn test_positive_error_negative_torque() {
        /* positive error produces restoring torque */
        let ctrl = BalanceController::new(BalanceGains::default());
        let t = compute_balance_torques(&ctrl, 0.1, 0.0, 0.0, 0.0);
        assert!(t[0] < 0.0);
    }

    #[test]
    fn test_clamp_torques() {
        /* torques are clamped */
        let t = clamp_torques([200.0, -200.0], 50.0);
        assert_eq!(t, [50.0, -50.0]);
    }

    #[test]
    fn test_balance_error_zero_at_target() {
        /* error is zero at target */
        assert!(balance_error(0.0, 0.0, 0.0, 0.0) < 1e-6);
    }

    #[test]
    fn test_balance_error_known() {
        /* 3-4-5 triangle */
        assert!((balance_error(3.0, 4.0, 0.0, 0.0) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_within_tolerance_true() {
        /* small error within tolerance */
        assert!(balance_error_within_tolerance(0.01, 0.0, 0.0, 0.0, 0.1));
    }

    #[test]
    fn test_within_tolerance_false() {
        /* large error outside tolerance */
        assert!(!balance_error_within_tolerance(1.0, 0.0, 0.0, 0.0, 0.1));
    }

    #[test]
    fn test_set_target() {
        /* target is updated */
        let mut ctrl = BalanceController::new(BalanceGains::default());
        ctrl.set_target(0.5, 0.3);
        assert!((ctrl.target_x - 0.5).abs() < 1e-6);
    }
}
