// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct RadiationTransfer {
    pub emissivity: f32,
    pub absorptivity: f32,
    pub stefan_boltzmann: f32,
}

#[allow(dead_code)]
pub fn new_radiation_transfer(emissivity: f32) -> RadiationTransfer {
    RadiationTransfer {
        emissivity,
        absorptivity: emissivity,
        stefan_boltzmann: 5.67e-8,
    }
}

#[allow(dead_code)]
pub fn rt_emitted_power(r: &RadiationTransfer, area: f32, temp_k: f32) -> f32 {
    r.emissivity * r.stefan_boltzmann * area * temp_k.powi(4)
}

#[allow(dead_code)]
pub fn rt_absorbed_power(r: &RadiationTransfer, incident: f32) -> f32 {
    r.absorptivity * incident
}

#[allow(dead_code)]
pub fn rt_net_power(r: &RadiationTransfer, area: f32, temp_k: f32, incident: f32) -> f32 {
    rt_absorbed_power(r, incident) - rt_emitted_power(r, area, temp_k)
}

#[allow(dead_code)]
pub fn rt_equilibrium_temp(r: &RadiationTransfer, incident: f32) -> f32 {
    if r.emissivity < 1e-10 || r.stefan_boltzmann < 1e-30 {
        return 0.0;
    }
    (r.absorptivity * incident / (r.emissivity * r.stefan_boltzmann)).powf(0.25)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitted_power_positive() {
        let r = new_radiation_transfer(0.9);
        let p = rt_emitted_power(&r, 1.0, 300.0);
        assert!(p > 0.0);
    }

    #[test]
    fn test_emitted_power_t4() {
        let r = new_radiation_transfer(1.0);
        let p1 = rt_emitted_power(&r, 1.0, 100.0);
        let p2 = rt_emitted_power(&r, 1.0, 200.0);
        assert!((p2 / p1 - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_absorbed_power() {
        let r = new_radiation_transfer(0.8);
        let p = rt_absorbed_power(&r, 100.0);
        assert!((p - 80.0).abs() < 1e-4);
    }

    #[test]
    fn test_net_power_sign() {
        let r = new_radiation_transfer(1.0);
        let net = rt_net_power(&r, 1.0, 10000.0, 0.0);
        assert!(net < 0.0);
    }

    #[test]
    fn test_equilibrium_temp_positive() {
        let r = new_radiation_transfer(1.0);
        let t = rt_equilibrium_temp(&r, 1000.0);
        assert!(t > 0.0);
    }

    #[test]
    fn test_equilibrium_temp_stefan_boltzmann() {
        let r = new_radiation_transfer(1.0);
        let incident = 5.67e-8 * 300.0f32.powi(4);
        let t = rt_equilibrium_temp(&r, incident);
        assert!((t - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_kirchhoff_law() {
        let r = new_radiation_transfer(0.7);
        assert!((r.emissivity - r.absorptivity).abs() < 1e-6);
    }

    #[test]
    fn test_stefan_boltzmann_value() {
        let r = new_radiation_transfer(1.0);
        assert!((r.stefan_boltzmann - 5.67e-8).abs() < 1e-15);
    }
}
