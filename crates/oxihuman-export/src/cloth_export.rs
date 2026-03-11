// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export cloth simulation settings to JSON-compatible format.

#![allow(dead_code)]

/// Cloth simulation export data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothExport {
    pub name: String,
    pub vertex_mass: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub gravity: [f32; 3],
    pub wind: [f32; 3],
    pub pinned_vertices: Vec<u32>,
}

/// Create a default cloth export.
#[allow(dead_code)]
pub fn default_cloth_export(name: &str) -> ClothExport {
    ClothExport {
        name: name.to_string(),
        vertex_mass: 0.1,
        stiffness: 1000.0,
        damping: 0.01,
        gravity: [0.0, -9.81, 0.0],
        wind: [0.0, 0.0, 0.0],
        pinned_vertices: Vec::new(),
    }
}

/// Pin a vertex (prevent it from moving during simulation).
#[allow(dead_code)]
pub fn add_pin(exp: &mut ClothExport, vert: u32) {
    exp.pinned_vertices.push(vert);
}

/// Return the number of pinned vertices.
#[allow(dead_code)]
pub fn pin_count(exp: &ClothExport) -> usize {
    exp.pinned_vertices.len()
}

/// Serialize cloth export to a JSON string.
#[allow(dead_code)]
pub fn export_cloth_to_json(exp: &ClothExport) -> String {
    let pins: Vec<String> = exp.pinned_vertices.iter().map(|v| v.to_string()).collect();
    format!(
        r#"{{"name":"{name}","vertex_mass":{vm},"stiffness":{stiff},"damping":{damp},"gravity":[{gx},{gy},{gz}],"wind":[{wx},{wy},{wz}],"pinned_vertices":[{pins}]}}"#,
        name = exp.name,
        vm = exp.vertex_mass,
        stiff = exp.stiffness,
        damp = exp.damping,
        gx = exp.gravity[0], gy = exp.gravity[1], gz = exp.gravity[2],
        wx = exp.wind[0], wy = exp.wind[1], wz = exp.wind[2],
        pins = pins.join(","),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_name() {
        let exp = default_cloth_export("cloth_01");
        assert_eq!(exp.name, "cloth_01");
    }

    #[test]
    fn test_default_no_pins() {
        let exp = default_cloth_export("c");
        assert_eq!(pin_count(&exp), 0);
    }

    #[test]
    fn test_add_pin() {
        let mut exp = default_cloth_export("c");
        add_pin(&mut exp, 0);
        add_pin(&mut exp, 1);
        assert_eq!(pin_count(&exp), 2);
    }

    #[test]
    fn test_json_contains_name() {
        let exp = default_cloth_export("my_cloth");
        let json = export_cloth_to_json(&exp);
        assert!(json.contains("my_cloth"));
    }

    #[test]
    fn test_json_contains_stiffness() {
        let exp = default_cloth_export("c");
        let json = export_cloth_to_json(&exp);
        assert!(json.contains("stiffness"));
    }

    #[test]
    fn test_json_contains_pin() {
        let mut exp = default_cloth_export("c");
        add_pin(&mut exp, 42);
        let json = export_cloth_to_json(&exp);
        assert!(json.contains("42"));
    }

    #[test]
    fn test_default_vertex_mass_positive() {
        let exp = default_cloth_export("c");
        assert!(exp.vertex_mass > 0.0);
    }

    #[test]
    fn test_default_gravity_downward() {
        let exp = default_cloth_export("c");
        assert!(exp.gravity[1] < 0.0);
    }

    #[test]
    fn test_default_wind_zero() {
        let exp = default_cloth_export("c");
        assert!((exp.wind[0]).abs() < 1e-5);
        assert!((exp.wind[1]).abs() < 1e-5);
        assert!((exp.wind[2]).abs() < 1e-5);
    }

    #[test]
    fn test_pin_vertex_values() {
        let mut exp = default_cloth_export("c");
        add_pin(&mut exp, 100);
        add_pin(&mut exp, 200);
        assert_eq!(exp.pinned_vertices[0], 100);
        assert_eq!(exp.pinned_vertices[1], 200);
    }
}
