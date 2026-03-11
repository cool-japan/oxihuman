//! Weighted average of multiple vertex position arrays for morph blending at geometry level.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedAvgConfig {
    pub vertex_count: usize,
    pub normalize_weights: bool,
    pub clamp_output: bool,
    pub min_weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedMesh {
    pub positions: Vec<[f32; 3]>,
    pub weight: f32,
    pub label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedAvgResult {
    pub positions: Vec<[f32; 3]>,
    pub total_weight: f32,
    pub mesh_count: usize,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub struct WeightedAvg {
    config: WeightedAvgConfig,
    meshes: Vec<WeightedMesh>,
}

#[allow(dead_code)]
pub fn default_weighted_avg_config() -> WeightedAvgConfig {
    WeightedAvgConfig {
        vertex_count: 0,
        normalize_weights: true,
        clamp_output: false,
        min_weight: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_weighted_avg(config: WeightedAvgConfig) -> WeightedAvg {
    WeightedAvg { config, meshes: Vec::new() }
}

#[allow(dead_code)]
pub fn weighted_avg_add_mesh(wa: &mut WeightedAvg, positions: Vec<[f32; 3]>, weight: f32, label: &str) {
    if wa.config.vertex_count == 0 {
        wa.config.vertex_count = positions.len();
    }
    wa.meshes.push(WeightedMesh {
        positions,
        weight: weight.max(wa.config.min_weight),
        label: label.to_string(),
    });
}

#[allow(dead_code)]
pub fn weighted_avg_compute(wa: &WeightedAvg) -> WeightedAvgResult {
    if wa.meshes.is_empty() {
        return WeightedAvgResult {
            positions: Vec::new(),
            total_weight: 0.0,
            mesh_count: 0,
            vertex_count: 0,
        };
    }

    let total_w = weighted_avg_total_weight(wa);
    let n = wa.config.vertex_count;
    let mut out = vec![[0.0f32; 3]; n];

    let denom = if wa.config.normalize_weights && total_w > 0.0 {
        total_w
    } else {
        1.0
    };

    for mesh in &wa.meshes {
        let w = mesh.weight / denom;
        for (i, p) in mesh.positions.iter().enumerate().take(n) {
            out[i][0] += p[0] * w;
            out[i][1] += p[1] * w;
            out[i][2] += p[2] * w;
        }
    }

    WeightedAvgResult {
        positions: out,
        total_weight: total_w,
        mesh_count: wa.meshes.len(),
        vertex_count: n,
    }
}

#[allow(dead_code)]
pub fn weighted_avg_mesh_count(wa: &WeightedAvg) -> usize {
    wa.meshes.len()
}

#[allow(dead_code)]
pub fn weighted_avg_set_weight(wa: &mut WeightedAvg, index: usize, weight: f32) {
    if let Some(m) = wa.meshes.get_mut(index) {
        m.weight = weight.max(wa.config.min_weight);
    }
}

#[allow(dead_code)]
pub fn weighted_avg_total_weight(wa: &WeightedAvg) -> f32 {
    wa.meshes.iter().map(|m| m.weight).sum()
}

#[allow(dead_code)]
pub fn weighted_avg_to_json(wa: &WeightedAvg) -> String {
    let total = weighted_avg_total_weight(wa);
    let labels: Vec<String> = wa.meshes.iter().map(|m| format!("\"{}\":{}", m.label, m.weight)).collect();
    format!(
        "{{\"mesh_count\":{},\"vertex_count\":{},\"total_weight\":{:.6},\"meshes\":[{}]}}",
        wa.meshes.len(),
        wa.config.vertex_count,
        total,
        labels.join(",")
    )
}

#[allow(dead_code)]
pub fn weighted_avg_clear(wa: &mut WeightedAvg) {
    wa.meshes.clear();
    wa.config.vertex_count = 0;
}

#[allow(dead_code)]
pub fn weighted_avg_vertex_count(wa: &WeightedAvg) -> usize {
    wa.config.vertex_count
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_square(offset: f32) -> Vec<[f32; 3]> {
        vec![
            [offset, 0.0, 0.0],
            [offset + 1.0, 0.0, 0.0],
            [offset + 1.0, 1.0, 0.0],
            [offset, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_weighted_avg_config();
        assert!(cfg.normalize_weights);
        assert_eq!(cfg.vertex_count, 0);
    }

    #[test]
    fn test_new_empty() {
        let wa = new_weighted_avg(default_weighted_avg_config());
        assert_eq!(weighted_avg_mesh_count(&wa), 0);
        assert_eq!(weighted_avg_vertex_count(&wa), 0);
    }

    #[test]
    fn test_add_mesh_updates_vertex_count() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, make_square(0.0), 1.0, "base");
        assert_eq!(weighted_avg_vertex_count(&wa), 4);
        assert_eq!(weighted_avg_mesh_count(&wa), 1);
    }

    #[test]
    fn test_compute_single_mesh() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, make_square(0.0), 1.0, "base");
        let res = weighted_avg_compute(&wa);
        assert_eq!(res.vertex_count, 4);
        assert!((res.positions[0][0] - 0.0).abs() < 1e-5);
        assert!((res.positions[1][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_two_meshes_equal_weight() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, vec![[0.0, 0.0, 0.0]], 1.0, "a");
        weighted_avg_add_mesh(&mut wa, vec![[2.0, 0.0, 0.0]], 1.0, "b");
        let res = weighted_avg_compute(&wa);
        assert!((res.positions[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_weight() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, vec![[0.0, 0.0, 0.0]], 1.0, "a");
        weighted_avg_set_weight(&mut wa, 0, 0.5);
        assert!((weighted_avg_total_weight(&wa) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_total_weight() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, vec![[0.0, 0.0, 0.0]], 0.3, "a");
        weighted_avg_add_mesh(&mut wa, vec![[1.0, 0.0, 0.0]], 0.7, "b");
        assert!((weighted_avg_total_weight(&wa) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, make_square(0.0), 1.0, "x");
        weighted_avg_clear(&mut wa);
        assert_eq!(weighted_avg_mesh_count(&wa), 0);
        assert_eq!(weighted_avg_vertex_count(&wa), 0);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, vec![[0.0, 0.0, 0.0]], 1.0, "base");
        let json = weighted_avg_to_json(&wa);
        assert!(json.contains("mesh_count"));
        assert!(json.contains("total_weight"));
        assert!(json.contains("base"));
    }

    #[test]
    fn test_compute_empty_returns_empty() {
        let wa = new_weighted_avg(default_weighted_avg_config());
        let res = weighted_avg_compute(&wa);
        assert_eq!(res.mesh_count, 0);
        assert!(res.positions.is_empty());
    }

    #[test]
    fn test_weighted_blend_asymmetric() {
        let mut wa = new_weighted_avg(default_weighted_avg_config());
        weighted_avg_add_mesh(&mut wa, vec![[0.0, 0.0, 0.0]], 0.25, "a");
        weighted_avg_add_mesh(&mut wa, vec![[4.0, 0.0, 0.0]], 0.75, "b");
        let res = weighted_avg_compute(&wa);
        // 0.25*0 + 0.75*4 = 3.0
        assert!((res.positions[0][0] - 3.0).abs() < 1e-5);
    }
}
