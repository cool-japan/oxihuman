// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Elastic joint: spring-damper joint connecting two rigid bodies.

use std::f32::consts::FRAC_PI_2;

/// Elastic joint parameters.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ElasticJoint {
    pub stiffness: f32,
    pub damping: f32,
    pub rest_length: f32,
    pub max_extension: f32,
    pub min_extension: f32,
}

/// Create an elastic joint.
#[allow(dead_code)]
pub fn new_elastic_joint(stiffness: f32, damping: f32, rest_length: f32) -> ElasticJoint {
    ElasticJoint {
        stiffness,
        damping,
        rest_length,
        max_extension: rest_length * 2.0,
        min_extension: 0.0,
    }
}

/// Spring force magnitude along the joint axis.
#[allow(dead_code)]
pub fn elastic_spring_force(joint: &ElasticJoint, current_length: f32, velocity: f32) -> f32 {
    let extension = (current_length - joint.rest_length).clamp(
        joint.min_extension - joint.rest_length,
        joint.max_extension - joint.rest_length,
    );
    joint.stiffness * extension + joint.damping * velocity
}

/// Potential energy stored in the joint.
#[allow(dead_code)]
pub fn elastic_potential_energy(joint: &ElasticJoint, current_length: f32) -> f32 {
    let ext = current_length - joint.rest_length;
    0.5 * joint.stiffness * ext * ext
}

/// Whether joint is within its limits.
#[allow(dead_code)]
pub fn elastic_within_limits(joint: &ElasticJoint, current_length: f32) -> bool {
    (joint.min_extension..=joint.max_extension).contains(&current_length)
}

/// Damping force (opposing velocity).
#[allow(dead_code)]
pub fn elastic_damping_force(joint: &ElasticJoint, velocity: f32) -> f32 {
    -joint.damping * velocity
}

/// Critical damping coefficient.
#[allow(dead_code)]
pub fn elastic_critical_damping(joint: &ElasticJoint, mass: f32) -> f32 {
    2.0 * (joint.stiffness * mass).sqrt()
}

/// Natural frequency of the joint.
#[allow(dead_code)]
pub fn elastic_natural_frequency(joint: &ElasticJoint, mass: f32) -> f32 {
    (joint.stiffness / mass.max(1e-12)).sqrt()
}

/// Update rest length.
#[allow(dead_code)]
pub fn elastic_set_rest(joint: &mut ElasticJoint, rest: f32) {
    joint.rest_length = rest;
}

/// Dummy use of constant.
#[allow(dead_code)]
pub fn joint_quarter_pi() -> f32 {
    FRAC_PI_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_force_at_rest() {
        let j = new_elastic_joint(100.0, 5.0, 1.0);
        let f = elastic_spring_force(&j, 1.0, 0.0);
        assert!(f.abs() < 1e-5);
    }

    #[test]
    fn test_spring_force_stretched() {
        let j = new_elastic_joint(100.0, 0.0, 1.0);
        let f = elastic_spring_force(&j, 1.5, 0.0);
        assert!((f - 50.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_spring_force_compressed() {
        let j = new_elastic_joint(100.0, 0.0, 1.0);
        let f = elastic_spring_force(&j, 0.5, 0.0);
        assert!(f < 0.0);
    }

    #[test]
    fn test_potential_energy_zero_at_rest() {
        let j = new_elastic_joint(100.0, 5.0, 2.0);
        assert!((elastic_potential_energy(&j, 2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_potential_energy_positive() {
        let j = new_elastic_joint(100.0, 5.0, 1.0);
        assert!(elastic_potential_energy(&j, 2.0) > 0.0);
    }

    #[test]
    fn test_within_limits() {
        let j = new_elastic_joint(100.0, 5.0, 1.0);
        assert!(elastic_within_limits(&j, 1.5));
        assert!(!elastic_within_limits(&j, 3.0));
    }

    #[test]
    fn test_natural_frequency() {
        let j = new_elastic_joint(100.0, 0.0, 1.0);
        let omega = elastic_natural_frequency(&j, 1.0);
        assert!((omega - 10.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_critical_damping() {
        let j = new_elastic_joint(100.0, 0.0, 1.0);
        let cd = elastic_critical_damping(&j, 1.0);
        assert!((cd - 20.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_set_rest() {
        let mut j = new_elastic_joint(100.0, 5.0, 1.0);
        elastic_set_rest(&mut j, 2.0);
        assert!((j.rest_length - 2.0_f32).abs() < 1e-5);
    }
}
