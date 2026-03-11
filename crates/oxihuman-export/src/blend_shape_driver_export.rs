// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export blend shape driver data (driver -> driven shape mapping).
#[allow(dead_code)]
pub struct BlendShapeDriver {
    pub name: String,
    pub driver_bone: String,
    pub axis: DriverAxis,
    pub input_range: [f32; 2],
    pub driven_shapes: Vec<DrivenShape>,
}

#[allow(dead_code)]
pub enum DriverAxis {
    RotX,
    RotY,
    RotZ,
    TransX,
    TransY,
    TransZ,
}

#[allow(dead_code)]
pub struct DrivenShape {
    pub shape_name: String,
    pub weight_at_min: f32,
    pub weight_at_max: f32,
}

#[allow(dead_code)]
pub struct BlendShapeDriverExport {
    pub drivers: Vec<BlendShapeDriver>,
}

#[allow(dead_code)]
pub fn new_blend_shape_driver_export() -> BlendShapeDriverExport {
    BlendShapeDriverExport { drivers: vec![] }
}

#[allow(dead_code)]
pub fn add_driver(export: &mut BlendShapeDriverExport, driver: BlendShapeDriver) {
    export.drivers.push(driver);
}

#[allow(dead_code)]
pub fn driver_count(export: &BlendShapeDriverExport) -> usize {
    export.drivers.len()
}

#[allow(dead_code)]
pub fn total_driven_shapes(export: &BlendShapeDriverExport) -> usize {
    export.drivers.iter().map(|d| d.driven_shapes.len()).sum()
}

#[allow(dead_code)]
pub fn find_driver_by_name<'a>(
    export: &'a BlendShapeDriverExport,
    name: &str,
) -> Option<&'a BlendShapeDriver> {
    export.drivers.iter().find(|d| d.name == name)
}

#[allow(dead_code)]
pub fn evaluate_driver(driver: &BlendShapeDriver, input: f32) -> Vec<(String, f32)> {
    let [min_in, max_in] = driver.input_range;
    let range = (max_in - min_in).abs();
    let t = if range < 1e-10 {
        0.0
    } else {
        ((input - min_in) / range).clamp(0.0, 1.0)
    };
    driver
        .driven_shapes
        .iter()
        .map(|s| {
            let w = s.weight_at_min + (s.weight_at_max - s.weight_at_min) * t;
            (s.shape_name.clone(), w.clamp(0.0, 1.0))
        })
        .collect()
}

#[allow(dead_code)]
pub fn driver_to_json(driver: &BlendShapeDriver) -> String {
    format!(
        "{{\"name\":\"{}\",\"driver_bone\":\"{}\",\"driven_shapes\":{}}}",
        driver.name,
        driver.driver_bone,
        driver.driven_shapes.len()
    )
}

#[allow(dead_code)]
pub fn blend_shape_driver_export_to_json(export: &BlendShapeDriverExport) -> String {
    format!(
        "{{\"driver_count\":{},\"total_driven_shapes\":{}}}",
        export.drivers.len(),
        total_driven_shapes(export)
    )
}

#[allow(dead_code)]
pub fn validate_driver(driver: &BlendShapeDriver) -> bool {
    driver.input_range[0] < driver.input_range[1]
        && driver
            .driven_shapes
            .iter()
            .all(|s| !s.shape_name.is_empty())
}

#[allow(dead_code)]
pub fn axis_name(axis: &DriverAxis) -> &'static str {
    match axis {
        DriverAxis::RotX => "rot_x",
        DriverAxis::RotY => "rot_y",
        DriverAxis::RotZ => "rot_z",
        DriverAxis::TransX => "trans_x",
        DriverAxis::TransY => "trans_y",
        DriverAxis::TransZ => "trans_z",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_driver() -> BlendShapeDriver {
        BlendShapeDriver {
            name: "smile_driver".to_string(),
            driver_bone: "jaw".to_string(),
            axis: DriverAxis::RotX,
            input_range: [0.0, 90.0],
            driven_shapes: vec![
                DrivenShape {
                    shape_name: "smile".to_string(),
                    weight_at_min: 0.0,
                    weight_at_max: 1.0,
                },
                DrivenShape {
                    shape_name: "cheek_puff".to_string(),
                    weight_at_min: 0.0,
                    weight_at_max: 0.5,
                },
            ],
        }
    }

    #[test]
    fn test_add_driver() {
        let mut e = new_blend_shape_driver_export();
        add_driver(&mut e, sample_driver());
        assert_eq!(driver_count(&e), 1);
    }

    #[test]
    fn test_total_driven_shapes() {
        let mut e = new_blend_shape_driver_export();
        add_driver(&mut e, sample_driver());
        assert_eq!(total_driven_shapes(&e), 2);
    }

    #[test]
    fn test_find_driver_found() {
        let mut e = new_blend_shape_driver_export();
        add_driver(&mut e, sample_driver());
        assert!(find_driver_by_name(&e, "smile_driver").is_some());
    }

    #[test]
    fn test_find_driver_not_found() {
        let e = new_blend_shape_driver_export();
        assert!(find_driver_by_name(&e, "missing").is_none());
    }

    #[test]
    fn test_evaluate_driver_at_max() {
        let d = sample_driver();
        let result = evaluate_driver(&d, 90.0);
        assert_eq!(result.len(), 2);
        assert!((result[0].1 - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_driver_at_min() {
        let d = sample_driver();
        let result = evaluate_driver(&d, 0.0);
        assert!((result[0].1).abs() < 1e-4);
    }

    #[test]
    fn test_evaluate_driver_midpoint() {
        let d = sample_driver();
        let result = evaluate_driver(&d, 45.0);
        assert!((result[0].1 - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_validate_driver_valid() {
        assert!(validate_driver(&sample_driver()));
    }

    #[test]
    fn test_to_json() {
        let d = sample_driver();
        let j = driver_to_json(&d);
        assert!(j.contains("smile_driver"));
    }

    #[test]
    fn test_axis_name() {
        assert_eq!(axis_name(&DriverAxis::RotX), "rot_x");
        assert_eq!(axis_name(&DriverAxis::TransZ), "trans_z");
    }
}
