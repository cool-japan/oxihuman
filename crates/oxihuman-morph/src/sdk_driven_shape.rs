// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Set-driven-key shape driver — maps a driver attribute to shape weights.

/// A single control point in the set-driven-key curve.
#[derive(Debug, Clone, Copy)]
pub struct SdkCurvePoint {
    pub driver_value: f32,
    pub shape_weight: f32,
}

/// Set-driven-key shape driver.
#[derive(Debug, Clone)]
pub struct SdkDrivenShape {
    pub driver_attr: String,
    pub shape_name: String,
    pub curve: Vec<SdkCurvePoint>,
    pub current_weight: f32,
}

impl SdkDrivenShape {
    pub fn new(driver_attr: &str, shape_name: &str) -> Self {
        SdkDrivenShape {
            driver_attr: driver_attr.to_string(),
            shape_name: shape_name.to_string(),
            curve: Vec::new(),
            current_weight: 0.0,
        }
    }
}

/// Create a new SDK driven shape.
pub fn new_sdk_driven_shape(driver_attr: &str, shape_name: &str) -> SdkDrivenShape {
    SdkDrivenShape::new(driver_attr, shape_name)
}

/// Add a curve control point.
pub fn sdk_add_point(shape: &mut SdkDrivenShape, driver_value: f32, shape_weight: f32) {
    shape.curve.push(SdkCurvePoint {
        driver_value,
        shape_weight: shape_weight.clamp(0.0, 1.0),
    });
    shape.curve.sort_by(|a, b| {
        a.driver_value
            .partial_cmp(&b.driver_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Evaluate the shape weight for a given driver value.
pub fn sdk_evaluate(shape: &mut SdkDrivenShape, driver_value: f32) -> f32 {
    if shape.curve.is_empty() {
        shape.current_weight = 0.0;
        return 0.0;
    }
    let first = shape.curve[0];
    let last = shape.curve[shape.curve.len() - 1];
    if driver_value <= first.driver_value {
        shape.current_weight = first.shape_weight;
        return first.shape_weight;
    }
    if driver_value >= last.driver_value {
        shape.current_weight = last.shape_weight;
        return last.shape_weight;
    }
    for i in 0..shape.curve.len().saturating_sub(1) {
        let a = shape.curve[i];
        let b = shape.curve[i + 1];
        if driver_value >= a.driver_value && driver_value <= b.driver_value {
            let t = (driver_value - a.driver_value) / (b.driver_value - a.driver_value);
            let w = a.shape_weight + t * (b.shape_weight - a.shape_weight);
            shape.current_weight = w;
            return w;
        }
    }
    shape.current_weight = 0.0;
    0.0
}

/// Reset the shape to zero weight.
pub fn sdk_reset(shape: &mut SdkDrivenShape) {
    shape.current_weight = 0.0;
}

/// Return a JSON-like string representation.
pub fn sdk_to_json(shape: &SdkDrivenShape) -> String {
    format!(
        r#"{{"driver":"{}","shape":"{}","weight":{:.4},"points":{}}}"#,
        shape.driver_attr,
        shape.shape_name,
        shape.current_weight,
        shape.curve.len()
    )
}

/// Return the number of control points.
pub fn sdk_point_count(shape: &SdkDrivenShape) -> usize {
    shape.curve.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sdk_shape_empty() {
        let s = new_sdk_driven_shape("jaw_open", "mouth_open");
        assert_eq!(sdk_point_count(&s), 0 /* new shape has no points */,);
    }

    #[test]
    fn test_add_point_increases_count() {
        let mut s = new_sdk_driven_shape("jaw_open", "mouth_open");
        sdk_add_point(&mut s, 0.0, 0.0);
        sdk_add_point(&mut s, 1.0, 1.0);
        assert_eq!(sdk_point_count(&s), 2 /* two points should be added */,);
    }

    #[test]
    fn test_evaluate_below_min_returns_first() {
        let mut s = new_sdk_driven_shape("brow", "brow_raise");
        sdk_add_point(&mut s, 0.5, 0.2);
        sdk_add_point(&mut s, 1.0, 1.0);
        let w = sdk_evaluate(&mut s, 0.0);
        assert!((w - 0.2).abs() < 1e-5, /* below range returns first point weight */);
    }

    #[test]
    fn test_evaluate_above_max_returns_last() {
        let mut s = new_sdk_driven_shape("brow", "brow_raise");
        sdk_add_point(&mut s, 0.0, 0.0);
        sdk_add_point(&mut s, 1.0, 0.8);
        let w = sdk_evaluate(&mut s, 2.0);
        assert!((w - 0.8).abs() < 1e-5, /* above range returns last point weight */);
    }

    #[test]
    fn test_evaluate_midpoint_interpolates() {
        let mut s = new_sdk_driven_shape("eye", "eye_wide");
        sdk_add_point(&mut s, 0.0, 0.0);
        sdk_add_point(&mut s, 1.0, 1.0);
        let w = sdk_evaluate(&mut s, 0.5);
        assert!((w - 0.5).abs() < 1e-5, /* midpoint should interpolate linearly */);
    }

    #[test]
    fn test_reset_zeroes_weight() {
        let mut s = new_sdk_driven_shape("lip", "lip_compress");
        sdk_add_point(&mut s, 0.0, 0.0);
        sdk_add_point(&mut s, 1.0, 1.0);
        sdk_evaluate(&mut s, 1.0);
        sdk_reset(&mut s);
        assert!((s.current_weight).abs() < 1e-6, /* reset should zero weight */);
    }

    #[test]
    fn test_empty_evaluate_returns_zero() {
        let mut s = new_sdk_driven_shape("chin", "chin_raise");
        let w = sdk_evaluate(&mut s, 0.5);
        assert!((w).abs() < 1e-6 /* empty curve returns 0 */,);
    }

    #[test]
    fn test_points_sorted_by_driver_value() {
        let mut s = new_sdk_driven_shape("test", "shape");
        sdk_add_point(&mut s, 1.0, 0.8);
        sdk_add_point(&mut s, 0.0, 0.0);
        assert!(s.curve[0].driver_value < s.curve[1].driver_value, /* must be sorted */);
    }

    #[test]
    fn test_to_json_contains_driver() {
        let s = new_sdk_driven_shape("jaw_open", "mouth_shape");
        let j = sdk_to_json(&s);
        assert!(j.contains("jaw_open"), /* JSON must contain driver name */);
    }

    #[test]
    fn test_weight_clamped_to_one() {
        let mut s = new_sdk_driven_shape("test", "shape");
        sdk_add_point(&mut s, 0.0, 2.0);
        assert!(s.curve[0].shape_weight <= 1.0, /* weight should be clamped */);
    }

    #[test]
    fn test_driver_attr_stored() {
        let s = new_sdk_driven_shape("shoulder_rot", "shoulder_shape");
        assert_eq!(
            s.driver_attr,
            "shoulder_rot", /* driver attr must match */
        );
    }
}
