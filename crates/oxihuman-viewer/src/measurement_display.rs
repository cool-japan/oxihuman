// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Body measurement display overlay.

#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum MeasurementUnit {
    Centimeters,
    Inches,
    Meters,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct BodyMeasurement {
    pub name: String,
    pub value_m: f32, // stored in meters
    pub unit: MeasurementUnit,
    pub points: Vec<[f32; 3]>, // world-space measurement points
    pub visible: bool,
}

#[allow(dead_code)]
pub struct MeasurementDisplay {
    pub measurements: Vec<BodyMeasurement>,
    pub show_labels: bool,
    pub show_lines: bool,
    pub color: [f32; 4],
    pub unit: MeasurementUnit,
}

#[allow(dead_code)]
pub fn new_measurement_display() -> MeasurementDisplay {
    MeasurementDisplay {
        measurements: Vec::new(),
        show_labels: true,
        show_lines: true,
        color: [1.0, 1.0, 0.0, 1.0],
        unit: MeasurementUnit::Centimeters,
    }
}

fn segment_length(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute value_m as sum of segment lengths from consecutive points.
#[allow(dead_code)]
pub fn add_measurement(display: &mut MeasurementDisplay, name: &str, points: Vec<[f32; 3]>) {
    let value_m = if points.len() < 2 {
        0.0
    } else {
        points.windows(2).map(|w| segment_length(w[0], w[1])).sum()
    };
    let unit = display.unit.clone();
    display.measurements.push(BodyMeasurement {
        name: name.to_string(),
        value_m,
        unit,
        points,
        visible: true,
    });
}

#[allow(dead_code)]
pub fn display_value(m: &BodyMeasurement) -> f32 {
    convert_to_unit(m.value_m, &m.unit)
}

#[allow(dead_code)]
pub fn unit_suffix(unit: &MeasurementUnit) -> &'static str {
    match unit {
        MeasurementUnit::Centimeters => "cm",
        MeasurementUnit::Inches => "in",
        MeasurementUnit::Meters => "m",
    }
}

#[allow(dead_code)]
pub fn convert_to_unit(meters: f32, unit: &MeasurementUnit) -> f32 {
    match unit {
        MeasurementUnit::Centimeters => meters * 100.0,
        MeasurementUnit::Inches => meters * 39.3701,
        MeasurementUnit::Meters => meters,
    }
}

#[allow(dead_code)]
pub fn measurement_count(display: &MeasurementDisplay) -> usize {
    display.measurements.len()
}

#[allow(dead_code)]
pub fn visible_measurements(display: &MeasurementDisplay) -> Vec<&BodyMeasurement> {
    display.measurements.iter().filter(|m| m.visible).collect()
}

#[allow(dead_code)]
pub fn get_measurement_by_name<'a>(
    display: &'a MeasurementDisplay,
    name: &str,
) -> Option<&'a BodyMeasurement> {
    display.measurements.iter().find(|m| m.name == name)
}

#[allow(dead_code)]
pub fn set_unit(display: &mut MeasurementDisplay, unit: MeasurementUnit) {
    display.unit = unit.clone();
    for m in display.measurements.iter_mut() {
        m.unit = unit.clone();
    }
}

#[allow(dead_code)]
pub fn measurement_to_string(m: &BodyMeasurement) -> String {
    let val = display_value(m);
    let suffix = unit_suffix(&m.unit);
    format!("{}: {:.1} {}", m.name, val, suffix)
}

/// Returns approximate proportional body measurements for a given height in meters.
/// Proportions roughly follow average human body ratios.
#[allow(dead_code)]
pub fn standard_body_measurements(height_m: f32) -> Vec<(String, f32)> {
    vec![
        ("Height".to_string(), height_m),
        ("Shoulder Width".to_string(), height_m * 0.259),
        ("Chest Circumference".to_string(), height_m * 0.53),
        ("Waist Circumference".to_string(), height_m * 0.43),
        ("Hip Circumference".to_string(), height_m * 0.54),
        ("Arm Length".to_string(), height_m * 0.33),
        ("Leg Length (inseam)".to_string(), height_m * 0.47),
        ("Head Circumference".to_string(), height_m * 0.35),
    ]
}

/// Returns pairs of consecutive points defining line segments.
#[allow(dead_code)]
pub fn measurement_line_segments(m: &BodyMeasurement) -> Vec<([f32; 3], [f32; 3])> {
    if m.points.len() < 2 {
        return Vec::new();
    }
    m.points.windows(2).map(|w| (w[0], w[1])).collect()
}

/// Returns (min, max) display values across all measurements.
#[allow(dead_code)]
pub fn total_display_range(display: &MeasurementDisplay) -> (f32, f32) {
    if display.measurements.is_empty() {
        return (0.0, 0.0);
    }
    let values: Vec<f32> = display.measurements.iter().map(display_value).collect();
    let mn = values.iter().cloned().fold(f32::MAX, f32::min);
    let mx = values.iter().cloned().fold(f32::MIN, f32::max);
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_measurement_display() {
        let d = new_measurement_display();
        assert_eq!(measurement_count(&d), 0);
        assert!(d.show_labels);
        assert!(d.show_lines);
    }

    #[test]
    fn test_add_measurement_single_segment() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Height", vec![[0.0, 0.0, 0.0], [0.0, 1.75, 0.0]]);
        assert_eq!(measurement_count(&d), 1);
        let m = &d.measurements[0];
        assert!((m.value_m - 1.75).abs() < 1e-4);
    }

    #[test]
    fn test_add_measurement_multi_segment() {
        let mut d = new_measurement_display();
        // L-shaped path: (0,0,0) -> (1,0,0) -> (1,1,0) total length = 2.0
        add_measurement(
            &mut d,
            "Path",
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
        );
        let m = &d.measurements[0];
        assert!(
            (m.value_m - 2.0).abs() < 1e-4,
            "multi-segment length should be 2.0"
        );
    }

    #[test]
    fn test_display_value_centimeters() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "X", vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        let m = &d.measurements[0];
        let val = display_value(m);
        assert!((val - 100.0).abs() < 1e-3, "1m in cm = 100");
    }

    #[test]
    fn test_unit_suffix() {
        assert_eq!(unit_suffix(&MeasurementUnit::Centimeters), "cm");
        assert_eq!(unit_suffix(&MeasurementUnit::Inches), "in");
        assert_eq!(unit_suffix(&MeasurementUnit::Meters), "m");
    }

    #[test]
    fn test_convert_to_unit() {
        assert!((convert_to_unit(1.0, &MeasurementUnit::Centimeters) - 100.0).abs() < 1e-3);
        assert!((convert_to_unit(1.0, &MeasurementUnit::Meters) - 1.0).abs() < 1e-6);
        // 1 meter ~ 39.37 inches
        assert!((convert_to_unit(1.0, &MeasurementUnit::Inches) - 39.3701).abs() < 1e-2);
    }

    #[test]
    fn test_measurement_count() {
        let mut d = new_measurement_display();
        assert_eq!(measurement_count(&d), 0);
        add_measurement(&mut d, "A", vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert_eq!(measurement_count(&d), 1);
        add_measurement(&mut d, "B", vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(measurement_count(&d), 2);
    }

    #[test]
    fn test_visible_measurements() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Visible", vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        add_measurement(&mut d, "Hidden", vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        d.measurements[1].visible = false;
        let vis = visible_measurements(&d);
        assert_eq!(vis.len(), 1);
        assert_eq!(vis[0].name, "Visible");
    }

    #[test]
    fn test_get_measurement_by_name() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Waist", vec![[0.0, 0.0, 0.0], [0.9, 0.0, 0.0]]);
        let found = get_measurement_by_name(&d, "Waist");
        assert!(found.is_some());
        let not_found = get_measurement_by_name(&d, "Nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_set_unit_updates_all() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Height", vec![[0.0, 0.0, 0.0], [0.0, 1.75, 0.0]]);
        set_unit(&mut d, MeasurementUnit::Inches);
        assert_eq!(d.unit, MeasurementUnit::Inches);
        assert_eq!(d.measurements[0].unit, MeasurementUnit::Inches);
    }

    #[test]
    fn test_measurement_to_string() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Height", vec![[0.0, 0.0, 0.0], [0.0, 1.75, 0.0]]);
        let s = measurement_to_string(&d.measurements[0]);
        assert!(s.contains("Height"), "string should contain name");
        assert!(s.contains("cm"), "default unit should be cm");
    }

    #[test]
    fn test_measurement_to_string_inches() {
        let mut d = new_measurement_display();
        set_unit(&mut d, MeasurementUnit::Inches);
        add_measurement(&mut d, "Arm", vec![[0.0, 0.0, 0.0], [0.6, 0.0, 0.0]]);
        let s = measurement_to_string(&d.measurements[0]);
        assert!(s.contains("in"), "string should contain 'in' suffix");
    }

    #[test]
    fn test_measurement_line_segments_empty() {
        let m = BodyMeasurement {
            name: "Empty".to_string(),
            value_m: 0.0,
            unit: MeasurementUnit::Meters,
            points: vec![],
            visible: true,
        };
        assert!(measurement_line_segments(&m).is_empty());
    }

    #[test]
    fn test_measurement_line_segments_two_points() {
        let m = BodyMeasurement {
            name: "Seg".to_string(),
            value_m: 1.0,
            unit: MeasurementUnit::Meters,
            points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            visible: true,
        };
        let segs = measurement_line_segments(&m);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].0, [0.0, 0.0, 0.0]);
        assert_eq!(segs[0].1, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_total_display_range() {
        let mut d = new_measurement_display();
        add_measurement(&mut d, "Short", vec![[0.0, 0.0, 0.0], [0.5, 0.0, 0.0]]);
        add_measurement(&mut d, "Long", vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let (mn, mx) = total_display_range(&d);
        // In centimeters: 50.0 and 200.0
        assert!((mn - 50.0).abs() < 1e-2, "min should be 50 cm");
        assert!((mx - 200.0).abs() < 1e-2, "max should be 200 cm");
    }

    #[test]
    fn test_standard_body_measurements() {
        let measurements = standard_body_measurements(1.75);
        assert!(!measurements.is_empty(), "should return some measurements");
        let height = measurements.iter().find(|(n, _)| n == "Height");
        assert!(height.is_some());
        let (_, h) = height.unwrap();
        assert!((h - 1.75).abs() < 1e-4);
    }
}
