// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export geometry scatter / instancing distribution data.
#[allow(dead_code)]
pub struct ScatterPoint {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub scale: f32,
    pub rotation_y: f32,
    pub instance_id: u32,
}

#[allow(dead_code)]
pub struct GeometryScatterExport {
    pub points: Vec<ScatterPoint>,
    pub instance_names: Vec<String>,
}

#[allow(dead_code)]
pub fn new_geometry_scatter_export() -> GeometryScatterExport {
    GeometryScatterExport {
        points: vec![],
        instance_names: vec![],
    }
}

#[allow(dead_code)]
pub fn add_scatter_point(export: &mut GeometryScatterExport, point: ScatterPoint) {
    export.points.push(point);
}

#[allow(dead_code)]
pub fn add_instance_name(export: &mut GeometryScatterExport, name: &str) {
    export.instance_names.push(name.to_string());
}

#[allow(dead_code)]
pub fn scatter_point_count(export: &GeometryScatterExport) -> usize {
    export.points.len()
}

#[allow(dead_code)]
pub fn instance_type_count(export: &GeometryScatterExport) -> usize {
    export.instance_names.len()
}

#[allow(dead_code)]
pub fn points_of_instance(export: &GeometryScatterExport, id: u32) -> Vec<&ScatterPoint> {
    export
        .points
        .iter()
        .filter(|p| p.instance_id == id)
        .collect()
}

#[allow(dead_code)]
pub fn scatter_bounds(export: &GeometryScatterExport) -> ([f32; 3], [f32; 3]) {
    if export.points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = export.points[0].position;
    let mut mx = export.points[0].position;
    for p in &export.points {
        let pos = p.position;
        for k in 0..3 {
            if pos[k] < mn[k] {
                mn[k] = pos[k];
            }
            if pos[k] > mx[k] {
                mx[k] = pos[k];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn avg_scale(export: &GeometryScatterExport) -> f32 {
    if export.points.is_empty() {
        return 0.0;
    }
    export.points.iter().map(|p| p.scale).sum::<f32>() / export.points.len() as f32
}

#[allow(dead_code)]
pub fn validate_scatter(export: &GeometryScatterExport) -> bool {
    export
        .points
        .iter()
        .all(|p| p.scale > 0.0 && p.instance_id < export.instance_names.len() as u32)
}

#[allow(dead_code)]
pub fn scatter_to_json(export: &GeometryScatterExport) -> String {
    format!(
        "{{\"point_count\":{},\"instance_count\":{}}}",
        export.points.len(),
        export.instance_names.len()
    )
}

/// Build a simple grid scatter.
#[allow(dead_code)]
pub fn grid_scatter(
    origin: [f32; 3],
    grid_size: u32,
    spacing: f32,
    instance_id: u32,
) -> GeometryScatterExport {
    let mut e = new_geometry_scatter_export();
    for xi in 0..grid_size {
        for zi in 0..grid_size {
            let x = origin[0] + xi as f32 * spacing;
            let z = origin[2] + zi as f32 * spacing;
            e.points.push(ScatterPoint {
                position: [x, origin[1], z],
                normal: [0.0, 1.0, 0.0],
                scale: 1.0,
                rotation_y: 0.0,
                instance_id,
            });
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scatter() -> GeometryScatterExport {
        let mut e = new_geometry_scatter_export();
        add_instance_name(&mut e, "tree");
        add_instance_name(&mut e, "rock");
        add_scatter_point(
            &mut e,
            ScatterPoint {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                scale: 1.0,
                rotation_y: 0.0,
                instance_id: 0,
            },
        );
        add_scatter_point(
            &mut e,
            ScatterPoint {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                scale: 0.5,
                rotation_y: 0.5,
                instance_id: 1,
            },
        );
        e
    }

    #[test]
    fn test_scatter_point_count() {
        let e = sample_scatter();
        assert_eq!(scatter_point_count(&e), 2);
    }

    #[test]
    fn test_instance_type_count() {
        let e = sample_scatter();
        assert_eq!(instance_type_count(&e), 2);
    }

    #[test]
    fn test_points_of_instance() {
        let e = sample_scatter();
        let pts = points_of_instance(&e, 0);
        assert_eq!(pts.len(), 1);
    }

    #[test]
    fn test_avg_scale() {
        let e = sample_scatter();
        assert!((avg_scale(&e) - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_validate_scatter() {
        let e = sample_scatter();
        assert!(validate_scatter(&e));
    }

    #[test]
    fn test_scatter_bounds() {
        let e = sample_scatter();
        let (mn, mx) = scatter_bounds(&e);
        assert!(mx[0] >= mn[0]);
    }

    #[test]
    fn test_grid_scatter_count() {
        let e = grid_scatter([0.0; 3], 4, 1.0, 0);
        assert_eq!(scatter_point_count(&e), 16);
    }

    #[test]
    fn test_to_json() {
        let e = sample_scatter();
        let j = scatter_to_json(&e);
        assert!(j.contains("point_count"));
    }

    #[test]
    fn test_empty_scatter_bounds() {
        let e = new_geometry_scatter_export();
        let (mn, mx) = scatter_bounds(&e);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }

    #[test]
    fn test_empty_avg_scale() {
        let e = new_geometry_scatter_export();
        assert_eq!(avg_scale(&e), 0.0);
    }
}
