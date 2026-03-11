// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export key-driven shape / property driver data.

/// A driver curve mapping an input value to an output value.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DriverCurve {
    pub input_min: f32,
    pub input_max: f32,
    pub output_min: f32,
    pub output_max: f32,
    pub clamp_output: bool,
}

impl Default for DriverCurve {
    fn default() -> Self {
        Self {
            input_min: 0.0,
            input_max: 1.0,
            output_min: 0.0,
            output_max: 1.0,
            clamp_output: true,
        }
    }
}

/// A single key driver connecting a source property to a target.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KeyDriver {
    pub name: String,
    pub source_bone: String,
    pub source_property: String,
    pub target_shape: String,
    pub curve: DriverCurve,
}

/// Key driver export collection.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct KeyDriverExport {
    pub drivers: Vec<KeyDriver>,
}

/// Create a new export.
#[allow(dead_code)]
pub fn new_key_driver_export() -> KeyDriverExport {
    KeyDriverExport::default()
}

/// Add a driver.
#[allow(dead_code)]
pub fn add_driver(export: &mut KeyDriverExport, driver: KeyDriver) {
    export.drivers.push(driver);
}

/// Evaluate a driver curve for a given input.
#[allow(dead_code)]
pub fn evaluate_curve(curve: &DriverCurve, input: f32) -> f32 {
    let range_in = curve.input_max - curve.input_min;
    let t = if range_in.abs() < 1e-8 {
        0.0
    } else {
        (input - curve.input_min) / range_in
    };
    let out = curve.output_min + t * (curve.output_max - curve.output_min);
    if curve.clamp_output {
        out.clamp(
            curve.output_min.min(curve.output_max),
            curve.output_min.max(curve.output_max),
        )
    } else {
        out
    }
}

/// Evaluate a named driver at the given input.
#[allow(dead_code)]
pub fn evaluate_driver(export: &KeyDriverExport, name: &str, input: f32) -> Option<f32> {
    export
        .drivers
        .iter()
        .find(|d| d.name == name)
        .map(|d| evaluate_curve(&d.curve, input))
}

/// Find all drivers targeting a given shape.
#[allow(dead_code)]
pub fn drivers_for_shape<'a>(export: &'a KeyDriverExport, shape: &str) -> Vec<&'a KeyDriver> {
    export
        .drivers
        .iter()
        .filter(|d| d.target_shape == shape)
        .collect()
}

/// Count drivers.
#[allow(dead_code)]
pub fn driver_count(export: &KeyDriverExport) -> usize {
    export.drivers.len()
}

/// Serialise curve to flat buffer.
#[allow(dead_code)]
pub fn serialise_curve(curve: &DriverCurve) -> Vec<f32> {
    vec![
        curve.input_min,
        curve.input_max,
        curve.output_min,
        curve.output_max,
    ]
}

/// Check all drivers have distinct names.
#[allow(dead_code)]
pub fn names_unique(export: &KeyDriverExport) -> bool {
    let mut seen = std::collections::HashSet::new();
    export.drivers.iter().all(|d| seen.insert(d.name.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_driver(name: &str, shape: &str) -> KeyDriver {
        KeyDriver {
            name: name.to_string(),
            source_bone: "arm".to_string(),
            source_property: "rotation_x".to_string(),
            target_shape: shape.to_string(),
            curve: DriverCurve::default(),
        }
    }

    #[test]
    fn test_evaluate_curve_midpoint() {
        let c = DriverCurve::default();
        assert!((evaluate_curve(&c, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_curve_min() {
        let c = DriverCurve::default();
        assert!((evaluate_curve(&c, 0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_curve_max() {
        let c = DriverCurve::default();
        assert!((evaluate_curve(&c, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_curve_clamp() {
        let c = DriverCurve::default();
        let out = evaluate_curve(&c, 2.0);
        assert!((0.0..=1.0).contains(&out));
    }

    #[test]
    fn test_add_driver() {
        let mut e = new_key_driver_export();
        add_driver(&mut e, sample_driver("d1", "smile"));
        assert_eq!(driver_count(&e), 1);
    }

    #[test]
    fn test_evaluate_driver_found() {
        let mut e = new_key_driver_export();
        add_driver(&mut e, sample_driver("brow_up", "brow"));
        let v = evaluate_driver(&e, "brow_up", 0.5);
        assert!(v.is_some());
    }

    #[test]
    fn test_evaluate_driver_not_found() {
        let e = new_key_driver_export();
        assert!(evaluate_driver(&e, "missing", 0.5).is_none());
    }

    #[test]
    fn test_drivers_for_shape() {
        let mut e = new_key_driver_export();
        add_driver(&mut e, sample_driver("d1", "smile"));
        add_driver(&mut e, sample_driver("d2", "frown"));
        let r = drivers_for_shape(&e, "smile");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_serialise_curve_length() {
        assert_eq!(serialise_curve(&DriverCurve::default()).len(), 4);
    }

    #[test]
    fn test_names_unique() {
        let mut e = new_key_driver_export();
        add_driver(&mut e, sample_driver("d1", "s"));
        add_driver(&mut e, sample_driver("d2", "s"));
        assert!(names_unique(&e));
    }
}
