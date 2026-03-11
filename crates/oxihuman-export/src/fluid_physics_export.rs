// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export fluid simulation settings to JSON-compatible format.

#![allow(dead_code)]

/// Fluid simulation export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FluidExport {
    pub name: String,
    pub resolution: u32,
    pub viscosity: f32,
    pub density: f32,
    pub surface_tension: f32,
    pub gravity: [f32; 3],
}

/// Create a default fluid export.
#[allow(dead_code)]
pub fn default_fluid_export(name: &str) -> FluidExport {
    FluidExport {
        name: name.to_string(),
        resolution: 64,
        viscosity: 0.001,
        density: 1000.0,
        surface_tension: 0.072,
        gravity: [0.0, -9.81, 0.0],
    }
}

/// Serialize fluid export to a JSON string.
#[allow(dead_code)]
pub fn export_fluid_to_json(exp: &FluidExport) -> String {
    format!(
        r#"{{"name":"{name}","resolution":{res},"viscosity":{visc},"density":{dens},"surface_tension":{st},"gravity":[{gx},{gy},{gz}]}}"#,
        name = exp.name,
        res = exp.resolution,
        visc = exp.viscosity,
        dens = exp.density,
        st = exp.surface_tension,
        gx = exp.gravity[0], gy = exp.gravity[1], gz = exp.gravity[2],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_name() {
        let exp = default_fluid_export("water");
        assert_eq!(exp.name, "water");
    }

    #[test]
    fn test_default_resolution() {
        let exp = default_fluid_export("f");
        assert!(exp.resolution > 0);
    }

    #[test]
    fn test_default_density_positive() {
        let exp = default_fluid_export("f");
        assert!(exp.density > 0.0);
    }

    #[test]
    fn test_default_viscosity_positive() {
        let exp = default_fluid_export("f");
        assert!(exp.viscosity > 0.0);
    }

    #[test]
    fn test_default_gravity_downward() {
        let exp = default_fluid_export("f");
        assert!(exp.gravity[1] < 0.0);
    }

    #[test]
    fn test_json_contains_name() {
        let exp = default_fluid_export("ocean");
        let json = export_fluid_to_json(&exp);
        assert!(json.contains("ocean"));
    }

    #[test]
    fn test_json_contains_viscosity() {
        let exp = default_fluid_export("f");
        let json = export_fluid_to_json(&exp);
        assert!(json.contains("viscosity"));
    }

    #[test]
    fn test_json_contains_density() {
        let exp = default_fluid_export("f");
        let json = export_fluid_to_json(&exp);
        assert!(json.contains("density"));
    }

    #[test]
    fn test_json_contains_gravity() {
        let exp = default_fluid_export("f");
        let json = export_fluid_to_json(&exp);
        assert!(json.contains("gravity"));
    }

    #[test]
    fn test_surface_tension_positive() {
        let exp = default_fluid_export("f");
        assert!(exp.surface_tension > 0.0);
    }
}
