//! Bezier and NURBS curve export module.

#[allow(dead_code)]
#[derive(Clone)]
pub struct BezierCurve {
    pub name: String,
    pub control_points: Vec<[f32; 3]>,
    pub closed: bool,
    pub degree: u32,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct NurbsCurve {
    pub name: String,
    pub control_points: Vec<[f32; 3]>,
    pub weights: Vec<f32>,
    pub knots: Vec<f32>,
    pub degree: u32,
}

#[allow(dead_code)]
pub struct CurveCollection {
    pub name: String,
    pub bezier_curves: Vec<BezierCurve>,
    pub nurbs_curves: Vec<NurbsCurve>,
}

#[allow(dead_code)]
pub fn new_curve_collection(name: &str) -> CurveCollection {
    CurveCollection {
        name: name.to_string(),
        bezier_curves: Vec::new(),
        nurbs_curves: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn add_bezier_curve(coll: &mut CurveCollection, curve: BezierCurve) {
    coll.bezier_curves.push(curve);
}

#[allow(dead_code)]
pub fn add_nurbs_curve(coll: &mut CurveCollection, curve: NurbsCurve) {
    coll.nurbs_curves.push(curve);
}

/// De Casteljau evaluation for any degree Bezier curve.
#[allow(dead_code)]
pub fn evaluate_bezier(curve: &BezierCurve, t: f32) -> [f32; 3] {
    let pts = &curve.control_points;
    if pts.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut work: Vec<[f32; 3]> = pts.clone();
    let n = work.len();
    for r in 1..n {
        for i in 0..(n - r) {
            work[i] = [
                (1.0 - t) * work[i][0] + t * work[i + 1][0],
                (1.0 - t) * work[i][1] + t * work[i + 1][1],
                (1.0 - t) * work[i][2] + t * work[i + 1][2],
            ];
        }
    }
    work[0]
}

/// B-spline (NURBS) evaluation using Cox-de Boor.
#[allow(dead_code)]
pub fn evaluate_nurbs(curve: &NurbsCurve, t: f32) -> [f32; 3] {
    let pts = &curve.control_points;
    let weights = &curve.weights;
    let knots = &curve.knots;
    let degree = curve.degree as usize;
    let n = pts.len();

    if n == 0 || knots.len() < n + degree + 1 {
        return [0.0, 0.0, 0.0];
    }

    // Find knot span
    let t_clamped = t.clamp(knots[degree], knots[n]);

    // Cox-de Boor basis functions
    let mut basis = vec![0.0f32; n];
    for i in 0..n {
        basis[i] = if t_clamped >= knots[i] && t_clamped < knots[i + 1] {
            1.0
        } else {
            0.0
        };
    }
    // Handle endpoint
    if (t_clamped - knots[n]).abs() < f32::EPSILON {
        basis = vec![0.0f32; n];
        basis[n - 1] = 1.0;
    }

    for p in 1..=degree {
        let mut new_basis = vec![0.0f32; n];
        for i in 0..n {
            let denom1 = knots[i + p] - knots[i];
            let denom2 = knots[i + p + 1] - knots[i + 1];
            let left = if denom1.abs() > f32::EPSILON {
                (t_clamped - knots[i]) / denom1 * basis[i]
            } else {
                0.0
            };
            let right = if i + 1 < n && denom2.abs() > f32::EPSILON {
                (knots[i + p + 1] - t_clamped) / denom2 * basis[i + 1]
            } else {
                0.0
            };
            new_basis[i] = left + right;
        }
        basis = new_basis;
    }

    let mut wx = 0.0f32;
    let mut wy = 0.0f32;
    let mut wz = 0.0f32;
    let mut wsum = 0.0f32;
    for i in 0..n {
        let w = if i < weights.len() { weights[i] } else { 1.0 };
        let bw = basis[i] * w;
        wx += pts[i][0] * bw;
        wy += pts[i][1] * bw;
        wz += pts[i][2] * bw;
        wsum += bw;
    }

    if wsum.abs() < f32::EPSILON {
        return pts[0];
    }
    [wx / wsum, wy / wsum, wz / wsum]
}

#[allow(dead_code)]
pub fn sample_bezier(curve: &BezierCurve, steps: u32) -> Vec<[f32; 3]> {
    if steps == 0 {
        return Vec::new();
    }
    let count = steps + 1;
    (0..count)
        .map(|i| {
            let t = i as f32 / steps as f32;
            evaluate_bezier(curve, t)
        })
        .collect()
}

#[allow(dead_code)]
pub fn sample_nurbs(curve: &NurbsCurve, steps: u32) -> Vec<[f32; 3]> {
    if steps == 0 {
        return Vec::new();
    }
    let count = steps + 1;
    (0..count)
        .map(|i| {
            let t = i as f32 / steps as f32;
            evaluate_nurbs(curve, t)
        })
        .collect()
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn bezier_arc_length(curve: &BezierCurve, steps: u32) -> f32 {
    let samples = sample_bezier(curve, steps);
    if samples.len() < 2 {
        return 0.0;
    }
    samples.windows(2).map(|w| dist3(w[0], w[1])).sum()
}

#[allow(dead_code)]
pub fn curve_collection_to_json(coll: &CurveCollection) -> String {
    let mut parts = Vec::new();

    let mut bezier_parts = Vec::new();
    for bc in &coll.bezier_curves {
        let cp_strs: Vec<String> = bc
            .control_points
            .iter()
            .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
            .collect();
        bezier_parts.push(format!(
            r#"{{"name":"{}","degree":{},"closed":{},"control_points":[{}]}}"#,
            bc.name,
            bc.degree,
            bc.closed,
            cp_strs.join(",")
        ));
    }

    let mut nurbs_parts = Vec::new();
    for nc in &coll.nurbs_curves {
        let cp_strs: Vec<String> = nc
            .control_points
            .iter()
            .map(|p| format!("[{},{},{}]", p[0], p[1], p[2]))
            .collect();
        let w_strs: Vec<String> = nc.weights.iter().map(|w| format!("{w}")).collect();
        let k_strs: Vec<String> = nc.knots.iter().map(|k| format!("{k}")).collect();
        nurbs_parts.push(format!(
            r#"{{"name":"{}","degree":{},"control_points":[{}],"weights":[{}],"knots":[{}]}}"#,
            nc.name,
            nc.degree,
            cp_strs.join(","),
            w_strs.join(","),
            k_strs.join(",")
        ));
    }

    parts.push(format!(r#""bezier_curves":[{}]"#, bezier_parts.join(",")));
    parts.push(format!(r#""nurbs_curves":[{}]"#, nurbs_parts.join(",")));

    format!(r#"{{"name":"{}",{}}}"#, coll.name, parts.join(","))
}

#[allow(dead_code)]
pub fn curve_collection_to_svg_paths(coll: &CurveCollection) -> Vec<String> {
    let mut paths = Vec::new();
    for bc in &coll.bezier_curves {
        let pts = &bc.control_points;
        if pts.is_empty() {
            continue;
        }
        let mut path = format!("M {} {}", pts[0][0], pts[0][1]);
        match bc.degree {
            1 => {
                for p in pts.iter().skip(1) {
                    path.push_str(&format!(" L {} {}", p[0], p[1]));
                }
            }
            2 => {
                let mut i = 1;
                while i + 1 < pts.len() {
                    path.push_str(&format!(
                        " Q {} {} {} {}",
                        pts[i][0],
                        pts[i][1],
                        pts[i + 1][0],
                        pts[i + 1][1]
                    ));
                    i += 2;
                }
            }
            _ => {
                let mut i = 1;
                while i + 2 < pts.len() {
                    path.push_str(&format!(
                        " C {} {} {} {} {} {}",
                        pts[i][0],
                        pts[i][1],
                        pts[i + 1][0],
                        pts[i + 1][1],
                        pts[i + 2][0],
                        pts[i + 2][1]
                    ));
                    i += 3;
                }
            }
        }
        if bc.closed {
            path.push_str(" Z");
        }
        paths.push(path);
    }
    paths
}

#[allow(dead_code)]
pub fn bezier_curve_count(coll: &CurveCollection) -> usize {
    coll.bezier_curves.len()
}

#[allow(dead_code)]
pub fn nurbs_curve_count(coll: &CurveCollection) -> usize {
    coll.nurbs_curves.len()
}

/// Clamped uniform knot vector for B-spline.
#[allow(dead_code)]
pub fn nurbs_default_knots(point_count: usize, degree: u32) -> Vec<f32> {
    let d = degree as usize;
    let n_knots = point_count + d + 1;
    let mut knots = Vec::with_capacity(n_knots);

    knots.extend(std::iter::repeat_n(0.0f32, d + 1));
    let inner = n_knots - 2 * (d + 1);
    for i in 1..=inner {
        knots.push(i as f32 / (inner + 1) as f32);
    }
    knots.extend(std::iter::repeat_n(1.0f32, d + 1));
    knots
}

#[allow(dead_code)]
pub fn linear_bezier(a: [f32; 3], b: [f32; 3]) -> BezierCurve {
    BezierCurve {
        name: "linear".to_string(),
        control_points: vec![a, b],
        closed: false,
        degree: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_curve_collection() {
        let coll = new_curve_collection("test");
        assert_eq!(coll.name, "test");
        assert!(coll.bezier_curves.is_empty());
        assert!(coll.nurbs_curves.is_empty());
    }

    #[test]
    fn test_add_bezier_curve() {
        let mut coll = new_curve_collection("c");
        let bc = BezierCurve {
            name: "b1".to_string(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            closed: false,
            degree: 1,
        };
        add_bezier_curve(&mut coll, bc);
        assert_eq!(bezier_curve_count(&coll), 1);
    }

    #[test]
    fn test_add_nurbs_curve() {
        let mut coll = new_curve_collection("c");
        let knots = nurbs_default_knots(3, 2);
        let nc = NurbsCurve {
            name: "n1".to_string(),
            control_points: vec![[0.0, 0.0, 0.0], [0.5, 1.0, 0.0], [1.0, 0.0, 0.0]],
            weights: vec![1.0, 1.0, 1.0],
            knots,
            degree: 2,
        };
        add_nurbs_curve(&mut coll, nc);
        assert_eq!(nurbs_curve_count(&coll), 1);
    }

    #[test]
    fn test_evaluate_bezier_at_t0() {
        let bc = BezierCurve {
            name: "b".to_string(),
            control_points: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
            closed: false,
            degree: 2,
        };
        let p = evaluate_bezier(&bc, 0.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert!((p[1] - 2.0).abs() < 1e-5);
        assert!((p[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_bezier_at_t1() {
        let bc = BezierCurve {
            name: "b".to_string(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]],
            closed: false,
            degree: 2,
        };
        let p = evaluate_bezier(&bc, 1.0);
        assert!((p[0] - 2.0).abs() < 1e-5);
        assert!((p[1] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_sample_bezier_length() {
        let bc = BezierCurve {
            name: "b".to_string(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            closed: false,
            degree: 1,
        };
        let samples = sample_bezier(&bc, 10);
        assert_eq!(samples.len(), 11);
    }

    #[test]
    fn test_bezier_arc_length_nonneg() {
        let bc = BezierCurve {
            name: "b".to_string(),
            control_points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            closed: false,
            degree: 1,
        };
        let len = bezier_arc_length(&bc, 100);
        assert!(len >= 0.0);
        assert!((len - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_curve_collection_to_json() {
        let mut coll = new_curve_collection("shapes");
        let bc = linear_bezier([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        add_bezier_curve(&mut coll, bc);
        let json = curve_collection_to_json(&coll);
        assert!(!json.is_empty());
        assert!(json.contains("shapes"));
        assert!(json.contains("bezier_curves"));
    }

    #[test]
    fn test_nurbs_default_knots_length() {
        let knots = nurbs_default_knots(4, 3);
        assert_eq!(knots.len(), 4 + 3 + 1);
    }

    #[test]
    fn test_nurbs_default_knots_clamped() {
        let knots = nurbs_default_knots(4, 3);
        assert!((knots[0] - 0.0).abs() < 1e-6);
        assert!((knots[knots.len() - 1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_bezier_endpoints() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 2.0, 3.0];
        let bc = linear_bezier(a, b);
        let p0 = evaluate_bezier(&bc, 0.0);
        let p1 = evaluate_bezier(&bc, 1.0);
        assert!((p0[0] - a[0]).abs() < 1e-5);
        assert!((p1[0] - b[0]).abs() < 1e-5);
        assert!((p1[2] - b[2]).abs() < 1e-5);
    }

    #[test]
    fn test_bezier_curve_count() {
        let mut coll = new_curve_collection("c");
        assert_eq!(bezier_curve_count(&coll), 0);
        add_bezier_curve(&mut coll, linear_bezier([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]));
        assert_eq!(bezier_curve_count(&coll), 1);
    }

    #[test]
    fn test_nurbs_curve_count() {
        let coll = new_curve_collection("c");
        assert_eq!(nurbs_curve_count(&coll), 0);
    }

    #[test]
    fn test_svg_paths_nonempty() {
        let mut coll = new_curve_collection("s");
        let bc = linear_bezier([0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        add_bezier_curve(&mut coll, bc);
        let paths = curve_collection_to_svg_paths(&coll);
        assert!(!paths.is_empty());
        assert!(paths[0].starts_with('M'));
    }

    #[test]
    fn test_sample_bezier_zero_steps() {
        let bc = linear_bezier([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        let samples = sample_bezier(&bc, 0);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_evaluate_bezier_empty() {
        let bc = BezierCurve {
            name: "empty".to_string(),
            control_points: Vec::new(),
            closed: false,
            degree: 1,
        };
        let p = evaluate_bezier(&bc, 0.5);
        assert_eq!(p, [0.0, 0.0, 0.0]);
    }
}
