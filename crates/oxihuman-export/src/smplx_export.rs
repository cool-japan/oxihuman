// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SmplxParams {
    pub betas: Vec<f32>,
    pub expression: Vec<f32>,
    pub pose_body: Vec<f32>,
    pub gender: u8,
}

pub fn new_smplx_params() -> SmplxParams {
    SmplxParams {
        betas: vec![0.0; 10],
        expression: vec![0.0; 10],
        pose_body: vec![0.0; 63],
        gender: 0,
    }
}

pub fn smplx_set_expression(p: &mut SmplxParams, i: usize, v: f32) {
    if i < p.expression.len() {
        p.expression[i] = v;
    }
}

pub fn smplx_to_json(p: &SmplxParams) -> String {
    let betas: Vec<_> = p.betas.iter().map(|v| v.to_string()).collect();
    let expr: Vec<_> = p.expression.iter().map(|v| v.to_string()).collect();
    format!(
        r#"{{"gender":{},"betas":[{}],"expression":[{}]}}"#,
        p.gender,
        betas.join(","),
        expr.join(",")
    )
}

pub fn smplx_num_betas(p: &SmplxParams) -> usize {
    p.betas.len()
}

pub fn smplx_num_expression(p: &SmplxParams) -> usize {
    p.expression.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_smplx_betas() {
        /* 10 betas */
        let p = new_smplx_params();
        assert_eq!(smplx_num_betas(&p), 10);
    }

    #[test]
    fn test_new_smplx_expression() {
        /* 10 expression params */
        let p = new_smplx_params();
        assert_eq!(smplx_num_expression(&p), 10);
    }

    #[test]
    fn test_smplx_set_expression() {
        /* set expression */
        let mut p = new_smplx_params();
        smplx_set_expression(&mut p, 0, 0.8);
        assert!((p.expression[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_smplx_to_json_not_empty() {
        /* json not empty */
        let p = new_smplx_params();
        let j = smplx_to_json(&p);
        assert!(!j.is_empty());
    }

    #[test]
    fn test_smplx_to_json_contains_gender() {
        /* json contains gender */
        let p = new_smplx_params();
        let j = smplx_to_json(&p);
        assert!(j.contains("gender"));
    }
}
