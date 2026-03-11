//! Cage-based mesh deformation using mean value coordinates.

/// Configuration for cage-based deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageDeformConfig {
    pub smooth_iterations: u32,
    pub blend_weight: f32,
    pub use_mean_value: bool,
}

/// A cage mesh used to deform an enclosed mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageMesh {
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// Per-vertex cage weights mapping a source vertex to cage vertices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageWeights {
    pub vertex_index: u32,
    pub cage_weights: Vec<f32>,
}

/// Result of deforming a mesh with a cage.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CageDeformResult {
    pub deformed_positions: Vec<[f32; 3]>,
    pub cage_vertex_count: usize,
    pub success: bool,
}

/// Return a default `CageDeformConfig`.
#[allow(dead_code)]
pub fn default_cage_deform_config() -> CageDeformConfig {
    CageDeformConfig {
        smooth_iterations: 2,
        blend_weight: 1.0,
        use_mean_value: true,
    }
}

/// Construct a new `CageMesh`.
#[allow(dead_code)]
pub fn new_cage_mesh(positions: Vec<[f32; 3]>, triangles: Vec<[u32; 3]>) -> CageMesh {
    CageMesh {
        positions,
        triangles,
    }
}

/// Compute mean value coordinate weights for `point` relative to `cage`.
///
/// For each cage vertex the weight is inversely proportional to the distance
/// from the point to that vertex.  Weights are normalised to sum to 1.
#[allow(dead_code)]
pub fn compute_cage_weights(point: [f32; 3], cage: &CageMesh) -> CageWeights {
    let n = cage.positions.len();
    if n == 0 {
        return CageWeights {
            vertex_index: 0,
            cage_weights: vec![],
        };
    }

    // Simple inverse-distance weighting as a practical approximation of MVC.
    let mut weights: Vec<f32> = cage
        .positions
        .iter()
        .map(|&cv| {
            let dx = point[0] - cv[0];
            let dy = point[1] - cv[1];
            let dz = point[2] - cv[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            1.0 / (dist + 1e-8)
        })
        .collect();

    let sum: f32 = weights.iter().sum();
    if sum > 1e-12 {
        for w in &mut weights {
            *w /= sum;
        }
    } else {
        let inv = 1.0 / n as f32;
        weights.fill(inv);
    }

    CageWeights {
        vertex_index: 0,
        cage_weights: weights,
    }
}

/// Deform `src_positions` by re-evaluating cage weights against `cage_deformed`.
#[allow(dead_code)]
pub fn deform_with_cage(
    src_positions: &[[f32; 3]],
    cage_orig: &CageMesh,
    cage_deformed: &CageMesh,
    cfg: &CageDeformConfig,
) -> CageDeformResult {
    if cage_orig.positions.len() != cage_deformed.positions.len() {
        return CageDeformResult {
            deformed_positions: src_positions.to_vec(),
            cage_vertex_count: cage_orig.positions.len(),
            success: false,
        };
    }

    let mut deformed: Vec<[f32; 3]> = src_positions
        .iter()
        .map(|&p| {
            let w = compute_cage_weights(p, cage_orig);
            let mut result = apply_cage_weights(&w, &cage_deformed.positions);
            if cfg.blend_weight < 1.0 {
                let t = cfg.blend_weight;
                result[0] = p[0] * (1.0 - t) + result[0] * t;
                result[1] = p[1] * (1.0 - t) + result[1] * t;
                result[2] = p[2] * (1.0 - t) + result[2] * t;
            }
            result
        })
        .collect();

    if cfg.smooth_iterations > 0 {
        smooth_deformed(&mut deformed, cfg.smooth_iterations);
    }

    let cage_vertex_count = cage_deformed.positions.len();
    CageDeformResult {
        deformed_positions: deformed,
        cage_vertex_count,
        success: true,
    }
}

/// Return the number of vertices in a cage.
#[allow(dead_code)]
pub fn cage_vertex_count(cage: &CageMesh) -> usize {
    cage.positions.len()
}

/// Return the number of triangles in a cage.
#[allow(dead_code)]
pub fn cage_face_count(cage: &CageMesh) -> usize {
    cage.triangles.len()
}

/// Interpolate cage vertex positions using the given weights.
#[allow(dead_code)]
pub fn apply_cage_weights(weights: &CageWeights, cage_positions: &[[f32; 3]]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    let n = weights.cage_weights.len().min(cage_positions.len());
    for (cw, cp) in weights.cage_weights.iter().zip(cage_positions.iter()).take(n) {
        out[0] += cw * cp[0];
        out[1] += cw * cp[1];
        out[2] += cw * cp[2];
    }
    out
}

/// Serialize a `CageDeformResult` to a JSON string.
#[allow(dead_code)]
pub fn cage_deform_result_to_json(r: &CageDeformResult) -> String {
    format!(
        "{{\"cage_vertex_count\":{},\"success\":{},\"deformed_count\":{}}}",
        r.cage_vertex_count,
        r.success,
        r.deformed_positions.len()
    )
}

/// Return `true` if the weights sum to approximately 1.0.
#[allow(dead_code)]
pub fn validate_cage_weights(weights: &CageWeights) -> bool {
    let sum: f32 = weights.cage_weights.iter().sum();
    (sum - 1.0).abs() < 1e-3
}

/// Apply a simple averaging smoothing pass over deformed positions.
#[allow(dead_code)]
pub fn smooth_deformed(positions: &mut [[f32; 3]], iterations: u32) {
    let n = positions.len();
    if n < 3 {
        return;
    }
    for _ in 0..iterations {
        let orig = positions.to_vec();
        for i in 0..n {
            let prev = &orig[(i + n - 1) % n];
            let next = &orig[(i + 1) % n];
            positions[i][0] = (prev[0] + orig[i][0] + next[0]) / 3.0;
            positions[i][1] = (prev[1] + orig[i][1] + next[1]) / 3.0;
            positions[i][2] = (prev[2] + orig[i][2] + next[2]) / 3.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cage() -> CageMesh {
        new_cage_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]],
        )
    }

    #[test]
    fn default_config_fields() {
        let cfg = default_cage_deform_config();
        assert!(cfg.blend_weight > 0.0);
        assert!(cfg.use_mean_value);
    }

    #[test]
    fn cage_vertex_count_correct() {
        let cage = simple_cage();
        assert_eq!(cage_vertex_count(&cage), 4);
    }

    #[test]
    fn cage_face_count_correct() {
        let cage = simple_cage();
        assert_eq!(cage_face_count(&cage), 4);
    }

    #[test]
    fn cage_weights_sum_to_one() {
        let cage = simple_cage();
        let w = compute_cage_weights([0.5, 0.3, 0.2], &cage);
        assert!(validate_cage_weights(&w), "weights must sum to 1");
    }

    #[test]
    fn apply_cage_weights_roundtrip() {
        let cage = simple_cage();
        // A point equal to cage vertex 0 should reconstruct very close to that vertex.
        let point = [0.0f32, 0.0, 0.0];
        let w = compute_cage_weights(point, &cage);
        let result = apply_cage_weights(&w, &cage.positions);
        // Result should be dominated by the nearest cage vertex
        assert!(
            result[0] < 0.5,
            "x should be near the source vertex x=0.0, got {}",
            result[0]
        );
    }

    #[test]
    fn deform_with_cage_success() {
        let cage_orig = simple_cage();
        let mut cage_def = cage_orig.clone();
        // Translate cage up by 1.0 in y
        for p in &mut cage_def.positions {
            p[1] += 1.0;
        }
        let src = vec![[0.5f32, 0.3, 0.1]];
        let cfg = default_cage_deform_config();
        let result = deform_with_cage(&src, &cage_orig, &cage_def, &cfg);
        assert!(result.success);
        assert_eq!(result.deformed_positions.len(), 1);
        // y should increase
        assert!(result.deformed_positions[0][1] > src[0][1]);
    }

    #[test]
    fn deform_with_cage_mismatched_returns_failure() {
        let cage_orig = simple_cage();
        let cage_def = new_cage_mesh(vec![[0.0, 0.0, 0.0]], vec![]);
        let src = vec![[0.1f32, 0.1, 0.1]];
        let cfg = default_cage_deform_config();
        let result = deform_with_cage(&src, &cage_orig, &cage_def, &cfg);
        assert!(!result.success);
    }

    #[test]
    fn cage_deform_result_to_json_contains_fields() {
        let r = CageDeformResult {
            deformed_positions: vec![[0.0, 0.0, 0.0]],
            cage_vertex_count: 4,
            success: true,
        };
        let json = cage_deform_result_to_json(&r);
        assert!(json.contains("cage_vertex_count"));
        assert!(json.contains("true"));
    }

    #[test]
    fn smooth_deformed_does_not_panic_on_small() {
        let mut pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        smooth_deformed(&mut pts, 3); // should not panic
    }
}
