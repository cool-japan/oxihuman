// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Chain-and-sprocket drive with slack modeling.

#![allow(dead_code)]

/// A chain-and-sprocket drive.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainDrive {
    /// Number of teeth on driver sprocket.
    pub driver_teeth: u32,
    /// Number of teeth on driven sprocket.
    pub driven_teeth: u32,
    /// Pitch of chain (distance between links, meters).
    pub chain_pitch: f32,
    /// Total chain length in pitches (link count).
    pub chain_links: u32,
    /// Mechanical efficiency.
    pub efficiency: f32,
    /// Slack (meters of free play).
    pub slack: f32,
    /// Driver angular velocity (rad/s).
    pub omega_driver: f32,
    /// Driven angular velocity (rad/s).
    pub omega_driven: f32,
    /// Chain linear speed (m/s).
    pub chain_speed: f32,
    /// Pre-tension force (N).
    pub pretension: f32,
}

/// Create a new chain drive.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn new_chain_drive(
    driver_teeth: u32,
    driven_teeth: u32,
    chain_pitch: f32,
    chain_links: u32,
    efficiency: f32,
    slack: f32,
    pretension: f32,
) -> ChainDrive {
    ChainDrive {
        driver_teeth,
        driven_teeth,
        chain_pitch,
        chain_links,
        efficiency: efficiency.clamp(0.0, 1.0),
        slack: slack.abs(),
        omega_driver: 0.0,
        omega_driven: 0.0,
        chain_speed: 0.0,
        pretension,
    }
}

/// Driver sprocket pitch radius (m).
#[allow(dead_code)]
pub fn chain_driver_radius(cd: &ChainDrive) -> f32 {
    cd.chain_pitch * cd.driver_teeth as f32 / (2.0 * std::f32::consts::PI)
}

/// Driven sprocket pitch radius (m).
#[allow(dead_code)]
pub fn chain_driven_radius(cd: &ChainDrive) -> f32 {
    cd.chain_pitch * cd.driven_teeth as f32 / (2.0 * std::f32::consts::PI)
}

/// Gear ratio: driven / driver = driver_teeth / driven_teeth.
#[allow(dead_code)]
pub fn chain_gear_ratio(cd: &ChainDrive) -> f32 {
    if cd.driven_teeth == 0 {
        return 0.0;
    }
    cd.driver_teeth as f32 / cd.driven_teeth as f32
}

/// Set input (driver) velocity and compute chain/driven velocities.
#[allow(dead_code)]
pub fn chain_set_input(cd: &mut ChainDrive, omega_driver: f32, torque_driver: f32) -> f32 {
    cd.omega_driver = omega_driver;
    let r_driver = chain_driver_radius(cd);
    cd.chain_speed = omega_driver * r_driver;
    let r_driven = chain_driven_radius(cd);
    cd.omega_driven = if r_driven > 1e-10 {
        cd.chain_speed / r_driven
    } else {
        0.0
    };
    let ratio = chain_gear_ratio(cd);
    if ratio.abs() > 1e-10 {
        torque_driver / ratio * cd.efficiency
    } else {
        0.0
    }
}

/// Effective center distance between sprockets (approximate).
#[allow(dead_code)]
pub fn chain_center_distance(cd: &ChainDrive) -> f32 {
    let r1 = chain_driver_radius(cd);
    let r2 = chain_driven_radius(cd);
    let total_chain_len = cd.chain_links as f32 * cd.chain_pitch;
    let approx = (total_chain_len - std::f32::consts::PI * (r1 + r2)) * 0.25;
    approx.max(r1 + r2)
}

/// Tight-side tension (N) given power and chain speed.
#[allow(dead_code)]
pub fn chain_tight_tension(cd: &ChainDrive, power: f32) -> f32 {
    if cd.chain_speed.abs() < 1e-8 {
        return cd.pretension;
    }
    power / cd.chain_speed.abs() + cd.pretension
}

/// Slack side tension (approximately pretension).
#[allow(dead_code)]
pub fn chain_slack_tension(cd: &ChainDrive) -> f32 {
    cd.pretension
}

/// Check if slack is taken up.
#[allow(dead_code)]
pub fn chain_has_slack(cd: &ChainDrive) -> bool {
    cd.slack > 1e-5
}

/// Total chain length in meters.
#[allow(dead_code)]
pub fn chain_length(cd: &ChainDrive) -> f32 {
    cd.chain_links as f32 * cd.chain_pitch
}

/// Reset to zero.
#[allow(dead_code)]
pub fn chain_reset(cd: &mut ChainDrive) {
    cd.omega_driver = 0.0;
    cd.omega_driven = 0.0;
    cd.chain_speed = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cd() -> ChainDrive {
        new_chain_drive(20, 40, 0.0127, 100, 0.98, 0.001, 50.0)
    }

    #[test]
    fn test_gear_ratio() {
        let cd = make_cd();
        assert!((chain_gear_ratio(&cd) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_chain_speed() {
        let mut cd = make_cd();
        chain_set_input(&mut cd, 10.0, 5.0);
        assert!(cd.chain_speed > 0.0);
    }

    #[test]
    fn test_driven_omega_reduced() {
        let mut cd = make_cd();
        chain_set_input(&mut cd, 10.0, 5.0);
        assert!(cd.omega_driven < cd.omega_driver);
    }

    #[test]
    fn test_driver_radius() {
        let cd = make_cd();
        let r = chain_driver_radius(&cd);
        assert!(r > 0.0);
    }

    #[test]
    fn test_chain_length() {
        let cd = make_cd();
        assert!((chain_length(&cd) - 100.0 * 0.0127).abs() < 1e-5);
    }

    #[test]
    fn test_has_slack() {
        let cd = make_cd();
        assert!(chain_has_slack(&cd));
    }

    #[test]
    fn test_tight_tension_at_rest() {
        let cd = make_cd();
        let t = chain_tight_tension(&cd, 0.0);
        assert!((t - cd.pretension).abs() < 1e-4);
    }

    #[test]
    fn test_center_distance_positive() {
        let cd = make_cd();
        assert!(chain_center_distance(&cd) > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut cd = make_cd();
        chain_set_input(&mut cd, 10.0, 5.0);
        chain_reset(&mut cd);
        assert_eq!(cd.omega_driver, 0.0);
        assert_eq!(cd.chain_speed, 0.0);
    }

    #[test]
    fn test_efficiency_clamped() {
        let cd = new_chain_drive(20, 40, 0.0127, 100, 1.5, 0.001, 50.0);
        assert!(cd.efficiency <= 1.0);
    }
}
