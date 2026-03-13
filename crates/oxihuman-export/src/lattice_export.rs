// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Exported lattice (free-form deformation cage) data.
#[allow(dead_code)]
pub struct LatticeExport {
    pub name: String,
    pub u: u32,
    pub v: u32,
    pub w: u32,
    pub points: Vec<[f32; 3]>,
    pub interpolation: u8,
}

/// Create a new lattice export; generates default grid points.
#[allow(dead_code)]
pub fn new_lattice_export(name: &str, u: u32, v: u32, w: u32) -> LatticeExport {
    let mut points = Vec::new();
    let u = u.max(2);
    let v = v.max(2);
    let w = w.max(2);
    for wi in 0..w {
        for vi in 0..v {
            for ui in 0..u {
                let x = (ui as f32) / (u - 1) as f32 - 0.5;
                let y = (vi as f32) / (v - 1) as f32 - 0.5;
                let z = (wi as f32) / (w - 1) as f32 - 0.5;
                points.push([x, y, z]);
            }
        }
    }
    LatticeExport {
        name: name.to_string(),
        u,
        v,
        w,
        points,
        interpolation: 0,
    }
}

/// Number of lattice control points.
#[allow(dead_code)]
pub fn lattice_point_count(l: &LatticeExport) -> usize {
    l.points.len()
}

/// Export the lattice to a JSON string.
#[allow(dead_code)]
pub fn export_lattice_to_json(l: &LatticeExport) -> String {
    let pts: String = l
        .points
        .iter()
        .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"name":"{}","u":{},"v":{},"w":{},"interpolation":{},"points":[{}]}}"#,
        l.name, l.u, l.v, l.w, l.interpolation, pts
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lattice_export_name() {
        let l = new_lattice_export("Lattice", 2, 2, 2);
        assert_eq!(l.name, "Lattice");
    }

    #[test]
    fn test_new_lattice_export_point_count_2x2x2() {
        let l = new_lattice_export("L", 2, 2, 2);
        assert_eq!(lattice_point_count(&l), 8);
    }

    #[test]
    fn test_new_lattice_export_point_count_3x2x2() {
        let l = new_lattice_export("L", 3, 2, 2);
        assert_eq!(lattice_point_count(&l), 12);
    }

    #[test]
    fn test_new_lattice_minimum_dimensions() {
        // dimensions below 2 are clamped to 2
        let l = new_lattice_export("L", 1, 1, 1);
        assert_eq!(lattice_point_count(&l), 8);
    }

    #[test]
    fn test_lattice_point_count_helper() {
        let l = new_lattice_export("L", 2, 3, 2);
        assert_eq!(lattice_point_count(&l), l.points.len());
    }

    #[test]
    fn test_export_lattice_to_json_name() {
        let l = new_lattice_export("MyLattice", 2, 2, 2);
        let json = export_lattice_to_json(&l);
        assert!(json.contains("MyLattice"));
    }

    #[test]
    fn test_export_lattice_to_json_has_points() {
        let l = new_lattice_export("L", 2, 2, 2);
        let json = export_lattice_to_json(&l);
        assert!(json.contains("points"));
    }

    #[test]
    fn test_export_lattice_to_json_structure() {
        let l = new_lattice_export("L", 2, 2, 2);
        let json = export_lattice_to_json(&l);
        assert!(json.starts_with('{') && json.ends_with('}'));
    }

    #[test]
    fn test_lattice_corner_positions() {
        let l = new_lattice_export("L", 2, 2, 2);
        // first point is at (-0.5, -0.5, -0.5)
        assert!((l.points[0][0] + 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_export_lattice_dimensions() {
        let l = new_lattice_export("L", 3, 3, 2);
        let json = export_lattice_to_json(&l);
        assert!(json.contains("\"u\":3"));
    }
}
