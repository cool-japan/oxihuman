// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SmplParams {
    pub betas: Vec<f32>,
    pub pose_angles: Vec<f32>,
    pub gender: u8,
}

pub fn new_smpl_params(num_betas: usize) -> SmplParams {
    SmplParams {
        betas: vec![0.0; num_betas],
        pose_angles: vec![0.0; 72],
        gender: 0,
    }
}

pub fn smpl_set_beta(p: &mut SmplParams, i: usize, v: f32) {
    if i < p.betas.len() {
        p.betas[i] = v;
    }
}

pub fn smpl_set_pose(p: &mut SmplParams, i: usize, v: f32) {
    if i < p.pose_angles.len() {
        p.pose_angles[i] = v;
    }
}

pub fn smpl_to_json(p: &SmplParams) -> String {
    let betas: Vec<_> = p.betas.iter().map(|v| v.to_string()).collect();
    format!(r#"{{"gender":{},"betas":[{}]}}"#, p.gender, betas.join(","))
}

pub fn smpl_gender_name(g: u8) -> &'static str {
    match g {
        1 => "male",
        2 => "female",
        _ => "neutral",
    }
}

pub fn smpl_param_count(p: &SmplParams) -> usize {
    p.betas.len() + p.pose_angles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_smpl_params() {
        /* 10 betas */
        let p = new_smpl_params(10);
        assert_eq!(p.betas.len(), 10);
    }

    #[test]
    fn test_smpl_set_beta() {
        /* set beta */
        let mut p = new_smpl_params(10);
        smpl_set_beta(&mut p, 0, 1.5);
        assert!((p.betas[0] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_smpl_gender_neutral() {
        /* gender 0 = neutral */
        assert_eq!(smpl_gender_name(0), "neutral");
    }

    #[test]
    fn test_smpl_gender_male() {
        /* gender 1 = male */
        assert_eq!(smpl_gender_name(1), "male");
    }

    #[test]
    fn test_smpl_param_count() {
        /* betas + pose */
        let p = new_smpl_params(10);
        assert_eq!(smpl_param_count(&p), 82);
    }
}
