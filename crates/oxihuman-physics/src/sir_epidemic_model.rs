// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SirModel {
    pub s: f32,
    pub i: f32,
    pub r: f32,
    pub beta: f32,
    pub gamma: f32,
}

pub fn new_sir_model(population: f32, initial_infected: f32, beta: f32, gamma: f32) -> SirModel {
    SirModel {
        s: population - initial_infected,
        i: initial_infected,
        r: 0.0,
        beta,
        gamma,
    }
}

pub fn sir_step(m: &mut SirModel, dt: f32) {
    let n = m.s + m.i + m.r;
    if n < 1e-10 {
        return;
    }
    let ds = -m.beta * m.s * m.i / n;
    let di = m.beta * m.s * m.i / n - m.gamma * m.i;
    let dr = m.gamma * m.i;
    m.s += ds * dt;
    m.i += di * dt;
    m.r += dr * dt;
    /* keep non-negative */
    m.s = m.s.max(0.0);
    m.i = m.i.max(0.0);
    m.r = m.r.max(0.0);
}

pub fn sir_r0(m: &SirModel) -> f32 {
    if m.gamma < 1e-10 {
        return f32::INFINITY;
    }
    m.beta / m.gamma
}

pub fn sir_herd_immunity_threshold(m: &SirModel) -> f32 {
    let r0 = sir_r0(m);
    if r0 <= 1.0 {
        return 0.0;
    }
    1.0 - 1.0 / r0
}

pub fn sir_is_epidemic(m: &SirModel) -> bool {
    sir_r0(m) > 1.0
}

pub fn sir_total(m: &SirModel) -> f32 {
    m.s + m.i + m.r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sir_model() {
        /* initial conditions set correctly */
        let m = new_sir_model(1000.0, 10.0, 0.3, 0.1);
        assert!((m.s - 990.0).abs() < 1e-4);
        assert!((m.i - 10.0).abs() < 1e-4);
        assert_eq!(m.r, 0.0);
    }

    #[test]
    fn test_sir_r0() {
        /* R0 = beta / gamma */
        let m = new_sir_model(1000.0, 1.0, 0.3, 0.1);
        assert!((sir_r0(&m) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_sir_is_epidemic() {
        /* R0 > 1 means epidemic */
        let m = new_sir_model(1000.0, 1.0, 0.5, 0.1);
        assert!(sir_is_epidemic(&m));
    }

    #[test]
    fn test_sir_not_epidemic() {
        /* R0 < 1 means no epidemic */
        let m = new_sir_model(1000.0, 1.0, 0.05, 0.3);
        assert!(!sir_is_epidemic(&m));
    }

    #[test]
    fn test_sir_step_conserves_population() {
        /* total population constant during step */
        let mut m = new_sir_model(1000.0, 10.0, 0.3, 0.1);
        let n_before = sir_total(&m);
        sir_step(&mut m, 0.1);
        let n_after = sir_total(&m);
        assert!((n_before - n_after).abs() < 1e-2);
    }

    #[test]
    fn test_sir_herd_immunity_threshold() {
        /* herd immunity threshold = 1 - 1/R0 */
        let m = new_sir_model(1000.0, 1.0, 0.3, 0.1);
        let hit = sir_herd_immunity_threshold(&m);
        assert!((hit - (1.0 - 1.0 / 3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_sir_infected_grows_initially() {
        /* infected count grows when R0 > 1 */
        let mut m = new_sir_model(10000.0, 100.0, 0.5, 0.1);
        let i_before = m.i;
        sir_step(&mut m, 0.5);
        assert!(m.i > i_before);
    }
}
