//! NURBS and Bezier curve export utilities.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Curve representation type.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CurveType {
    Bezier,
    BSpline,
    Nurbs,
    Polyline,
}

/// Configuration for curve export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveExportConfig {
    pub curve_type: CurveType,
    pub degree: u32,
    pub include_knots: bool,
    pub precision: u8,
}

/// A single exportable curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExportCurve {
    pub control_points: Vec<[f32; 3]>,
    pub weights: Vec<f32>,
    pub knots: Vec<f32>,
    pub degree: u32,
}

/// Aggregated result of exporting a collection of curves.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvesExportResult {
    pub curves: Vec<String>,
    pub total_control_points: usize,
}

// ── Functions ──────────────────────────────────────────────────────────────────

/// Return a default [`CurveExportConfig`] for cubic Bezier curves.
#[allow(dead_code)]
pub fn default_curves_export_config() -> CurveExportConfig {
    CurveExportConfig {
        curve_type: CurveType::Bezier,
        degree: 3,
        include_knots: false,
        precision: 6,
    }
}

/// Construct a new [`ExportCurve`] with the given control points and degree.
/// Weights default to 1.0; knots are empty until populated.
#[allow(dead_code)]
pub fn new_export_curve(cps: Vec<[f32; 3]>, degree: u32) -> ExportCurve {
    let n = cps.len();
    ExportCurve {
        control_points: cps,
        weights: vec![1.0; n],
        knots: Vec::new(),
        degree,
    }
}

/// Populate uniform knot vector for a clamped B-spline.
/// Knot count = n + degree + 1, where n = number of control points.
#[allow(dead_code)]
pub fn add_uniform_knots(curve: &mut ExportCurve) {
    let n = curve.control_points.len();
    let d = curve.degree as usize;
    let total = n + d + 1;
    let inner = total.saturating_sub(2 * (d + 1));
    let mut knots = Vec::with_capacity(total);
    // d+1 leading zeros
    knots.extend(std::iter::repeat_n(0.0_f32, d + 1));
    // uniform interior
    for i in 1..=inner {
        knots.push(i as f32 / (inner + 1) as f32);
    }
    // d+1 trailing ones
    while knots.len() < total {
        knots.push(1.0_f32);
    }
    curve.knots = knots;
}

/// Serialise a single curve to a JSON string.
#[allow(dead_code)]
pub fn curve_to_json(curve: &ExportCurve, cfg: &CurveExportConfig) -> String {
    let pts: Vec<String> = curve
        .control_points
        .iter()
        .map(|p| format!("[{:.prec$},{:.prec$},{:.prec$}]", p[0], p[1], p[2], prec = cfg.precision as usize))
        .collect();
    let knots_str = if cfg.include_knots {
        let ks: Vec<String> = curve.knots.iter().map(|k| format!("{:.prec$}", k, prec = cfg.precision as usize)).collect();
        format!(r#","knots":[{}]"#, ks.join(","))
    } else {
        String::new()
    };
    format!(
        r#"{{"type":"{}","degree":{},"control_points":[{}]{}}}   "#,
        curve_type_name(cfg),
        curve.degree,
        pts.join(","),
        knots_str
    )
    .trim()
    .to_string()
}

/// Export a slice of curves using the given config.
#[allow(dead_code)]
pub fn export_curves(curves: &[ExportCurve], cfg: &CurveExportConfig) -> CurvesExportResult {
    let json_curves: Vec<String> = curves.iter().map(|c| curve_to_json(c, cfg)).collect();
    let total_control_points = curves.iter().map(|c| c.control_points.len()).sum();
    CurvesExportResult {
        curves: json_curves,
        total_control_points,
    }
}

/// Evaluate a Bezier curve at parameter `t` using de Casteljau's algorithm.
#[allow(dead_code)]
pub fn bezier_point_at(curve: &ExportCurve, t: f32) -> [f32; 3] {
    if curve.control_points.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let t = t.clamp(0.0, 1.0);
    let mut pts = curve.control_points.clone();
    let n = pts.len();
    for r in 1..n {
        for i in 0..(n - r) {
            pts[i] = [
                (1.0 - t) * pts[i][0] + t * pts[i + 1][0],
                (1.0 - t) * pts[i][1] + t * pts[i + 1][1],
                (1.0 - t) * pts[i][2] + t * pts[i + 1][2],
            ];
        }
    }
    pts[0]
}

/// Return the number of control points in a curve.
#[allow(dead_code)]
pub fn control_point_count(curve: &ExportCurve) -> usize {
    curve.control_points.len()
}

/// Return the human-readable name of the curve type in the config.
#[allow(dead_code)]
pub fn curve_type_name(cfg: &CurveExportConfig) -> &'static str {
    match cfg.curve_type {
        CurveType::Bezier => "bezier",
        CurveType::BSpline => "bspline",
        CurveType::Nurbs => "nurbs",
        CurveType::Polyline => "polyline",
    }
}

/// Approximate arc length by sampling `samples` equally-spaced points along
/// a Bezier curve and summing chord lengths.
#[allow(dead_code)]
pub fn curve_arc_length_approx(curve: &ExportCurve, samples: usize) -> f32 {
    if samples < 2 || curve.control_points.is_empty() {
        return 0.0;
    }
    let mut len = 0.0_f32;
    let mut prev = bezier_point_at(curve, 0.0);
    for i in 1..=samples {
        let t = i as f32 / samples as f32;
        let p = bezier_point_at(curve, t);
        let dx = p[0] - prev[0];
        let dy = p[1] - prev[1];
        let dz = p[2] - prev[2];
        len += (dx * dx + dy * dy + dz * dz).sqrt();
        prev = p;
    }
    len
}

/// Return `true` if the curve has at least `degree + 1` control points.
#[allow(dead_code)]
pub fn validate_curve(curve: &ExportCurve) -> bool {
    curve.control_points.len() > curve.degree as usize
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn line_curve() -> ExportCurve {
        new_export_curve(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            3,
        )
    }

    #[test]
    fn default_config_is_bezier_cubic() {
        let cfg = default_curves_export_config();
        assert_eq!(cfg.curve_type, CurveType::Bezier);
        assert_eq!(cfg.degree, 3);
    }

    #[test]
    fn new_export_curve_weights_default_one() {
        let c = new_export_curve(vec![[0.0, 0.0, 0.0]; 4], 3);
        assert!(c.weights.iter().all(|&w| (w - 1.0).abs() < 1e-6));
    }

    #[test]
    fn add_uniform_knots_correct_count() {
        let mut c = line_curve();
        add_uniform_knots(&mut c);
        // n=4, d=3 → total = 4 + 3 + 1 = 8
        assert_eq!(c.knots.len(), 8);
    }

    #[test]
    fn bezier_at_endpoints() {
        let c = line_curve();
        let p0 = bezier_point_at(&c, 0.0);
        let p1 = bezier_point_at(&c, 1.0);
        assert!((p0[0] - 0.0).abs() < 1e-4);
        assert!((p1[0] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn curve_arc_length_positive() {
        let c = line_curve();
        let len = curve_arc_length_approx(&c, 100);
        assert!(len > 0.0);
    }

    #[test]
    fn validate_curve_passes_valid() {
        let c = line_curve();
        assert!(validate_curve(&c));
    }

    #[test]
    fn validate_curve_fails_too_few_points() {
        let c = new_export_curve(vec![[0.0, 0.0, 0.0]], 3);
        assert!(!validate_curve(&c));
    }

    #[test]
    fn export_curves_counts_control_points() {
        let cfg = default_curves_export_config();
        let curves = vec![line_curve(), line_curve()];
        let result = export_curves(&curves, &cfg);
        assert_eq!(result.total_control_points, 8);
        assert_eq!(result.curves.len(), 2);
    }
}
