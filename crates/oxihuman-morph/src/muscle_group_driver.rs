// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Muscle group driver — grouped muscle activation derived from pose parameters.

/// Named muscle group.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MuscleGroup {
    Pectoralis,
    Deltoid,
    Biceps,
    Triceps,
    Quadriceps,
    Hamstrings,
    Gluteus,
    Gastrocnemius,
    Abdominals,
    Trapezius,
}

/// Activation record for a single muscle group.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MuscleActivation {
    pub group: Option<MuscleGroup>,
    /// Activation level 0..=1.
    pub level: f32,
    /// Contraction ratio affecting mesh bulge 0..=1.
    pub contraction: f32,
}

/// Driver state holding per-group activation levels.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct MuscleGroupDriver {
    activations: Vec<MuscleActivation>,
}

#[allow(dead_code)]
pub fn new_muscle_group_driver() -> MuscleGroupDriver {
    MuscleGroupDriver::default()
}

#[allow(dead_code)]
pub fn mgd_set(driver: &mut MuscleGroupDriver, group: MuscleGroup, level: f32, contraction: f32) {
    let level = level.clamp(0.0, 1.0);
    let contraction = contraction.clamp(0.0, 1.0);
    if let Some(a) = driver
        .activations
        .iter_mut()
        .find(|a| a.group == Some(group))
    {
        a.level = level;
        a.contraction = contraction;
    } else {
        driver.activations.push(MuscleActivation {
            group: Some(group),
            level,
            contraction,
        });
    }
}

#[allow(dead_code)]
pub fn mgd_get(driver: &MuscleGroupDriver, group: MuscleGroup) -> Option<&MuscleActivation> {
    driver.activations.iter().find(|a| a.group == Some(group))
}

#[allow(dead_code)]
pub fn mgd_reset(driver: &mut MuscleGroupDriver) {
    driver.activations.clear();
}

#[allow(dead_code)]
pub fn mgd_active_count(driver: &MuscleGroupDriver) -> usize {
    driver.activations.iter().filter(|a| a.level > 1e-4).count()
}

#[allow(dead_code)]
pub fn mgd_total_activation(driver: &MuscleGroupDriver) -> f32 {
    driver.activations.iter().map(|a| a.level).sum()
}

/// Compute morph weight for a group (bulge factor).
#[allow(dead_code)]
pub fn mgd_bulge_weight(driver: &MuscleGroupDriver, group: MuscleGroup) -> f32 {
    mgd_get(driver, group).map_or(0.0, |a| a.level * a.contraction)
}

/// Blend two drivers at factor t.
#[allow(dead_code)]
pub fn mgd_blend(a: &MuscleGroupDriver, b: &MuscleGroupDriver, t: f32) -> MuscleGroupDriver {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let mut result = MuscleGroupDriver::default();
    for act_a in &a.activations {
        if let Some(group) = act_a.group {
            let level_b = b
                .activations
                .iter()
                .find(|x| x.group == Some(group))
                .map_or(0.0, |x| x.level);
            let cont_b = b
                .activations
                .iter()
                .find(|x| x.group == Some(group))
                .map_or(0.0, |x| x.contraction);
            result.activations.push(MuscleActivation {
                group: Some(group),
                level: act_a.level * inv + level_b * t,
                contraction: act_a.contraction * inv + cont_b * t,
            });
        }
    }
    result
}

#[allow(dead_code)]
pub fn mgd_to_json(driver: &MuscleGroupDriver) -> String {
    format!(
        "{{\"active_count\":{},\"total_activation\":{:.4}}}",
        mgd_active_count(driver),
        mgd_total_activation(driver)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_driver_has_zero_active() {
        assert_eq!(mgd_active_count(&new_muscle_group_driver()), 0);
    }

    #[test]
    fn set_and_get() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Biceps, 0.8, 0.6);
        let a = mgd_get(&d, MuscleGroup::Biceps).unwrap();
        assert!((a.level - 0.8).abs() < 1e-6);
    }

    #[test]
    fn clamps_level() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Triceps, 5.0, 0.5);
        let a = mgd_get(&d, MuscleGroup::Triceps).unwrap();
        assert!((a.level - 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Deltoid, 1.0, 1.0);
        mgd_reset(&mut d);
        assert_eq!(mgd_active_count(&d), 0);
    }

    #[test]
    fn total_activation_sums() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Biceps, 0.5, 0.5);
        mgd_set(&mut d, MuscleGroup::Triceps, 0.5, 0.5);
        assert!((mgd_total_activation(&d) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn bulge_weight_product() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Pectoralis, 0.8, 0.5);
        assert!((mgd_bulge_weight(&d, MuscleGroup::Pectoralis) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn missing_group_bulge_zero() {
        let d = new_muscle_group_driver();
        assert!(mgd_bulge_weight(&d, MuscleGroup::Quadriceps) < 1e-8);
    }

    #[test]
    fn update_existing_entry() {
        let mut d = new_muscle_group_driver();
        mgd_set(&mut d, MuscleGroup::Abdominals, 0.3, 0.3);
        mgd_set(&mut d, MuscleGroup::Abdominals, 0.9, 0.9);
        assert_eq!(mgd_active_count(&d), 1);
        assert!((mgd_get(&d, MuscleGroup::Abdominals).unwrap().level - 0.9).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = new_muscle_group_driver();
        mgd_set(&mut a, MuscleGroup::Gluteus, 1.0, 1.0);
        let b = new_muscle_group_driver();
        let r = mgd_blend(&a, &b, 0.5);
        assert!((mgd_get(&r, MuscleGroup::Gluteus).unwrap().level - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let d = new_muscle_group_driver();
        let j = mgd_to_json(&d);
        assert!(j.contains("active_count") && j.contains("total_activation"));
    }
}
