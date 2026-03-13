// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nociception (pain) threshold model with sensitization and adaptation.

pub struct NociceptorState {
    pub baseline_threshold: f32,
    pub sensitization: f32,
    pub adaptation: f32,
}

pub fn new_nociceptor() -> NociceptorState {
    NociceptorState {
        baseline_threshold: 1.0,
        sensitization: 0.0,
        adaptation: 0.0,
    }
}

pub fn noci_threshold(n: &NociceptorState) -> f32 {
    n.baseline_threshold / (1.0 + n.sensitization).max(1e-6)
}

pub fn noci_is_active(n: &NociceptorState, stimulus: f32) -> bool {
    stimulus > noci_threshold(n)
}

pub fn noci_sensitize(n: &mut NociceptorState, amount: f32) {
    n.sensitization = (n.sensitization + amount).max(0.0);
}

pub fn noci_adapt(n: &mut NociceptorState, dt: f32) {
    /* slow decay of sensitization over time */
    n.sensitization = (n.sensitization - 0.01 * dt).max(0.0);
    n.adaptation = (n.adaptation + 0.001 * dt).min(1.0);
}

pub fn noci_reset(n: &mut NociceptorState) {
    n.sensitization = 0.0;
    n.adaptation = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_nociceptor() {
        /* new nociceptor has default threshold */
        let n = new_nociceptor();
        assert!((noci_threshold(&n) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_noci_is_active_above_threshold() {
        /* active when stimulus exceeds threshold */
        let n = new_nociceptor();
        assert!(noci_is_active(&n, 2.0));
    }

    #[test]
    fn test_noci_is_not_active_below_threshold() {
        /* not active when stimulus below threshold */
        let n = new_nociceptor();
        assert!(!noci_is_active(&n, 0.5));
    }

    #[test]
    fn test_noci_sensitize() {
        /* sensitization lowers threshold */
        let mut n = new_nociceptor();
        noci_sensitize(&mut n, 1.0);
        assert!(noci_threshold(&n) < 1.0);
    }

    #[test]
    fn test_noci_adapt_reduces_sensitization() {
        /* adaptation reduces sensitization over time */
        let mut n = new_nociceptor();
        noci_sensitize(&mut n, 2.0);
        let sens_before = n.sensitization;
        noci_adapt(&mut n, 100.0);
        assert!(n.sensitization < sens_before);
    }

    #[test]
    fn test_noci_reset() {
        /* reset clears sensitization */
        let mut n = new_nociceptor();
        noci_sensitize(&mut n, 5.0);
        noci_reset(&mut n);
        assert_eq!(n.sensitization, 0.0);
    }

    #[test]
    fn test_noci_threshold_positive() {
        /* threshold is always positive */
        let n = new_nociceptor();
        assert!(noci_threshold(&n) > 0.0);
    }
}
