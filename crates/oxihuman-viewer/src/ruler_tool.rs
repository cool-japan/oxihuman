// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Measurement ruler overlay for distance/angle measurement in viewport.

/// Ruler measurement mode.
#[allow(dead_code)]
#[derive(Clone, PartialEq, Debug)]
pub enum RulerMode {
    Distance,
    Angle,
    Circumference,
}

/// A single point in 3D space used by the ruler.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RulerPoint {
    pub position: [f32; 3],
    pub label: String,
}

/// A completed measurement result.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RulerMeasurement {
    pub mode: RulerMode,
    pub value: f32,
    pub unit: String,
}

/// Ruler tool state.
#[allow(dead_code)]
pub struct RulerTool {
    pub points: Vec<RulerPoint>,
    pub mode: RulerMode,
    pub scale: f32,
    pub unit: String,
}

/// Type alias for measurement text result.
#[allow(dead_code)]
pub type MeasurementText = String;

// ── Public API ────────────────────────────────────────────────────────────────

/// Create a new ruler tool in distance mode.
#[allow(dead_code)]
pub fn new_ruler_tool() -> RulerTool {
    RulerTool {
        points: Vec::new(),
        mode: RulerMode::Distance,
        scale: 1.0,
        unit: "m".to_string(),
    }
}

/// Add a measurement point.
#[allow(dead_code)]
pub fn add_ruler_point(ruler: &mut RulerTool, position: [f32; 3], label: &str) {
    ruler.points.push(RulerPoint {
        position,
        label: label.to_string(),
    });
}

/// Clear all points from the ruler.
#[allow(dead_code)]
pub fn clear_ruler(ruler: &mut RulerTool) {
    ruler.points.clear();
}

/// Compute the Euclidean distance between two 3D points.
#[allow(dead_code)]
pub fn compute_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute the angle (in degrees) between vectors `ba` and `bc` at vertex `b`.
/// Requires three points: `a`, `b` (vertex), `c`.
#[allow(dead_code)]
pub fn compute_angle_deg(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ba = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
    let dot = ba[0] * bc[0] + ba[1] * bc[1] + ba[2] * bc[2];
    let len_ba = (ba[0] * ba[0] + ba[1] * ba[1] + ba[2] * ba[2]).sqrt();
    let len_bc = (bc[0] * bc[0] + bc[1] * bc[1] + bc[2] * bc[2]).sqrt();
    let denom = len_ba * len_bc;
    if denom < 1e-12 {
        return 0.0;
    }
    let cos_angle = (dot / denom).clamp(-1.0, 1.0);
    cos_angle.acos().to_degrees()
}

/// Compute the circumference (perimeter) of the polyline formed by all ruler points.
/// If there are fewer than 2 points, returns 0.
#[allow(dead_code)]
pub fn compute_circumference(ruler: &RulerTool) -> f32 {
    if ruler.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 0..ruler.points.len() {
        let j = (i + 1) % ruler.points.len();
        total += compute_distance(ruler.points[i].position, ruler.points[j].position);
    }
    total * ruler.scale
}

/// Number of points currently stored in the ruler.
#[allow(dead_code)]
pub fn ruler_point_count(ruler: &RulerTool) -> usize {
    ruler.points.len()
}

/// Generate human-readable measurement text based on current mode and points.
#[allow(dead_code)]
pub fn ruler_measurement_text(ruler: &RulerTool) -> MeasurementText {
    match ruler.mode {
        RulerMode::Distance => {
            if ruler.points.len() < 2 {
                return "Add at least 2 points".to_string();
            }
            let d = compute_distance(
                ruler.points[0].position,
                ruler.points[ruler.points.len() - 1].position,
            ) * ruler.scale;
            format!("{:.4} {}", d, ruler.unit)
        }
        RulerMode::Angle => {
            if ruler.points.len() < 3 {
                return "Add at least 3 points".to_string();
            }
            let a = compute_angle_deg(
                ruler.points[0].position,
                ruler.points[1].position,
                ruler.points[2].position,
            );
            format!("{:.2} deg", a)
        }
        RulerMode::Circumference => {
            let c = compute_circumference(ruler);
            format!("{:.4} {}", c, ruler.unit)
        }
    }
}

/// Set the ruler measurement mode.
#[allow(dead_code)]
pub fn set_ruler_mode(ruler: &mut RulerTool, mode: RulerMode) {
    ruler.mode = mode;
}

/// Return the unit label string.
#[allow(dead_code)]
pub fn ruler_unit_label(ruler: &RulerTool) -> &str {
    &ruler.unit
}

/// Set the ruler scale factor (e.g. scene units to real-world units).
#[allow(dead_code)]
pub fn set_ruler_scale(ruler: &mut RulerTool, scale: f32) {
    ruler.scale = scale;
}

/// Serialize the ruler state to a JSON string.
#[allow(dead_code)]
pub fn ruler_to_json(ruler: &RulerTool) -> String {
    let mut out = String::from("{\n");
    let mode_str = match &ruler.mode {
        RulerMode::Distance => "distance",
        RulerMode::Angle => "angle",
        RulerMode::Circumference => "circumference",
    };
    out.push_str(&format!("  \"mode\": \"{mode_str}\",\n"));
    out.push_str(&format!("  \"scale\": {:.6},\n", ruler.scale));
    out.push_str(&format!("  \"unit\": \"{}\",\n", ruler.unit));
    out.push_str("  \"points\": [\n");
    for (i, p) in ruler.points.iter().enumerate() {
        let comma = if i + 1 < ruler.points.len() { "," } else { "" };
        out.push_str(&format!(
            "    {{\"label\": \"{}\", \"position\": [{:.6}, {:.6}, {:.6}]}}{comma}\n",
            p.label, p.position[0], p.position[1], p.position[2]
        ));
    }
    out.push_str("  ]\n}");
    out
}

/// Remove the last point added to the ruler.
#[allow(dead_code)]
pub fn remove_last_point(ruler: &mut RulerTool) {
    ruler.points.pop();
}

/// Compute the total polyline length (open, not closed) across all points.
#[allow(dead_code)]
pub fn ruler_total_length(ruler: &RulerTool) -> f32 {
    if ruler.points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    for i in 1..ruler.points.len() {
        total += compute_distance(ruler.points[i - 1].position, ruler.points[i].position);
    }
    total * ruler.scale
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ruler_defaults() {
        let r = new_ruler_tool();
        assert_eq!(r.mode, RulerMode::Distance);
        assert!((r.scale - 1.0).abs() < 1e-6);
        assert_eq!(r.unit, "m");
        assert!(r.points.is_empty());
    }

    #[test]
    fn add_and_count_points() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "B");
        assert_eq!(ruler_point_count(&r), 2);
    }

    #[test]
    fn clear_ruler_empties() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        clear_ruler(&mut r);
        assert_eq!(ruler_point_count(&r), 0);
    }

    #[test]
    fn compute_distance_basic() {
        let d = compute_distance([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]);
        assert!((d - 5.0).abs() < 1e-4);
    }

    #[test]
    fn compute_distance_same_point() {
        let d = compute_distance([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn compute_angle_right_angle() {
        let a = compute_angle_deg([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 90.0).abs() < 1e-3);
    }

    #[test]
    fn compute_angle_straight() {
        let a = compute_angle_deg([-1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((a - 180.0).abs() < 1e-3);
    }

    #[test]
    fn compute_angle_zero_length() {
        let a = compute_angle_deg([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        assert!(a.abs() < 1e-6);
    }

    #[test]
    fn circumference_square() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "B");
        add_ruler_point(&mut r, [1.0, 1.0, 0.0], "C");
        add_ruler_point(&mut r, [0.0, 1.0, 0.0], "D");
        let c = compute_circumference(&r);
        assert!((c - 4.0).abs() < 1e-4);
    }

    #[test]
    fn circumference_too_few_points() {
        let r = new_ruler_tool();
        assert!(compute_circumference(&r).abs() < 1e-6);
    }

    #[test]
    fn measurement_text_distance() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [3.0, 4.0, 0.0], "B");
        let text = ruler_measurement_text(&r);
        assert!(text.contains("5.0000"));
        assert!(text.contains("m"));
    }

    #[test]
    fn measurement_text_angle() {
        let mut r = new_ruler_tool();
        set_ruler_mode(&mut r, RulerMode::Angle);
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "B");
        add_ruler_point(&mut r, [0.0, 1.0, 0.0], "C");
        let text = ruler_measurement_text(&r);
        assert!(text.contains("90.00"));
    }

    #[test]
    fn set_scale_affects_distance() {
        let mut r = new_ruler_tool();
        set_ruler_scale(&mut r, 100.0);
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "B");
        let length = ruler_total_length(&r);
        assert!((length - 100.0).abs() < 1e-4);
    }

    #[test]
    fn ruler_to_json_contains_mode() {
        let r = new_ruler_tool();
        let json = ruler_to_json(&r);
        assert!(json.contains("\"mode\": \"distance\""));
    }

    #[test]
    fn remove_last_point_works() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "B");
        remove_last_point(&mut r);
        assert_eq!(ruler_point_count(&r), 1);
    }

    #[test]
    fn total_length_open_line() {
        let mut r = new_ruler_tool();
        add_ruler_point(&mut r, [0.0, 0.0, 0.0], "A");
        add_ruler_point(&mut r, [1.0, 0.0, 0.0], "B");
        add_ruler_point(&mut r, [2.0, 0.0, 0.0], "C");
        let l = ruler_total_length(&r);
        assert!((l - 2.0).abs() < 1e-4);
    }

    #[test]
    fn unit_label() {
        let r = new_ruler_tool();
        assert_eq!(ruler_unit_label(&r), "m");
    }
}
