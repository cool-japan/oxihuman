// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Generalized winding number for inside/outside mesh queries.

#[allow(dead_code)]
pub struct WindingConfig {
    pub epsilon: f32,
    pub use_fast_approx: bool,
    pub solid_angle_threshold: f32,
}

#[allow(dead_code)]
pub struct WindingQuery {
    pub point: [f32; 3],
    pub winding_number: f32,
    pub is_inside: bool,
}

#[allow(dead_code)]
pub struct WindingResult {
    pub queries: Vec<WindingQuery>,
    pub total_queries: usize,
    pub inside_count: usize,
}

#[allow(dead_code)]
pub fn default_winding_config() -> WindingConfig {
    WindingConfig {
        epsilon: 1e-6,
        use_fast_approx: false,
        solid_angle_threshold: 0.5,
    }
}

#[allow(dead_code)]
pub fn solid_angle_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let pa = [a[0] - p[0], a[1] - p[1], a[2] - p[2]];
    let pb = [b[0] - p[0], b[1] - p[1], b[2] - p[2]];
    let pc = [c[0] - p[0], c[1] - p[1], c[2] - p[2]];

    let pa_len = (pa[0] * pa[0] + pa[1] * pa[1] + pa[2] * pa[2]).sqrt();
    let pb_len = (pb[0] * pb[0] + pb[1] * pb[1] + pb[2] * pb[2]).sqrt();
    let pc_len = (pc[0] * pc[0] + pc[1] * pc[1] + pc[2] * pc[2]).sqrt();

    if pa_len < 1e-12 || pb_len < 1e-12 || pc_len < 1e-12 {
        return 0.0;
    }

    let pa_n = [pa[0] / pa_len, pa[1] / pa_len, pa[2] / pa_len];
    let pb_n = [pb[0] / pb_len, pb[1] / pb_len, pb[2] / pb_len];
    let pc_n = [pc[0] / pc_len, pc[1] / pc_len, pc[2] / pc_len];

    // Numerator: scalar triple product of pa_n, pb_n, pc_n
    let numerator = pa_n[0] * (pb_n[1] * pc_n[2] - pb_n[2] * pc_n[1])
        - pa_n[1] * (pb_n[0] * pc_n[2] - pb_n[2] * pc_n[0])
        + pa_n[2] * (pb_n[0] * pc_n[1] - pb_n[1] * pc_n[0]);

    let dot_ab = pa_n[0] * pb_n[0] + pa_n[1] * pb_n[1] + pa_n[2] * pb_n[2];
    let dot_ac = pa_n[0] * pc_n[0] + pa_n[1] * pc_n[1] + pa_n[2] * pc_n[2];
    let dot_bc = pb_n[0] * pc_n[0] + pb_n[1] * pc_n[1] + pb_n[2] * pc_n[2];

    let denominator = 1.0 + dot_ab + dot_ac + dot_bc;
    2.0 * numerator.atan2(denominator)
}

#[allow(dead_code)]
pub fn compute_winding_number(
    point: [f32; 3],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &WindingConfig,
) -> WindingQuery {
    let mut winding_sum = 0.0_f32;
    for tri in triangles {
        let ia = tri[0] as usize;
        let ib = tri[1] as usize;
        let ic = tri[2] as usize;
        if ia < positions.len() && ib < positions.len() && ic < positions.len() {
            let sa = solid_angle_triangle(point, positions[ia], positions[ib], positions[ic]);
            winding_sum += sa;
        }
    }
    let winding_number = winding_sum / (4.0 * std::f32::consts::PI);
    let is_inside = winding_classify(winding_number, cfg.solid_angle_threshold);
    WindingQuery {
        point,
        winding_number,
        is_inside,
    }
}

#[allow(dead_code)]
pub fn batch_winding(
    points: &[[f32; 3]],
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &WindingConfig,
) -> WindingResult {
    let queries: Vec<WindingQuery> = points
        .iter()
        .map(|&pt| compute_winding_number(pt, positions, triangles, cfg))
        .collect();
    let inside_count = queries.iter().filter(|q| q.is_inside).count();
    let total_queries = queries.len();
    WindingResult {
        queries,
        total_queries,
        inside_count,
    }
}

#[allow(dead_code)]
pub fn is_inside_mesh(wq: &WindingQuery) -> bool {
    wq.is_inside
}

#[allow(dead_code)]
pub fn winding_result_inside_ratio(wr: &WindingResult) -> f32 {
    if wr.total_queries == 0 {
        return 0.0;
    }
    wr.inside_count as f32 / wr.total_queries as f32
}

#[allow(dead_code)]
pub fn winding_query_to_json(wq: &WindingQuery) -> String {
    format!(
        r#"{{"point":[{},{},{}],"winding_number":{},"is_inside":{}}}"#,
        wq.point[0], wq.point[1], wq.point[2], wq.winding_number, wq.is_inside
    )
}

#[allow(dead_code)]
pub fn winding_result_to_json(wr: &WindingResult) -> String {
    format!(
        r#"{{"total_queries":{},"inside_count":{}}}"#,
        wr.total_queries, wr.inside_count
    )
}

#[allow(dead_code)]
pub fn winding_classify(wn: f32, threshold: f32) -> bool {
    wn.abs() > threshold
}

#[allow(dead_code)]
pub fn inside_points(wr: &WindingResult) -> Vec<[f32; 3]> {
    wr.queries
        .iter()
        .filter(|q| q.is_inside)
        .map(|q| q.point)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tetrahedron_positions() -> Vec<[f32; 3]> {
        vec![
            [1.0, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ]
    }

    fn unit_tetrahedron_triangles() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_winding_config();
        assert!(cfg.epsilon > 0.0);
        assert!(cfg.solid_angle_threshold > 0.0);
    }

    #[test]
    fn winding_classify_above_threshold() {
        assert!(winding_classify(0.9, 0.5));
        assert!(!winding_classify(0.3, 0.5));
    }

    #[test]
    fn winding_classify_negative_wn() {
        assert!(winding_classify(-0.9, 0.5));
        assert!(!winding_classify(-0.1, 0.5));
    }

    #[test]
    fn solid_angle_triangle_origin_far() {
        // Point far from triangle: solid angle near zero
        let sa = solid_angle_triangle([0.0, 0.0, 1000.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(sa.abs() < 0.01, "sa={sa}");
    }

    #[test]
    fn solid_angle_coincident_point_returns_zero() {
        // Point at same location as vertex
        let sa = solid_angle_triangle([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert_eq!(sa, 0.0);
    }

    #[test]
    fn batch_winding_count_matches() {
        let cfg = default_winding_config();
        let pts = vec![[0.0_f32; 3]; 5];
        let pos = unit_tetrahedron_positions();
        let tris = unit_tetrahedron_triangles();
        let result = batch_winding(&pts, &pos, &tris, &cfg);
        assert_eq!(result.total_queries, 5);
    }

    #[test]
    fn inside_ratio_empty() {
        let wr = WindingResult {
            queries: vec![],
            total_queries: 0,
            inside_count: 0,
        };
        assert_eq!(winding_result_inside_ratio(&wr), 0.0);
    }

    #[test]
    fn inside_ratio_all_inside() {
        let q = WindingQuery {
            point: [0.0; 3],
            winding_number: 1.0,
            is_inside: true,
        };
        let wr = WindingResult {
            total_queries: 1,
            inside_count: 1,
            queries: vec![q],
        };
        assert!((winding_result_inside_ratio(&wr) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn inside_points_filters_correctly() {
        let q1 = WindingQuery { point: [1.0, 0.0, 0.0], winding_number: 0.8, is_inside: true };
        let q2 = WindingQuery { point: [2.0, 0.0, 0.0], winding_number: 0.1, is_inside: false };
        let wr = WindingResult { queries: vec![q1, q2], total_queries: 2, inside_count: 1 };
        let pts = inside_points(&wr);
        assert_eq!(pts.len(), 1);
        assert!((pts[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn winding_query_to_json_format() {
        let wq = WindingQuery { point: [1.0, 2.0, 3.0], winding_number: 0.5, is_inside: true };
        let s = winding_query_to_json(&wq);
        assert!(s.contains("winding_number"));
        assert!(s.contains("is_inside"));
    }

    #[test]
    fn winding_result_to_json_format() {
        let wr = WindingResult { queries: vec![], total_queries: 10, inside_count: 5 };
        let s = winding_result_to_json(&wr);
        assert!(s.contains("total_queries"));
        assert!(s.contains("inside_count"));
    }
}
