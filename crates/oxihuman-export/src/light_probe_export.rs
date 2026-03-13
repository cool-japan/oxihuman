// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Export light probe / irradiance volume data.
#[allow(dead_code)]
pub enum ProbeType {
    Irradiance,
    Reflection,
    Combined,
}

#[allow(dead_code)]
pub struct LightProbe {
    pub name: String,
    pub position: [f32; 3],
    pub probe_type: ProbeType,
    pub radius: f32,
    pub intensity: f32,
    pub sh_coefficients: Vec<[f32; 3]>, // 9 SH bands RGB
}

#[allow(dead_code)]
pub struct LightProbeExport {
    pub probes: Vec<LightProbe>,
    pub grid_size: [u32; 3],
}

#[allow(dead_code)]
pub fn new_light_probe_export() -> LightProbeExport {
    LightProbeExport {
        probes: vec![],
        grid_size: [1, 1, 1],
    }
}

#[allow(dead_code)]
pub fn add_probe(export: &mut LightProbeExport, probe: LightProbe) {
    export.probes.push(probe);
}

#[allow(dead_code)]
pub fn probe_count(export: &LightProbeExport) -> usize {
    export.probes.len()
}

#[allow(dead_code)]
pub fn default_irradiance_probe(name: &str, position: [f32; 3]) -> LightProbe {
    LightProbe {
        name: name.to_string(),
        position,
        probe_type: ProbeType::Irradiance,
        radius: 5.0,
        intensity: 1.0,
        sh_coefficients: vec![[0.0; 3]; 9],
    }
}

/// Evaluate SH irradiance for a given direction (L0 + L1 approximation).
#[allow(dead_code)]
pub fn eval_sh_l1(sh: &[[f32; 3]], dir: [f32; 3]) -> [f32; 3] {
    if sh.is_empty() {
        return [0.0; 3];
    }
    let l0 = sh[0];
    let mut result = [l0[0] * 0.282095, l0[1] * 0.282095, l0[2] * 0.282095];
    if sh.len() >= 4 {
        let coeff = 0.488603;
        for c in 0..3 {
            result[c] += sh[1][c] * coeff * dir[1];
            result[c] += sh[2][c] * coeff * dir[2];
            result[c] += sh[3][c] * coeff * dir[0];
        }
    }
    result
}

#[allow(dead_code)]
pub fn probe_to_json(probe: &LightProbe) -> String {
    let ptype = match probe.probe_type {
        ProbeType::Irradiance => "irradiance",
        ProbeType::Reflection => "reflection",
        ProbeType::Combined => "combined",
    };
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"radius\":{},\"intensity\":{}}}",
        probe.name, ptype, probe.radius, probe.intensity
    )
}

#[allow(dead_code)]
pub fn light_probe_export_to_json(export: &LightProbeExport) -> String {
    format!("{{\"probe_count\":{}}}", export.probes.len())
}

#[allow(dead_code)]
pub fn validate_probe(probe: &LightProbe) -> bool {
    !probe.name.is_empty()
        && probe.radius > 0.0
        && probe.intensity >= 0.0
        && (probe.sh_coefficients.is_empty() || probe.sh_coefficients.len() == 9)
}

#[allow(dead_code)]
pub fn find_probe<'a>(export: &'a LightProbeExport, name: &str) -> Option<&'a LightProbe> {
    export.probes.iter().find(|p| p.name == name)
}

/// Build a probe grid along XZ plane.
#[allow(dead_code)]
pub fn probe_grid(origin: [f32; 3], grid: [u32; 3], spacing: f32) -> LightProbeExport {
    let mut e = new_light_probe_export();
    e.grid_size = grid;
    for xi in 0..grid[0] {
        for zi in 0..grid[2] {
            let pos = [
                origin[0] + xi as f32 * spacing,
                origin[1],
                origin[2] + zi as f32 * spacing,
            ];
            let name = format!("probe_{xi}_{zi}");
            e.probes.push(default_irradiance_probe(&name, pos));
        }
    }
    e
}

/// Compute total probe coverage area (pi * r^2 * count).
#[allow(dead_code)]
pub fn total_coverage_area(export: &LightProbeExport) -> f32 {
    export.probes.iter().map(|p| PI * p.radius * p.radius).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_probe() {
        let mut e = new_light_probe_export();
        add_probe(&mut e, default_irradiance_probe("p1", [0.0; 3]));
        assert_eq!(probe_count(&e), 1);
    }

    #[test]
    fn test_validate_probe() {
        let p = default_irradiance_probe("test", [0.0; 3]);
        assert!(validate_probe(&p));
    }

    #[test]
    fn test_find_probe_found() {
        let mut e = new_light_probe_export();
        add_probe(&mut e, default_irradiance_probe("main", [0.0; 3]));
        assert!(find_probe(&e, "main").is_some());
    }

    #[test]
    fn test_find_probe_missing() {
        let e = new_light_probe_export();
        assert!(find_probe(&e, "none").is_none());
    }

    #[test]
    fn test_probe_grid_count() {
        let e = probe_grid([0.0; 3], [3, 1, 3], 2.0);
        assert_eq!(probe_count(&e), 9);
    }

    #[test]
    fn test_total_coverage_positive() {
        let mut e = new_light_probe_export();
        add_probe(&mut e, default_irradiance_probe("p1", [0.0; 3]));
        assert!(total_coverage_area(&e) > 0.0);
    }

    #[test]
    fn test_eval_sh_l1_constant() {
        let sh = vec![[1.0f32, 1.0, 1.0]; 9];
        let c = eval_sh_l1(&sh, [0.0, 1.0, 0.0]);
        assert!(c[0] > 0.0);
    }

    #[test]
    fn test_eval_sh_empty() {
        let c = eval_sh_l1(&[], [0.0, 1.0, 0.0]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn test_to_json() {
        let p = default_irradiance_probe("test", [0.0; 3]);
        let j = probe_to_json(&p);
        assert!(j.contains("irradiance"));
    }

    #[test]
    fn test_export_to_json() {
        let mut e = new_light_probe_export();
        add_probe(&mut e, default_irradiance_probe("p", [0.0; 3]));
        let j = light_probe_export_to_json(&e);
        assert!(j.contains("probe_count"));
    }
}
