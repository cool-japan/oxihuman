// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct SphPhase {
    pub density_0: f32,
    pub viscosity: f32,
    pub label: u8,
}

#[allow(dead_code)]
pub struct SphMultiphase {
    pub phases: Vec<SphPhase>,
}

#[allow(dead_code)]
pub fn new_sph_multiphase() -> SphMultiphase {
    SphMultiphase { phases: Vec::new() }
}

#[allow(dead_code)]
pub fn spm_add_phase(m: &mut SphMultiphase, density_0: f32, viscosity: f32, label: u8) {
    m.phases.push(SphPhase { density_0, viscosity, label });
}

#[allow(dead_code)]
pub fn spm_phase_count(m: &SphMultiphase) -> usize {
    m.phases.len()
}

#[allow(dead_code)]
pub fn spm_get_phase(m: &SphMultiphase, label: u8) -> Option<&SphPhase> {
    m.phases.iter().find(|p| p.label == label)
}

#[allow(dead_code)]
pub fn spm_avg_density(m: &SphMultiphase) -> f32 {
    if m.phases.is_empty() {
        return 0.0;
    }
    m.phases.iter().map(|p| p.density_0).sum::<f32>() / m.phases.len() as f32
}

#[allow(dead_code)]
pub fn spm_interface_tension(m: &SphMultiphase, a: u8, b: u8) -> f32 {
    let da = m.phases.iter().find(|p| p.label == a).map(|p| p.density_0).unwrap_or(0.0);
    let db = m.phases.iter().find(|p| p.label == b).map(|p| p.density_0).unwrap_or(0.0);
    (da - db).abs() * 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let m = new_sph_multiphase();
        assert_eq!(spm_phase_count(&m), 0);
    }

    #[test]
    fn test_add_phase() {
        let mut m = new_sph_multiphase();
        spm_add_phase(&mut m, 1000.0, 0.001, 0);
        assert_eq!(spm_phase_count(&m), 1);
    }

    #[test]
    fn test_get_phase() {
        let mut m = new_sph_multiphase();
        spm_add_phase(&mut m, 1000.0, 0.001, 1);
        let phase = spm_get_phase(&m, 1);
        assert!(phase.is_some());
        assert!((phase.expect("should succeed").density_0 - 1000.0).abs() < 0.1);
    }

    #[test]
    fn test_get_phase_missing() {
        let m = new_sph_multiphase();
        assert!(spm_get_phase(&m, 99).is_none());
    }

    #[test]
    fn test_avg_density() {
        let mut m = new_sph_multiphase();
        spm_add_phase(&mut m, 1000.0, 0.001, 0);
        spm_add_phase(&mut m, 800.0, 0.01, 1);
        assert!((spm_avg_density(&m) - 900.0).abs() < 0.1);
    }

    #[test]
    fn test_avg_density_empty() {
        let m = new_sph_multiphase();
        assert!((spm_avg_density(&m)).abs() < 1e-6);
    }

    #[test]
    fn test_interface_tension() {
        let mut m = new_sph_multiphase();
        spm_add_phase(&mut m, 1000.0, 0.001, 0);
        spm_add_phase(&mut m, 800.0, 0.01, 1);
        let t = spm_interface_tension(&m, 0, 1);
        assert!((t - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_interface_tension_same_density() {
        let mut m = new_sph_multiphase();
        spm_add_phase(&mut m, 1000.0, 0.001, 0);
        spm_add_phase(&mut m, 1000.0, 0.001, 1);
        let t = spm_interface_tension(&m, 0, 1);
        assert!((t).abs() < 1e-6);
    }
}
