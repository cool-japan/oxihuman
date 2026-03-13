// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Interstitial fluid compartment (Starling equilibrium).
pub struct InterstitialCompartment {
    pub pressure: f32,
    pub volume: f32,
    pub compliance: f32,
    pub lymph_flow_rate: f32,
    pub capillary_filtration: f32,
}

impl InterstitialCompartment {
    pub fn new() -> Self {
        InterstitialCompartment {
            pressure: -3.0,              // mmHg (sub-atmospheric at rest)
            volume: 11.0,                // L (total body interstitial)
            compliance: 1.0,             // L/mmHg
            lymph_flow_rate: 0.002,      // L/min
            capillary_filtration: 0.002, // L/min (balance)
        }
    }
}

impl Default for InterstitialCompartment {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_interstitial_compartment() -> InterstitialCompartment {
    InterstitialCompartment::new()
}

/// Net flow = filtration - lymph drainage (positive = accumulation)
pub fn interstitial_net_flow(c: &InterstitialCompartment) -> f32 {
    c.capillary_filtration - c.lymph_flow_rate
}

/// dV/dt = net_flow * dt; pressure adjusts via compliance
pub fn interstitial_step(c: &mut InterstitialCompartment, dt: f32) {
    let net = interstitial_net_flow(c);
    let dv = net * dt;
    c.volume += dv;
    if c.compliance > 0.0 {
        c.pressure += dv / c.compliance;
    }
}

/// Edematous when interstitial pressure > 0 mmHg
pub fn interstitial_is_edematous(c: &InterstitialCompartment) -> bool {
    c.pressure > 0.0
}

pub fn interstitial_volume_change(c: &InterstitialCompartment, dt: f32) -> f32 {
    interstitial_net_flow(c) * dt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        /* new compartment has sub-atmospheric pressure */
        let c = new_interstitial_compartment();
        assert!(c.pressure < 0.0);
    }

    #[test]
    fn test_net_flow_zero_at_balance() {
        /* at balance, net flow is zero */
        let c = new_interstitial_compartment();
        assert!(interstitial_net_flow(&c).abs() < 1e-9);
    }

    #[test]
    fn test_step_filtration_excess() {
        /* excess filtration raises pressure */
        let mut c = new_interstitial_compartment();
        c.capillary_filtration = 0.01;
        interstitial_step(&mut c, 1.0);
        /* pressure should increase */
        assert!(c.pressure > -3.0);
    }

    #[test]
    fn test_is_edematous_false_at_rest() {
        /* at rest, not edematous */
        let c = new_interstitial_compartment();
        assert!(!interstitial_is_edematous(&c));
    }

    #[test]
    fn test_is_edematous_true() {
        /* positive pressure is edematous */
        let mut c = new_interstitial_compartment();
        c.pressure = 2.0;
        assert!(interstitial_is_edematous(&c));
    }

    #[test]
    fn test_volume_change_sign() {
        /* excess filtration gives positive volume change */
        let mut c = new_interstitial_compartment();
        c.capillary_filtration = 0.01;
        let dv = interstitial_volume_change(&c, 1.0);
        assert!(dv > 0.0);
    }

    #[test]
    fn test_default() {
        /* Default impl works */
        let c = InterstitialCompartment::default();
        assert!(c.volume > 0.0);
    }
}
