// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Driver: maps input parameter to morph weight.

#[allow(dead_code)]
pub struct DriverCurvePoint {
    pub param: f32,
    pub weight: f32,
}

#[allow(dead_code)]
pub struct WeightDriver {
    pub input_param: f32,
    pub curve: Vec<DriverCurvePoint>,
    pub morph_idx: usize,
}

#[allow(dead_code)]
pub fn new_weight_driver(morph_idx: usize) -> WeightDriver {
    WeightDriver { input_param: 0.0, curve: Vec::new(), morph_idx }
}

#[allow(dead_code)]
pub fn wd2_add_curve_point(d: &mut WeightDriver, param: f32, weight: f32) {
    let pos = d.curve.partition_point(|p| p.param <= param);
    d.curve.insert(pos, DriverCurvePoint { param, weight });
}

#[allow(dead_code)]
pub fn wd2_evaluate(d: &WeightDriver, param: f32) -> f32 {
    if d.curve.is_empty() {
        return 0.0;
    }
    if param <= d.curve[0].param {
        return d.curve[0].weight;
    }
    let last = &d.curve[d.curve.len() - 1];
    if param >= last.param {
        return last.weight;
    }
    let idx = d.curve.partition_point(|p| p.param <= param);
    let a = &d.curve[idx - 1];
    let b = &d.curve[idx];
    let span = b.param - a.param;
    let t = if span > 1e-7 { (param - a.param) / span } else { 0.0 };
    a.weight + (b.weight - a.weight) * t
}

#[allow(dead_code)]
pub fn wd2_set_input(d: &mut WeightDriver, v: f32) {
    d.input_param = v;
}

#[allow(dead_code)]
pub fn wd2_current_weight(d: &WeightDriver) -> f32 {
    wd2_evaluate(d, d.input_param)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_curve_point() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.0, 0.0);
        assert_eq!(d.curve.len(), 1);
    }

    #[test]
    fn test_evaluate_empty_returns_zero() {
        let d = new_weight_driver(0);
        assert_eq!(wd2_evaluate(&d, 0.5), 0.0);
    }

    #[test]
    fn test_evaluate_at_point() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.0, 0.0);
        wd2_add_curve_point(&mut d, 1.0, 1.0);
        assert!((wd2_evaluate(&d, 0.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_between() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.0, 0.0);
        wd2_add_curve_point(&mut d, 1.0, 1.0);
        assert!((wd2_evaluate(&d, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_set_input() {
        let mut d = new_weight_driver(0);
        wd2_set_input(&mut d, 0.7);
        assert!((d.input_param - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_current_weight() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.0, 0.0);
        wd2_add_curve_point(&mut d, 1.0, 1.0);
        wd2_set_input(&mut d, 0.5);
        assert!((wd2_current_weight(&d) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_before_range() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.5, 0.5);
        wd2_add_curve_point(&mut d, 1.0, 1.0);
        assert!((wd2_evaluate(&d, 0.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_after_range() {
        let mut d = new_weight_driver(0);
        wd2_add_curve_point(&mut d, 0.0, 0.0);
        wd2_add_curve_point(&mut d, 1.0, 0.8);
        assert!((wd2_evaluate(&d, 2.0) - 0.8).abs() < 1e-5);
    }
}
