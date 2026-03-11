//! UV seam and hard edge detection for mesh unwrapping.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SeamDetectConfig {
    pub angle_threshold_deg: f32,
    pub detect_uv_seams: bool,
    pub detect_hard_edges: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshSeam {
    pub edge_a: u32,
    pub edge_b: u32,
    pub seam_type: SeamType,
    pub angle: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum SeamType {
    UvSeam,
    HardEdge,
    Both,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SeamDetectResult {
    pub seams: Vec<MeshSeam>,
    pub hard_edge_count: usize,
    pub uv_seam_count: usize,
}

#[allow(dead_code)]
pub fn default_seam_detect_config() -> SeamDetectConfig {
    SeamDetectConfig {
        angle_threshold_deg: 30.0,
        detect_uv_seams: true,
        detect_hard_edges: true,
    }
}

#[allow(dead_code)]
pub fn edge_angle_deg(n_a: [f32; 3], n_b: [f32; 3]) -> f32 {
    let dot = (n_a[0] * n_b[0] + n_a[1] * n_b[1] + n_a[2] * n_b[2]).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}

#[allow(dead_code)]
pub fn is_hard_edge(n_a: [f32; 3], n_b: [f32; 3], threshold_deg: f32) -> bool {
    edge_angle_deg(n_a, n_b) > threshold_deg
}

#[allow(dead_code)]
pub fn detect_seams(
    normals: &[[f32; 3]],
    uv0: &[[f32; 2]],
    edges: &[[u32; 2]],
    cfg: &SeamDetectConfig,
) -> SeamDetectResult {
    let mut seams = Vec::new();
    let mut hard_edge_count = 0usize;
    let mut uv_seam_count = 0usize;

    for edge in edges {
        let a = edge[0] as usize;
        let b = edge[1] as usize;

        if a >= normals.len() || b >= normals.len() {
            continue;
        }

        let hard = cfg.detect_hard_edges && is_hard_edge(normals[a], normals[b], cfg.angle_threshold_deg);
        let uv_seam = cfg.detect_uv_seams && a < uv0.len() && b < uv0.len() && {
            let du = uv0[a][0] - uv0[b][0];
            let dv = uv0[a][1] - uv0[b][1];
            (du * du + dv * dv).sqrt() > 0.001
        };

        let seam_type = match (hard, uv_seam) {
            (true, true) => Some(SeamType::Both),
            (true, false) => Some(SeamType::HardEdge),
            (false, true) => Some(SeamType::UvSeam),
            _ => None,
        };

        if let Some(st) = seam_type {
            if matches!(st, SeamType::HardEdge | SeamType::Both) {
                hard_edge_count += 1;
            }
            if matches!(st, SeamType::UvSeam | SeamType::Both) {
                uv_seam_count += 1;
            }
            let angle = edge_angle_deg(normals[a], normals[b]);
            seams.push(MeshSeam {
                edge_a: edge[0],
                edge_b: edge[1],
                seam_type: st,
                angle,
            });
        }
    }

    SeamDetectResult { seams, hard_edge_count, uv_seam_count }
}

#[allow(dead_code)]
pub fn seam_type_name(s: &MeshSeam) -> &'static str {
    match s.seam_type {
        SeamType::UvSeam => "uv_seam",
        SeamType::HardEdge => "hard_edge",
        SeamType::Both => "both",
    }
}

#[allow(dead_code)]
pub fn seam_count(result: &SeamDetectResult) -> usize {
    result.seams.len()
}

#[allow(dead_code)]
pub fn seam_detect_to_json(result: &SeamDetectResult) -> String {
    format!(
        "{{\"hard_edge_count\":{},\"uv_seam_count\":{},\"total\":{}}}",
        result.hard_edge_count,
        result.uv_seam_count,
        result.seams.len()
    )
}

#[allow(dead_code)]
pub fn filter_seams_by_type<'a>(result: &'a SeamDetectResult, t: &SeamType) -> Vec<&'a MeshSeam> {
    result.seams.iter().filter(|s| &s.seam_type == t).collect()
}

#[allow(dead_code)]
pub fn seam_to_json(s: &MeshSeam) -> String {
    format!(
        "{{\"edge_a\":{},\"edge_b\":{},\"type\":\"{}\",\"angle\":{:.4}}}",
        s.edge_a,
        s.edge_b,
        seam_type_name(s),
        s.angle
    )
}

#[allow(dead_code)]
pub fn mark_seam_edges(result: &SeamDetectResult, edge_count: usize) -> Vec<bool> {
    let mut marks = vec![false; edge_count];
    for seam in &result.seams {
        let idx = seam.edge_a as usize;
        if idx < edge_count {
            marks[idx] = true;
        }
    }
    marks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_normals(n: usize) -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 1.0]; n]
    }

    #[test]
    fn default_config_has_sane_threshold() {
        let cfg = default_seam_detect_config();
        assert!(cfg.angle_threshold_deg > 0.0);
        assert!(cfg.detect_uv_seams);
        assert!(cfg.detect_hard_edges);
    }

    #[test]
    fn edge_angle_same_normal_is_zero() {
        let n = [0.0f32, 1.0, 0.0];
        let angle = edge_angle_deg(n, n);
        assert!(angle.abs() < 1e-4, "same normal => 0 deg");
    }

    #[test]
    fn edge_angle_opposite_normals_is_180() {
        let n_a = [0.0f32, 1.0, 0.0];
        let n_b = [0.0f32, -1.0, 0.0];
        let angle = edge_angle_deg(n_a, n_b);
        assert!((angle - 180.0).abs() < 1e-3);
    }

    #[test]
    fn is_hard_edge_detects_90deg() {
        let n_a = [1.0f32, 0.0, 0.0];
        let n_b = [0.0f32, 1.0, 0.0];
        assert!(is_hard_edge(n_a, n_b, 30.0));
        assert!(!is_hard_edge(n_a, n_b, 95.0));
    }

    #[test]
    fn detect_seams_empty_returns_empty() {
        let cfg = default_seam_detect_config();
        let result = detect_seams(&[], &[], &[], &cfg);
        assert_eq!(result.seams.len(), 0);
        assert_eq!(result.hard_edge_count, 0);
        assert_eq!(result.uv_seam_count, 0);
    }

    #[test]
    fn detect_seams_hard_edge_counted() {
        let normals = vec![[1.0f32, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uv0 = vec![[0.0f32, 0.0], [0.0, 0.0]];
        let edges = vec![[0u32, 1]];
        let cfg = SeamDetectConfig {
            angle_threshold_deg: 30.0,
            detect_uv_seams: false,
            detect_hard_edges: true,
        };
        let result = detect_seams(&normals, &uv0, &edges, &cfg);
        assert_eq!(result.hard_edge_count, 1);
    }

    #[test]
    fn detect_seams_uv_seam_counted() {
        let normals = flat_normals(2);
        let uv0 = vec![[0.0f32, 0.0], [1.0, 1.0]];
        let edges = vec![[0u32, 1]];
        let cfg = SeamDetectConfig {
            angle_threshold_deg: 30.0,
            detect_uv_seams: true,
            detect_hard_edges: false,
        };
        let result = detect_seams(&normals, &uv0, &edges, &cfg);
        assert_eq!(result.uv_seam_count, 1);
    }

    #[test]
    fn seam_type_name_returns_correct_str() {
        let s = MeshSeam { edge_a: 0, edge_b: 1, seam_type: SeamType::Both, angle: 45.0 };
        assert_eq!(seam_type_name(&s), "both");
        let s2 = MeshSeam { seam_type: SeamType::UvSeam, ..s.clone() };
        assert_eq!(seam_type_name(&s2), "uv_seam");
        let s3 = MeshSeam { seam_type: SeamType::HardEdge, ..s };
        assert_eq!(seam_type_name(&s3), "hard_edge");
    }

    #[test]
    fn seam_count_matches_vec_len() {
        let result = SeamDetectResult {
            seams: vec![
                MeshSeam { edge_a: 0, edge_b: 1, seam_type: SeamType::UvSeam, angle: 0.0 },
                MeshSeam { edge_a: 2, edge_b: 3, seam_type: SeamType::HardEdge, angle: 45.0 },
            ],
            hard_edge_count: 1,
            uv_seam_count: 1,
        };
        assert_eq!(seam_count(&result), 2);
    }

    #[test]
    fn seam_detect_to_json_contains_counts() {
        let result = SeamDetectResult {
            seams: vec![],
            hard_edge_count: 3,
            uv_seam_count: 2,
        };
        let json = seam_detect_to_json(&result);
        assert!(json.contains("\"hard_edge_count\":3"));
        assert!(json.contains("\"uv_seam_count\":2"));
    }

    #[test]
    fn filter_seams_by_type_works() {
        let result = SeamDetectResult {
            seams: vec![
                MeshSeam { edge_a: 0, edge_b: 1, seam_type: SeamType::UvSeam, angle: 0.0 },
                MeshSeam { edge_a: 2, edge_b: 3, seam_type: SeamType::HardEdge, angle: 45.0 },
                MeshSeam { edge_a: 4, edge_b: 5, seam_type: SeamType::UvSeam, angle: 1.0 },
            ],
            hard_edge_count: 1,
            uv_seam_count: 2,
        };
        let uv_seams = filter_seams_by_type(&result, &SeamType::UvSeam);
        assert_eq!(uv_seams.len(), 2);
        let hard = filter_seams_by_type(&result, &SeamType::HardEdge);
        assert_eq!(hard.len(), 1);
    }

    #[test]
    fn mark_seam_edges_marks_correct_indices() {
        let result = SeamDetectResult {
            seams: vec![
                MeshSeam { edge_a: 0, edge_b: 1, seam_type: SeamType::UvSeam, angle: 0.0 },
                MeshSeam { edge_a: 2, edge_b: 3, seam_type: SeamType::HardEdge, angle: 45.0 },
            ],
            hard_edge_count: 1,
            uv_seam_count: 1,
        };
        let marks = mark_seam_edges(&result, 4);
        assert!(marks[0]);
        assert!(!marks[1]);
        assert!(marks[2]);
        assert!(!marks[3]);
    }

    #[test]
    fn seam_to_json_contains_fields() {
        let s = MeshSeam { edge_a: 1, edge_b: 2, seam_type: SeamType::Both, angle: 90.0 };
        let json = seam_to_json(&s);
        assert!(json.contains("\"edge_a\":1"));
        assert!(json.contains("\"edge_b\":2"));
        assert!(json.contains("\"type\":\"both\""));
    }
}
