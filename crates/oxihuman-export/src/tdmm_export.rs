// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct TdmmParams {
    pub id_coeffs: Vec<f32>,
    pub exp_coeffs: Vec<f32>,
    pub tex_coeffs: Vec<f32>,
}

pub fn new_tdmm_params(id_dim: usize, exp_dim: usize, tex_dim: usize) -> TdmmParams {
    TdmmParams {
        id_coeffs: vec![0.0; id_dim],
        exp_coeffs: vec![0.0; exp_dim],
        tex_coeffs: vec![0.0; tex_dim],
    }
}

pub fn tdmm_to_json(p: &TdmmParams) -> String {
    let id: Vec<_> = p.id_coeffs.iter().map(|v| v.to_string()).collect();
    let exp: Vec<_> = p.exp_coeffs.iter().map(|v| v.to_string()).collect();
    format!(
        r#"{{"id_dim":{},"exp_dim":{},"tex_dim":{},"id":[{}],"exp":[{}]}}"#,
        p.id_coeffs.len(),
        p.exp_coeffs.len(),
        p.tex_coeffs.len(),
        id.join(","),
        exp.join(",")
    )
}

pub fn tdmm_id_count(p: &TdmmParams) -> usize {
    p.id_coeffs.len()
}

pub fn tdmm_exp_count(p: &TdmmParams) -> usize {
    p.exp_coeffs.len()
}

pub fn tdmm_reconstruct_stub(p: &TdmmParams) -> Vec<f32> {
    vec![0.0; p.id_coeffs.len() + p.exp_coeffs.len() + p.tex_coeffs.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tdmm_params() {
        /* construction */
        let p = new_tdmm_params(80, 64, 80);
        assert_eq!(tdmm_id_count(&p), 80);
    }

    #[test]
    fn test_tdmm_exp_count() {
        /* exp count */
        let p = new_tdmm_params(80, 64, 80);
        assert_eq!(tdmm_exp_count(&p), 64);
    }

    #[test]
    fn test_tdmm_to_json_not_empty() {
        /* json not empty */
        let p = new_tdmm_params(10, 10, 10);
        let j = tdmm_to_json(&p);
        assert!(!j.is_empty());
    }

    #[test]
    fn test_tdmm_reconstruct_stub_length() {
        /* stub length = sum of dims */
        let p = new_tdmm_params(5, 3, 2);
        let r = tdmm_reconstruct_stub(&p);
        assert_eq!(r.len(), 10);
    }

    #[test]
    fn test_tdmm_to_json_contains_id_dim() {
        /* json contains id_dim */
        let p = new_tdmm_params(80, 64, 80);
        let j = tdmm_to_json(&p);
        assert!(j.contains("id_dim"));
    }
}
