// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct LotkaVolterra {
    pub prey: f32,
    pub predator: f32,
    pub alpha: f32,
    pub beta: f32,
    pub delta: f32,
    pub gamma: f32,
}

pub fn new_lotka_volterra(prey: f32, pred: f32) -> LotkaVolterra {
    LotkaVolterra {
        prey,
        predator: pred,
        alpha: 1.0,
        beta: 0.1,
        delta: 0.075,
        gamma: 1.5,
    }
}

pub fn lv_step(m: &mut LotkaVolterra, dt: f32) {
    let dx = m.alpha * m.prey - m.beta * m.prey * m.predator;
    let dy = m.delta * m.prey * m.predator - m.gamma * m.predator;
    m.prey = (m.prey + dx * dt).max(0.0);
    m.predator = (m.predator + dy * dt).max(0.0);
}

pub fn lv_prey_growth(m: &LotkaVolterra) -> f32 {
    m.alpha * m.prey
}

pub fn lv_predator_loss(m: &LotkaVolterra) -> f32 {
    m.gamma * m.predator
}

pub fn lv_total(m: &LotkaVolterra) -> f32 {
    m.prey + m.predator
}

pub fn lv_is_stable(m: &LotkaVolterra) -> bool {
    m.prey > 0.0 && m.predator > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lotka_volterra() {
        /* initial populations set correctly */
        let m = new_lotka_volterra(40.0, 9.0);
        assert!((m.prey - 40.0).abs() < 1e-5);
        assert!((m.predator - 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_lv_prey_growth() {
        /* prey growth is alpha * prey */
        let m = new_lotka_volterra(10.0, 5.0);
        assert!((lv_prey_growth(&m) - m.alpha * 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_lv_predator_loss() {
        /* predator natural loss */
        let m = new_lotka_volterra(10.0, 5.0);
        assert!((lv_predator_loss(&m) - m.gamma * 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_lv_total() {
        /* total is sum of both populations */
        let m = new_lotka_volterra(10.0, 5.0);
        assert!((lv_total(&m) - 15.0).abs() < 1e-5);
    }

    #[test]
    fn test_lv_is_stable_both_positive() {
        /* stable when both > 0 */
        let m = new_lotka_volterra(10.0, 5.0);
        assert!(lv_is_stable(&m));
    }

    #[test]
    fn test_lv_step_runs() {
        /* step does not produce NaN */
        let mut m = new_lotka_volterra(40.0, 9.0);
        lv_step(&mut m, 0.01);
        assert!(m.prey.is_finite());
        assert!(m.predator.is_finite());
    }

    #[test]
    fn test_lv_step_no_negative() {
        /* populations remain non-negative */
        let mut m = new_lotka_volterra(1.0, 100.0);
        for _ in 0..100 {
            lv_step(&mut m, 0.1);
        }
        assert!(m.prey >= 0.0);
        assert!(m.predator >= 0.0);
    }
}
