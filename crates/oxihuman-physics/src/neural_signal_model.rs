// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! FitzHugh-Nagumo neuron model (simplified Hodgkin-Huxley).

pub struct FitzHughNagumo {
    pub v: f32,
    pub w: f32,
    pub a: f32,
    pub b: f32,
    pub tau: f32,
    pub i_ext: f32,
}

pub fn new_fitzhugh_nagumo() -> FitzHughNagumo {
    FitzHughNagumo {
        v: 0.0,
        w: 0.0,
        a: 0.7,
        b: 0.8,
        tau: 12.5,
        i_ext: 0.0,
    }
}

pub fn fhn_step(n: &mut FitzHughNagumo, dt: f32) {
    /* dv/dt = v - v³/3 - w + I_ext */
    let dv = n.v - n.v * n.v * n.v / 3.0 - n.w + n.i_ext;
    /* dw/dt = (v + a - b*w) / tau */
    let dw = (n.v + n.a - n.b * n.w) / n.tau;
    n.v += dv * dt;
    n.w += dw * dt;
}

pub fn fhn_is_spiking(n: &FitzHughNagumo) -> bool {
    n.v > 0.5
}

pub fn fhn_set_current(n: &mut FitzHughNagumo, i: f32) {
    n.i_ext = i;
}

pub fn fhn_membrane_potential(n: &FitzHughNagumo) -> f32 {
    n.v
}

pub fn fhn_recovery(n: &FitzHughNagumo) -> f32 {
    n.w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fitzhugh_nagumo() {
        /* new neuron starts at rest */
        let n = new_fitzhugh_nagumo();
        assert_eq!(n.v, 0.0);
        assert_eq!(n.w, 0.0);
    }

    #[test]
    fn test_fhn_step_no_current() {
        /* without current, neuron remains near rest */
        let mut n = new_fitzhugh_nagumo();
        for _ in 0..100 {
            fhn_step(&mut n, 0.01);
        }
        assert!(n.v.abs() < 2.0);
    }

    #[test]
    fn test_fhn_spiking_with_current() {
        /* with sufficient current, neuron eventually spikes */
        let mut n = new_fitzhugh_nagumo();
        fhn_set_current(&mut n, 0.8);
        let mut spiked = false;
        for _ in 0..1000 {
            fhn_step(&mut n, 0.05);
            if fhn_is_spiking(&n) {
                spiked = true;
                break;
            }
        }
        assert!(spiked);
    }

    #[test]
    fn test_fhn_set_current() {
        /* set_current updates i_ext */
        let mut n = new_fitzhugh_nagumo();
        fhn_set_current(&mut n, 1.5);
        assert!((n.i_ext - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_fhn_membrane_potential() {
        /* membrane_potential returns v */
        let n = new_fitzhugh_nagumo();
        assert_eq!(fhn_membrane_potential(&n), 0.0);
    }

    #[test]
    fn test_fhn_recovery() {
        /* recovery variable is initially 0 */
        let n = new_fitzhugh_nagumo();
        assert_eq!(fhn_recovery(&n), 0.0);
    }

    #[test]
    fn test_fhn_parameters() {
        /* default parameters match spec */
        let n = new_fitzhugh_nagumo();
        assert!((n.a - 0.7).abs() < 1e-5);
        assert!((n.b - 0.8).abs() < 1e-5);
        assert!((n.tau - 12.5).abs() < 1e-5);
    }
}
