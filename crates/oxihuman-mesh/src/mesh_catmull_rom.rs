//! Catmull-Rom spline fitting through mesh vertices.
//!
//! Generates smooth curves through control points using the Catmull-Rom
//! formulation with configurable alpha (0 = uniform, 0.5 = centripetal,
//! 1 = chordal).

/// Configuration for Catmull-Rom spline.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CatmullRomConfig {
    /// Alpha parameter: 0.0 = uniform, 0.5 = centripetal, 1.0 = chordal.
    pub alpha: f32,
    /// Whether to close the spline into a loop.
    pub closed: bool,
}

/// A Catmull-Rom spline defined by control points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CatmullRomSpline {
    /// Control points of the spline.
    pub control_points: Vec<[f32; 3]>,
    /// Alpha parameter.
    pub alpha: f32,
    /// Whether the spline is closed.
    pub closed: bool,
}

/// Returns the default Catmull-Rom configuration (centripetal, open).
#[allow(dead_code)]
pub fn default_catmull_rom_config() -> CatmullRomConfig {
    CatmullRomConfig {
        alpha: 0.5,
        closed: false,
    }
}

/// Creates a new Catmull-Rom spline from control points.
#[allow(dead_code)]
pub fn new_catmull_rom_spline(
    control_points: Vec<[f32; 3]>,
    cfg: &CatmullRomConfig,
) -> CatmullRomSpline {
    CatmullRomSpline {
        control_points,
        alpha: cfg.alpha,
        closed: cfg.closed,
    }
}

/// Returns the number of segments in the spline.
///
/// For an open spline with N points: max(0, N - 3) segments (needs 4 points
/// per segment with phantom endpoints). We use phantom endpoint clamping so
/// a spline with N >= 2 points has N-1 segments.
#[allow(dead_code)]
pub fn catmull_rom_segment_count(spline: &CatmullRomSpline) -> usize {
    let n = spline.control_points.len();
    if n < 2 {
        return 0;
    }
    if spline.closed {
        n
    } else {
        n - 1
    }
}

/// Returns the number of control points.
#[allow(dead_code)]
pub fn catmull_rom_point_count(spline: &CatmullRomSpline) -> usize {
    spline.control_points.len()
}

/// Fetches the i-th control point with clamping (open) or wrap (closed).
fn get_point(spline: &CatmullRomSpline, i: isize) -> [f32; 3] {
    let n = spline.control_points.len() as isize;
    if n == 0 {
        return [0.0; 3];
    }
    let idx = if spline.closed {
        i.rem_euclid(n) as usize
    } else {
        i.clamp(0, n - 1) as usize
    };
    spline.control_points[idx]
}

/// Knot parameter for centripetal / chordal parameterization.
fn knot_distance(a: [f32; 3], b: [f32; 3], alpha: f32) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    let d2 = dx * dx + dy * dy + dz * dz;
    if alpha == 0.5 {
        d2.sqrt().sqrt() // d^alpha with alpha=0.5
    } else if alpha == 0.0 {
        1.0
    } else {
        d2.powf(alpha * 0.5)
    }
}

/// Evaluates the Catmull-Rom spline at global parameter `t` in [0, segments].
#[allow(dead_code)]
pub fn catmull_rom_evaluate(spline: &CatmullRomSpline, t: f32) -> [f32; 3] {
    let segs = catmull_rom_segment_count(spline);
    if segs == 0 {
        return spline.control_points.first().copied().unwrap_or([0.0; 3]);
    }
    let t = t.clamp(0.0, segs as f32);
    let seg = (t.floor() as usize).min(segs - 1);
    let local_t = t - seg as f32;

    let i = seg as isize;
    let p0 = get_point(spline, i - 1);
    let p1 = get_point(spline, i);
    let p2 = get_point(spline, i + 1);
    let p3 = get_point(spline, i + 2);

    catmull_rom_interp(p0, p1, p2, p3, local_t, spline.alpha)
}

/// Core Catmull-Rom interpolation with Barry-Goldman algorithm.
fn catmull_rom_interp(
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
    p3: [f32; 3],
    t: f32,
    alpha: f32,
) -> [f32; 3] {
    if alpha == 0.0 {
        // Uniform: classical cubic formula
        let t2 = t * t;
        let t3 = t2 * t;
        let mut out = [0.0f32; 3];
        for k in 0..3 {
            out[k] = 0.5
                * ((2.0 * p1[k])
                    + (-p0[k] + p2[k]) * t
                    + (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) * t2
                    + (-p0[k] + 3.0 * p1[k] - 3.0 * p2[k] + p3[k]) * t3);
        }
        return out;
    }

    // Barry-Goldman algorithm for non-uniform parameterization.
    let t0 = 0.0f32;
    let t1 = t0 + knot_distance(p0, p1, alpha);
    let t2 = t1 + knot_distance(p1, p2, alpha);
    let t3 = t2 + knot_distance(p2, p3, alpha);

    // Remap local t into [t1, t2]
    let tg = t1 + t * (t2 - t1);

    let lerp3 = |a: [f32; 3], b: [f32; 3], ta: f32, tb: f32, tc: f32| -> [f32; 3] {
        if (tb - ta).abs() < 1e-10 {
            return a;
        }
        let frac = (tc - ta) / (tb - ta);
        [
            a[0] + frac * (b[0] - a[0]),
            a[1] + frac * (b[1] - a[1]),
            a[2] + frac * (b[2] - a[2]),
        ]
    };

    let a1 = lerp3(p0, p1, t0, t1, tg);
    let a2 = lerp3(p1, p2, t1, t2, tg);
    let a3 = lerp3(p2, p3, t2, t3, tg);
    let b1 = lerp3(a1, a2, t0, t2, tg);
    let b2 = lerp3(a2, a3, t1, t3, tg);
    lerp3(b1, b2, t1, t2, tg)
}

/// Samples the spline at `n_samples` equally-spaced parameter values.
#[allow(dead_code)]
pub fn catmull_rom_sample(spline: &CatmullRomSpline, n_samples: usize) -> Vec<[f32; 3]> {
    let segs = catmull_rom_segment_count(spline);
    if n_samples == 0 || segs == 0 {
        return vec![];
    }
    let total = segs as f32;
    (0..n_samples)
        .map(|i| {
            let t = if n_samples == 1 {
                0.0
            } else {
                total * (i as f32) / (n_samples - 1) as f32
            };
            catmull_rom_evaluate(spline, t)
        })
        .collect()
}

/// Evaluates the tangent (first derivative) at global parameter `t`.
#[allow(dead_code)]
pub fn catmull_rom_tangent(spline: &CatmullRomSpline, t: f32) -> [f32; 3] {
    let eps = 1e-4f32;
    let segs = catmull_rom_segment_count(spline) as f32;
    let t0 = (t - eps).clamp(0.0, segs);
    let t1 = (t + eps).clamp(0.0, segs);
    let p0 = catmull_rom_evaluate(spline, t0);
    let p1 = catmull_rom_evaluate(spline, t1);
    let dt = (t1 - t0).max(1e-10);
    [
        (p1[0] - p0[0]) / dt,
        (p1[1] - p0[1]) / dt,
        (p1[2] - p0[2]) / dt,
    ]
}

/// Approximates the arc length of the full spline using `n_steps` segments.
#[allow(dead_code)]
pub fn catmull_rom_arc_length(spline: &CatmullRomSpline, n_steps: usize) -> f32 {
    let segs = catmull_rom_segment_count(spline);
    if n_steps == 0 || segs == 0 {
        return 0.0;
    }
    let total = segs as f32;
    let mut length = 0.0f32;
    let mut prev = catmull_rom_evaluate(spline, 0.0);
    for step in 1..=n_steps {
        let t = total * step as f32 / n_steps as f32;
        let cur = catmull_rom_evaluate(spline, t);
        let dx = cur[0] - prev[0];
        let dy = cur[1] - prev[1];
        let dz = cur[2] - prev[2];
        length += (dx * dx + dy * dy + dz * dz).sqrt();
        prev = cur;
    }
    length
}

/// Finds the parameter `t` on the spline closest to `point` using `n_steps`
/// uniform samples.
#[allow(dead_code)]
pub fn catmull_rom_closest_t(
    spline: &CatmullRomSpline,
    point: [f32; 3],
    n_steps: usize,
) -> f32 {
    let segs = catmull_rom_segment_count(spline);
    if n_steps == 0 || segs == 0 {
        return 0.0;
    }
    let total = segs as f32;
    let mut best_t = 0.0f32;
    let mut best_dist2 = f32::MAX;
    for step in 0..=n_steps {
        let t = total * step as f32 / n_steps as f32;
        let p = catmull_rom_evaluate(spline, t);
        let dx = p[0] - point[0];
        let dy = p[1] - point[1];
        let dz = p[2] - point[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 < best_dist2 {
            best_dist2 = d2;
            best_t = t;
        }
    }
    best_t
}

/// Sets the alpha parameter on an existing spline.
#[allow(dead_code)]
pub fn catmull_rom_set_alpha(spline: &mut CatmullRomSpline, alpha: f32) {
    spline.alpha = alpha.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_line_spline() -> CatmullRomSpline {
        let cfg = default_catmull_rom_config();
        new_catmull_rom_spline(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            &cfg,
        )
    }

    #[test]
    fn test_segment_count() {
        let s = make_line_spline();
        assert_eq!(catmull_rom_segment_count(&s), 3);
    }

    #[test]
    fn test_point_count() {
        let s = make_line_spline();
        assert_eq!(catmull_rom_point_count(&s), 4);
    }

    #[test]
    fn test_evaluate_endpoints() {
        let s = make_line_spline();
        let p0 = catmull_rom_evaluate(&s, 0.0);
        let p_end = catmull_rom_evaluate(&s, 3.0);
        // Should be at or near first and last control points.
        assert!((p0[0] - 0.0).abs() < 1e-4, "start x={}", p0[0]);
        assert!((p_end[0] - 3.0).abs() < 1e-4, "end x={}", p_end[0]);
    }

    #[test]
    fn test_sample_count() {
        let s = make_line_spline();
        let samples = catmull_rom_sample(&s, 10);
        assert_eq!(samples.len(), 10);
    }

    #[test]
    fn test_arc_length_line() {
        let s = make_line_spline();
        let len = catmull_rom_arc_length(&s, 300);
        // A straight line of 3 units should have arc length ~3.
        assert!((len - 3.0).abs() < 0.05, "arc length={}", len);
    }

    #[test]
    fn test_set_alpha() {
        let cfg = default_catmull_rom_config();
        let mut s = new_catmull_rom_spline(
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]],
            &cfg,
        );
        catmull_rom_set_alpha(&mut s, 1.0);
        assert!((s.alpha - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_closest_t_midpoint() {
        let s = make_line_spline();
        // Midpoint of the line is at x=1.5, t~1.5
        let t = catmull_rom_closest_t(&s, [1.5, 0.0, 0.0], 300);
        assert!((t - 1.5).abs() < 0.15, "closest t={}", t);
    }

    #[test]
    fn test_tangent_direction() {
        let s = make_line_spline();
        let tan = catmull_rom_tangent(&s, 1.5);
        // On a straight line along X the tangent should be ~(1,0,0).
        assert!(tan[0] > 0.9, "tangent x={}", tan[0]);
        assert!(tan[1].abs() < 0.1, "tangent y={}", tan[1]);
    }

    #[test]
    fn test_empty_spline() {
        let cfg = default_catmull_rom_config();
        let s = new_catmull_rom_spline(vec![], &cfg);
        assert_eq!(catmull_rom_segment_count(&s), 0);
        assert_eq!(catmull_rom_sample(&s, 5).len(), 0);
        assert_eq!(catmull_rom_arc_length(&s, 10), 0.0);
    }

    #[test]
    fn test_uniform_alpha() {
        let cfg = CatmullRomConfig { alpha: 0.0, closed: false };
        let s = new_catmull_rom_spline(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
            &cfg,
        );
        let p = catmull_rom_evaluate(&s, 1.5);
        assert!((p[0] - 1.5).abs() < 0.05, "mid x={}", p[0]);
    }
}
