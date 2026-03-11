//! 3D measurement/ruler tool.
//!
//! Computes distances, angles, and areas between picked points in 3D space.

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for the measurement tool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeasurementConfig {
    /// Unit label appended to distance results (e.g. `"m"`, `"cm"`).
    pub unit_label: String,
    /// Number of decimal places shown in formatted output.
    pub decimal_places: u8,
    /// Whether to snap measurements to the nearest grid unit.
    pub snap_to_grid: bool,
    /// Grid cell size used when `snap_to_grid` is enabled.
    pub grid_size: f32,
}

// ── Point ─────────────────────────────────────────────────────────────────────

/// A single picked point in 3D space.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeasurementPoint {
    /// World-space position.
    pub position: [f32; 3],
    /// Optional label for this point.
    pub label: String,
}

// ── Result ────────────────────────────────────────────────────────────────────

/// The result of a single measurement operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeasurementResult {
    /// Computed scalar value (distance in world units, angle in degrees, or area).
    pub value: f32,
    /// Human-readable description.
    pub description: String,
}

// ── Session ───────────────────────────────────────────────────────────────────

/// An active measurement session that accumulates picked points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeasurementSession {
    /// Session configuration.
    pub config: MeasurementConfig,
    /// Picked points in order of addition.
    pub points: Vec<MeasurementPoint>,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Returns sensible default `MeasurementConfig`.
#[allow(dead_code)]
pub fn default_measurement_config() -> MeasurementConfig {
    MeasurementConfig {
        unit_label: "m".to_string(),
        decimal_places: 4,
        snap_to_grid: false,
        grid_size: 0.1,
    }
}

/// Creates a new, empty `MeasurementSession`.
#[allow(dead_code)]
pub fn new_measurement_session(cfg: &MeasurementConfig) -> MeasurementSession {
    MeasurementSession {
        config: cfg.clone(),
        points: Vec::new(),
    }
}

/// Appends a picked point to the session.
#[allow(dead_code)]
pub fn measure_add_point(session: &mut MeasurementSession, point: [f32; 3]) {
    let label = format!("P{}", session.points.len() + 1);
    session.points.push(MeasurementPoint {
        position: point,
        label,
    });
}

/// Returns the Euclidean distance between two 3D points.
#[allow(dead_code)]
pub fn measure_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Returns the angle at `vertex` formed by rays towards `a` and `b`, in degrees.
#[allow(dead_code)]
pub fn measure_angle_deg(a: [f32; 3], vertex: [f32; 3], b: [f32; 3]) -> f32 {
    let va = [a[0] - vertex[0], a[1] - vertex[1], a[2] - vertex[2]];
    let vb = [b[0] - vertex[0], b[1] - vertex[1], b[2] - vertex[2]];
    let dot = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let len_a = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let len_b = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
    if len_a < 1e-9 || len_b < 1e-9 {
        return 0.0;
    }
    let cos_theta = (dot / (len_a * len_b)).clamp(-1.0, 1.0);
    cos_theta.acos().to_degrees()
}

/// Returns the area of the triangle formed by three 3D points.
#[allow(dead_code)]
pub fn measure_triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    // Cross product.
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let mag = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    mag * 0.5
}

/// Returns the number of picked points in the session.
#[allow(dead_code)]
pub fn measurement_point_count(session: &MeasurementSession) -> usize {
    session.points.len()
}

/// Returns the total path length through all picked points in order.
#[allow(dead_code)]
pub fn measurement_total_length(session: &MeasurementSession) -> f32 {
    session
        .points
        .windows(2)
        .map(|w| measure_distance(w[0].position, w[1].position))
        .sum()
}

/// Clears all picked points from the session.
#[allow(dead_code)]
pub fn measurement_clear(session: &mut MeasurementSession) {
    session.points.clear();
}

/// Formats session data as a human-readable string.
#[allow(dead_code)]
pub fn measurement_to_string(session: &MeasurementSession) -> String {
    let prec = session.config.decimal_places as usize;
    let mut out = format!(
        "MeasurementSession [{} points, unit={}]\n",
        session.points.len(),
        session.config.unit_label,
    );
    for (i, pt) in session.points.iter().enumerate() {
        out.push_str(&format!(
            "  {}: ({:.prec$}, {:.prec$}, {:.prec$}) [{}]\n",
            i,
            pt.position[0],
            pt.position[1],
            pt.position[2],
            pt.label,
        ));
    }
    let total = measurement_total_length(session);
    out.push_str(&format!(
        "  total_length = {:.prec$} {}\n",
        total,
        session.config.unit_label,
    ));
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_measurement_config();
        assert_eq!(cfg.unit_label, "m");
        assert!(!cfg.snap_to_grid);
    }

    #[test]
    fn test_new_session_empty() {
        let cfg = default_measurement_config();
        let session = new_measurement_session(&cfg);
        assert_eq!(measurement_point_count(&session), 0);
    }

    #[test]
    fn test_add_point() {
        let cfg = default_measurement_config();
        let mut session = new_measurement_session(&cfg);
        measure_add_point(&mut session, [1.0, 0.0, 0.0]);
        assert_eq!(measurement_point_count(&session), 1);
    }

    #[test]
    fn test_measure_distance_unit() {
        let d = measure_distance([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_measure_distance_3d() {
        let d = measure_distance([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((d - 3.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_measure_angle_right() {
        // 90 degrees at origin between X and Y axes.
        let angle = measure_angle_deg([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - 90.0).abs() < 1e-4);
    }

    #[test]
    fn test_measure_triangle_area_unit() {
        // Right triangle in XY plane with legs of length 1.
        let area = measure_triangle_area(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!((area - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_total_length() {
        let cfg = default_measurement_config();
        let mut session = new_measurement_session(&cfg);
        measure_add_point(&mut session, [0.0, 0.0, 0.0]);
        measure_add_point(&mut session, [1.0, 0.0, 0.0]);
        measure_add_point(&mut session, [1.0, 1.0, 0.0]);
        let total = measurement_total_length(&session);
        assert!((total - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_measurement_clear() {
        let cfg = default_measurement_config();
        let mut session = new_measurement_session(&cfg);
        measure_add_point(&mut session, [1.0, 2.0, 3.0]);
        measurement_clear(&mut session);
        assert_eq!(measurement_point_count(&session), 0);
    }

    #[test]
    fn test_measurement_to_string() {
        let cfg = default_measurement_config();
        let mut session = new_measurement_session(&cfg);
        measure_add_point(&mut session, [0.0, 0.0, 0.0]);
        let s = measurement_to_string(&session);
        assert!(s.contains("MeasurementSession"));
        assert!(s.contains("total_length"));
    }
}
