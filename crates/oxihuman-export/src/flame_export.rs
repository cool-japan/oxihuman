// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct FlameParams {
    pub shape: Vec<f32>,
    pub expression: Vec<f32>,
    pub pose: Vec<f32>,
}

pub fn new_flame_params() -> FlameParams {
    FlameParams {
        shape: vec![0.0; 100],
        expression: vec![0.0; 50],
        pose: vec![0.0; 15],
    }
}

pub fn flame_set_shape(p: &mut FlameParams, i: usize, v: f32) {
    if i < p.shape.len() {
        p.shape[i] = v;
    }
}

pub fn flame_to_json(p: &FlameParams) -> String {
    let shape: Vec<_> = p.shape.iter().map(|v| v.to_string()).collect();
    let expr: Vec<_> = p.expression.iter().map(|v| v.to_string()).collect();
    format!(
        r#"{{"shape_dim":{},"expression_dim":{},"shape":[{}],"expression":[{}]}}"#,
        p.shape.len(),
        p.expression.len(),
        shape.join(","),
        expr.join(",")
    )
}

pub fn flame_shape_count(p: &FlameParams) -> usize {
    p.shape.len()
}

pub fn flame_expression_count(p: &FlameParams) -> usize {
    p.expression.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flame_shape_dim() {
        /* 100 shape params */
        let p = new_flame_params();
        assert_eq!(flame_shape_count(&p), 100);
    }

    #[test]
    fn test_new_flame_expression_dim() {
        /* 50 expression params */
        let p = new_flame_params();
        assert_eq!(flame_expression_count(&p), 50);
    }

    #[test]
    fn test_flame_set_shape() {
        /* set shape param */
        let mut p = new_flame_params();
        flame_set_shape(&mut p, 0, 1.2);
        assert!((p.shape[0] - 1.2).abs() < 1e-6);
    }

    #[test]
    fn test_flame_to_json_not_empty() {
        /* json not empty */
        let p = new_flame_params();
        let j = flame_to_json(&p);
        assert!(!j.is_empty());
    }

    #[test]
    fn test_flame_to_json_contains_shape_dim() {
        /* json contains shape_dim */
        let p = new_flame_params();
        let j = flame_to_json(&p);
        assert!(j.contains("shape_dim"));
    }
}
