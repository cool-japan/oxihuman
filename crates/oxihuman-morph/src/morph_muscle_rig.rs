// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Muscle rig: muscles driving morph targets.

#[allow(dead_code)]
pub struct Muscle {
    pub name: String,
    pub contraction: f32,
    pub morph_idx: usize,
    pub gain: f32,
}

#[allow(dead_code)]
pub struct MuscleRig {
    pub muscles: Vec<Muscle>,
}

#[allow(dead_code)]
pub fn new_muscle_rig() -> MuscleRig {
    MuscleRig { muscles: Vec::new() }
}

#[allow(dead_code)]
pub fn mr_add_muscle(rig: &mut MuscleRig, name: &str, morph_idx: usize, gain: f32) -> usize {
    let idx = rig.muscles.len();
    rig.muscles.push(Muscle { name: name.to_string(), contraction: 0.0, morph_idx, gain });
    idx
}

#[allow(dead_code)]
pub fn mr_contract(rig: &mut MuscleRig, idx: usize, amount: f32) {
    if idx < rig.muscles.len() {
        rig.muscles[idx].contraction = amount.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn mr_compute_weights(rig: &MuscleRig, n_morphs: usize) -> Vec<f32> {
    let mut weights = vec![0.0f32; n_morphs];
    for m in &rig.muscles {
        if m.morph_idx < n_morphs {
            weights[m.morph_idx] += m.gain * m.contraction;
        }
    }
    weights
}

#[allow(dead_code)]
pub fn mr_muscle_count(rig: &MuscleRig) -> usize {
    rig.muscles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_muscle() {
        let mut rig = new_muscle_rig();
        let idx = mr_add_muscle(&mut rig, "bicep", 0, 1.0);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_muscle_count() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "a", 0, 1.0);
        mr_add_muscle(&mut rig, "b", 1, 1.0);
        assert_eq!(mr_muscle_count(&rig), 2);
    }

    #[test]
    fn test_contract_clamped() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "m", 0, 1.0);
        mr_contract(&mut rig, 0, 2.0);
        assert!((rig.muscles[0].contraction - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_contract_negative_clamped() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "m", 0, 1.0);
        mr_contract(&mut rig, 0, -1.0);
        assert!(rig.muscles[0].contraction.abs() < 1e-5);
    }

    #[test]
    fn test_compute_weights() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "m", 0, 0.8);
        mr_contract(&mut rig, 0, 1.0);
        let w = mr_compute_weights(&rig, 2);
        assert!((w[0] - 0.8).abs() < 1e-5);
        assert!(w[1].abs() < 1e-5);
    }

    #[test]
    fn test_compute_weights_zero_contraction() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "m", 0, 1.0);
        let w = mr_compute_weights(&rig, 1);
        assert!(w[0].abs() < 1e-5);
    }

    #[test]
    fn test_compute_weights_accumulate() {
        let mut rig = new_muscle_rig();
        mr_add_muscle(&mut rig, "m1", 0, 0.5);
        mr_add_muscle(&mut rig, "m2", 0, 0.5);
        mr_contract(&mut rig, 0, 1.0);
        mr_contract(&mut rig, 1, 1.0);
        let w = mr_compute_weights(&rig, 1);
        assert!((w[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_contract_out_of_bounds_safe() {
        let mut rig = new_muscle_rig();
        mr_contract(&mut rig, 99, 1.0); /* should not panic */
    }
}
