//! Muscle-driven surface deformation system (distinct from muscle_line visualization).

#[allow(dead_code)]
pub struct MuscleDeformConfig {
    pub max_influences: usize,
    pub blend_weight: f32,
    pub normalize: bool,
    pub energy_threshold: f32,
}

#[allow(dead_code)]
pub struct MuscleInfluence {
    pub muscle_name: String,
    pub vertex_index: usize,
    pub weight: f32,
    pub direction: [f32; 3],
    pub magnitude: f32,
}

#[allow(dead_code)]
pub struct MuscleDeformer {
    pub config: MuscleDeformConfig,
    pub influences: Vec<MuscleInfluence>,
    pub vertex_count: usize,
    pub activations: Vec<f32>,
}

#[allow(dead_code)]
pub fn default_muscle_deform_config() -> MuscleDeformConfig {
    MuscleDeformConfig {
        max_influences: 4,
        blend_weight: 1.0,
        normalize: true,
        energy_threshold: 1e-6,
    }
}

#[allow(dead_code)]
pub fn new_muscle_deformer(vertex_count: usize, config: MuscleDeformConfig) -> MuscleDeformer {
    let activations = vec![0.0f32; vertex_count];
    MuscleDeformer {
        config,
        influences: Vec::new(),
        vertex_count,
        activations,
    }
}

#[allow(dead_code)]
pub fn add_muscle_influence(deformer: &mut MuscleDeformer, influence: MuscleInfluence) {
    deformer.influences.push(influence);
}

#[allow(dead_code)]
pub fn compute_deformation(deformer: &MuscleDeformer) -> Vec<[f32; 3]> {
    let mut deltas = vec![[0.0f32; 3]; deformer.vertex_count];
    for inf in &deformer.influences {
        if inf.vertex_index >= deformer.vertex_count {
            continue;
        }
        let w = inf.weight * inf.magnitude * deformer.config.blend_weight;
        let d = &mut deltas[inf.vertex_index];
        d[0] += inf.direction[0] * w;
        d[1] += inf.direction[1] * w;
        d[2] += inf.direction[2] * w;
    }
    deltas
}

#[allow(dead_code)]
pub fn deformer_vertex_count(deformer: &MuscleDeformer) -> usize {
    deformer.vertex_count
}

#[allow(dead_code)]
pub fn muscle_influence_count(deformer: &MuscleDeformer) -> usize {
    deformer.influences.len()
}

#[allow(dead_code)]
pub fn set_influence_weight(deformer: &mut MuscleDeformer, index: usize, weight: f32) {
    if let Some(inf) = deformer.influences.get_mut(index) {
        inf.weight = weight.clamp(0.0, 1.0);
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
pub fn apply_deformation_to_mesh(
    positions: &mut [[f32; 3]],
    deformer: &MuscleDeformer,
    scale: f32,
) {
    let deltas = compute_deformation(deformer);
    for (i, pos) in positions.iter_mut().enumerate() {
        if let Some(d) = deltas.get(i) {
            pos[0] += d[0] * scale;
            pos[1] += d[1] * scale;
            pos[2] += d[2] * scale;
        }
    }
}

#[allow(dead_code)]
pub fn deformation_energy(deformer: &MuscleDeformer) -> f32 {
    deformer.influences.iter().map(|inf| {
        let w = inf.weight * inf.magnitude;
        w * w
    }).sum()
}

#[allow(dead_code)]
pub fn normalize_influences(deformer: &mut MuscleDeformer) {
    // Compute total weight per vertex
    let mut totals = vec![0.0f32; deformer.vertex_count];
    for inf in &deformer.influences {
        if inf.vertex_index < deformer.vertex_count {
            totals[inf.vertex_index] += inf.weight;
        }
    }
    for inf in &mut deformer.influences {
        if inf.vertex_index < deformer.vertex_count {
            let t = totals[inf.vertex_index];
            if t > 1e-8 {
                inf.weight /= t;
            }
        }
    }
}

#[allow(dead_code)]
pub fn muscle_deform_to_json(deformer: &MuscleDeformer) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"vertex_count\":{}", deformer.vertex_count));
    parts.push(format!("\"influence_count\":{}", deformer.influences.len()));
    parts.push(format!("\"blend_weight\":{}", deformer.config.blend_weight));
    parts.push(format!("\"normalize\":{}", deformer.config.normalize));
    let energy = deformation_energy(deformer);
    parts.push(format!("\"energy\":{energy:.6}"));
    format!("{{{}}}", parts.join(","))
}

#[allow(dead_code)]
pub fn reset_deformer(deformer: &mut MuscleDeformer) {
    deformer.influences.clear();
    for a in &mut deformer.activations {
        *a = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_deformer() -> MuscleDeformer {
        let cfg = default_muscle_deform_config();
        new_muscle_deformer(10, cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_muscle_deform_config();
        assert_eq!(cfg.max_influences, 4);
        assert!((cfg.blend_weight - 1.0).abs() < 1e-6);
        assert!(cfg.normalize);
    }

    #[test]
    fn test_new_deformer_vertex_count() {
        let d = make_deformer();
        assert_eq!(deformer_vertex_count(&d), 10);
    }

    #[test]
    fn test_add_influence() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "bicep".to_string(),
            vertex_index: 0,
            weight: 0.5,
            direction: [0.0, 1.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        assert_eq!(muscle_influence_count(&d), 1);
    }

    #[test]
    fn test_compute_deformation_zero() {
        let d = make_deformer();
        let deltas = compute_deformation(&d);
        assert_eq!(deltas.len(), 10);
        for delta in &deltas {
            assert_eq!(*delta, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_compute_deformation_single_influence() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "tricep".to_string(),
            vertex_index: 3,
            weight: 1.0,
            direction: [1.0, 0.0, 0.0],
            magnitude: 2.0,
        };
        add_muscle_influence(&mut d, inf);
        let deltas = compute_deformation(&d);
        assert!((deltas[3][0] - 2.0).abs() < 1e-5);
        assert!((deltas[3][1]).abs() < 1e-5);
    }

    #[test]
    fn test_set_influence_weight() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "quad".to_string(),
            vertex_index: 1,
            weight: 0.3,
            direction: [0.0, 0.0, 1.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        set_influence_weight(&mut d, 0, 0.9);
        assert!((d.influences[0].weight - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_influence_weight_clamped() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "calf".to_string(),
            vertex_index: 2,
            weight: 0.5,
            direction: [0.0, 1.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        set_influence_weight(&mut d, 0, 1.5);
        assert!((d.influences[0].weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_deformation_to_mesh() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "pec".to_string(),
            vertex_index: 0,
            weight: 1.0,
            direction: [0.0, 1.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        let mut positions = vec![[0.0f32; 3]; 10];
        apply_deformation_to_mesh(&mut positions, &d, 1.0);
        assert!((positions[0][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deformation_energy_zero() {
        let d = make_deformer();
        let energy = deformation_energy(&d);
        assert!((energy).abs() < 1e-8);
    }

    #[test]
    fn test_deformation_energy_nonzero() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "delt".to_string(),
            vertex_index: 5,
            weight: 1.0,
            direction: [1.0, 0.0, 0.0],
            magnitude: 2.0,
        };
        add_muscle_influence(&mut d, inf);
        let energy = deformation_energy(&d);
        assert!(energy > 0.0);
    }

    #[test]
    fn test_normalize_influences() {
        let mut d = make_deformer();
        for i in 0..2 {
            let inf = MuscleInfluence {
                muscle_name: format!("m{i}"),
                vertex_index: 0,
                weight: 0.5,
                direction: [1.0, 0.0, 0.0],
                magnitude: 1.0,
            };
            add_muscle_influence(&mut d, inf);
        }
        normalize_influences(&mut d);
        let total: f32 = d.influences.iter().filter(|i| i.vertex_index == 0).map(|i| i.weight).sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_muscle_deform_to_json() {
        let d = make_deformer();
        let json = muscle_deform_to_json(&d);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("influence_count"));
        assert!(json.contains("energy"));
    }

    #[test]
    fn test_reset_deformer() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "lat".to_string(),
            vertex_index: 0,
            weight: 1.0,
            direction: [1.0, 0.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        reset_deformer(&mut d);
        assert_eq!(muscle_influence_count(&d), 0);
    }

    #[test]
    fn test_out_of_bounds_influence_ignored() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "ghost".to_string(),
            vertex_index: 999,
            weight: 1.0,
            direction: [1.0, 0.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        let deltas = compute_deformation(&d);
        for delta in &deltas {
            assert_eq!(*delta, [0.0, 0.0, 0.0]);
        }
    }

    #[test]
    fn test_multiple_influences_same_vertex() {
        let mut d = make_deformer();
        for _ in 0..3 {
            let inf = MuscleInfluence {
                muscle_name: "multi".to_string(),
                vertex_index: 2,
                weight: 1.0,
                direction: [0.0, 0.0, 1.0],
                magnitude: 1.0,
            };
            add_muscle_influence(&mut d, inf);
        }
        let deltas = compute_deformation(&d);
        assert!((deltas[2][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_deformation_scale() {
        let mut d = make_deformer();
        let inf = MuscleInfluence {
            muscle_name: "scaled".to_string(),
            vertex_index: 0,
            weight: 1.0,
            direction: [1.0, 0.0, 0.0],
            magnitude: 1.0,
        };
        add_muscle_influence(&mut d, inf);
        let mut positions = vec![[0.0f32; 3]; 10];
        apply_deformation_to_mesh(&mut positions, &d, 2.0);
        assert!((positions[0][0] - 2.0).abs() < 1e-5);
    }
}
