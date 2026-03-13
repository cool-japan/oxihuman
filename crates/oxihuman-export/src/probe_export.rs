#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export reflection/irradiance probe data.

use std::f32::consts::PI;

#[allow(dead_code)]
pub struct ProbeExport {
    pub name: String,
    pub probe_type: u8,
    pub position: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
    pub resolution: u32,
}

#[allow(dead_code)]
pub fn default_probe_export(name: &str) -> ProbeExport {
    ProbeExport {
        name: name.to_string(),
        probe_type: 0,
        position: [0.0, 0.0, 0.0],
        radius: 1.0,
        intensity: 1.0,
        resolution: 128,
    }
}

#[allow(dead_code)]
pub fn export_probe_to_json(p: &ProbeExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":{},\"position\":[{},{},{}],\"radius\":{},\"intensity\":{},\"resolution\":{}}}",
        p.name, p.probe_type,
        p.position[0], p.position[1], p.position[2],
        p.radius, p.intensity, p.resolution
    )
}

/// Volume of the sphere influence region: (4/3) * PI * r^3.
#[allow(dead_code)]
pub fn probe_volume(p: &ProbeExport) -> f32 {
    (4.0 / 3.0) * PI * p.radius * p.radius * p.radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn default_probe_name() {
        let p = default_probe_export("env_probe");
        assert_eq!(p.name, "env_probe");
    }

    #[test]
    fn default_radius_one() {
        let p = default_probe_export("x");
        assert!((p.radius - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_resolution() {
        let p = default_probe_export("x");
        assert_eq!(p.resolution, 128);
    }

    #[test]
    fn default_intensity_one() {
        let p = default_probe_export("x");
        assert!((p.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn probe_volume_unit_sphere() {
        let p = default_probe_export("x");
        let v = probe_volume(&p);
        let expected = (4.0 / 3.0) * PI;
        assert!((v - expected).abs() < 1e-4, "expected ~{expected}, got {v}");
    }

    #[test]
    fn probe_volume_double_radius() {
        let mut p = default_probe_export("x");
        p.radius = 2.0;
        let v = probe_volume(&p);
        let expected = (4.0 / 3.0) * PI * 8.0;
        assert!((v - expected).abs() < 1e-3);
    }

    #[test]
    fn json_contains_name() {
        let p = default_probe_export("sky_probe");
        let json = export_probe_to_json(&p);
        assert!(json.contains("sky_probe"));
    }

    #[test]
    fn json_contains_radius() {
        let p = default_probe_export("x");
        let json = export_probe_to_json(&p);
        assert!(json.contains("radius"));
    }

    #[test]
    fn json_valid_brackets() {
        let p = default_probe_export("x");
        let json = export_probe_to_json(&p);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn probe_type_stored() {
        let mut p = default_probe_export("x");
        p.probe_type = 1;
        assert_eq!(p.probe_type, 1);
    }
}
