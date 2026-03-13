// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Automatic skinning weight computation via heat diffusion from bone endpoints.

#[allow(dead_code)]
pub struct BoneEndpoint {
    pub name: String,
    pub position: [f32; 3],
    pub radius: f32,
}

#[allow(dead_code)]
pub enum WeightFalloff {
    Linear,
    Quadratic,
    Gaussian,
}

#[allow(dead_code)]
pub struct SkinWeightConfig {
    pub diffusion_iterations: u32,
    pub falloff: WeightFalloff,
    pub normalize: bool,
}

impl Default for SkinWeightConfig {
    fn default() -> Self {
        Self {
            diffusion_iterations: 30,
            falloff: WeightFalloff::Linear,
            normalize: true,
        }
    }
}

#[allow(dead_code)]
pub struct AutoSkinResult {
    pub weights: Vec<Vec<f32>>,
    pub bone_count: usize,
    pub vertex_count: usize,
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute heat influence of a bone on a vertex.
#[allow(dead_code)]
pub fn bone_heat(vertex: [f32; 3], bone: &BoneEndpoint, falloff: &WeightFalloff) -> f32 {
    let d = dist3(vertex, bone.position);
    if d >= bone.radius || bone.radius < 1e-12 {
        return 0.0;
    }
    let t = 1.0 - d / bone.radius; // t in (0, 1]
    match falloff {
        WeightFalloff::Linear => t,
        WeightFalloff::Quadratic => t * t,
        WeightFalloff::Gaussian => {
            // Gaussian: exp(-3 * (d/radius)^2) clamped to 0 at boundary
            let normalized = d / bone.radius;
            (-3.0 * normalized * normalized).exp() - (-3.0f32).exp()
        }
    }
    .max(0.0)
}

/// Laplacian smooth each bone's weight channel over the mesh topology.
#[allow(dead_code)]
pub fn diffuse_weights(weights: &mut [Vec<f32>], adj: &[Vec<usize>], iterations: u32) {
    if weights.is_empty() {
        return;
    }
    let n_bones = weights[0].len();
    let n_verts = weights.len();
    let mut tmp: Vec<Vec<f32>> = vec![vec![0.0; n_bones]; n_verts];

    for _ in 0..iterations {
        for vi in 0..n_verts {
            let neighbors = &adj[vi];
            if neighbors.is_empty() {
                tmp[vi].clone_from(&weights[vi]);
                continue;
            }
            for bi in 0..n_bones {
                let neighbor_avg: f32 = neighbors.iter().map(|&ni| weights[ni][bi]).sum::<f32>()
                    / neighbors.len() as f32;
                // Mix original with neighbor average: 0.5 / 0.5 balance
                tmp[vi][bi] = 0.5 * weights[vi][bi] + 0.5 * neighbor_avg;
            }
        }
        weights.clone_from_slice(&tmp);
    }
}

/// Normalize skin weights so that per-vertex weights sum to 1.
/// If all weights are zero, assign uniform weights.
#[allow(dead_code)]
pub fn normalize_skin_weights(weights: &mut [Vec<f32>]) {
    for row in weights.iter_mut() {
        let sum: f32 = row.iter().sum();
        if sum > 1e-12 {
            for w in row.iter_mut() {
                *w /= sum;
            }
        } else if !row.is_empty() {
            // Uniform fallback
            let uniform = 1.0 / row.len() as f32;
            for w in row.iter_mut() {
                *w = uniform;
            }
        }
    }
}

/// Return the index of the bone with the highest weight.
#[allow(dead_code)]
pub fn dominant_bone(weights: &[f32]) -> usize {
    weights
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Zero out weights below threshold and renormalize.
#[allow(dead_code)]
pub fn prune_small_weights(weights: &mut [Vec<f32>], threshold: f32) {
    for row in weights.iter_mut() {
        for w in row.iter_mut() {
            if *w < threshold {
                *w = 0.0;
            }
        }
    }
    normalize_skin_weights(weights);
}

/// Build adjacency list from triangle mesh.
fn build_adj(n_verts: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    let n_tri = indices.len() / 3;
    for ti in 0..n_tri {
        let a = indices[ti * 3] as usize;
        let b = indices[ti * 3 + 1] as usize;
        let c = indices[ti * 3 + 2] as usize;
        for &(x, y) in &[(a, b), (b, c), (a, c), (b, a), (c, b), (c, a)] {
            if x < n_verts && y < n_verts && !adj[x].contains(&y) {
                adj[x].push(y);
            }
        }
    }
    adj
}

/// Compute automatic skin weights for a mesh from bone endpoints.
#[allow(dead_code)]
pub fn compute_auto_skin_weights(
    positions: &[[f32; 3]],
    indices: &[u32],
    bones: &[BoneEndpoint],
    cfg: &SkinWeightConfig,
) -> AutoSkinResult {
    let n_verts = positions.len();
    let n_bones = bones.len();

    // Initial heat-based weights.
    let mut weights: Vec<Vec<f32>> = positions
        .iter()
        .map(|&p| {
            bones
                .iter()
                .map(|b| bone_heat(p, b, &cfg.falloff))
                .collect()
        })
        .collect();

    // Diffuse weights over mesh topology.
    if cfg.diffusion_iterations > 0 && !indices.is_empty() {
        let adj = build_adj(n_verts, indices);
        diffuse_weights(&mut weights, &adj, cfg.diffusion_iterations);
    }

    // Normalize.
    if cfg.normalize {
        normalize_skin_weights(&mut weights);
    }

    AutoSkinResult {
        bone_count: n_bones,
        vertex_count: n_verts,
        weights,
    }
}

/// Serialize skin weights to a compact JSON string.
#[allow(dead_code)]
pub fn skin_weights_to_json(result: &AutoSkinResult) -> String {
    let mut s = String::from("{\"bone_count\":");
    s.push_str(&result.bone_count.to_string());
    s.push_str(",\"vertex_count\":");
    s.push_str(&result.vertex_count.to_string());
    s.push_str(",\"weights\":[");
    for (i, row) in result.weights.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('[');
        for (j, &w) in row.iter().enumerate() {
            if j > 0 {
                s.push(',');
            }
            s.push_str(&format!("{w:.4}"));
        }
        s.push(']');
    }
    s.push_str("]}");
    s
}

/// Count non-zero weights per vertex (above threshold).
#[allow(dead_code)]
pub fn max_influences_per_vertex(result: &AutoSkinResult, threshold: f32) -> Vec<usize> {
    result
        .weights
        .iter()
        .map(|row| row.iter().filter(|&&w| w > threshold).count())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bone(pos: [f32; 3], radius: f32) -> BoneEndpoint {
        BoneEndpoint {
            name: "test".to_string(),
            position: pos,
            radius,
        }
    }

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3];
        (positions, indices)
    }

    #[test]
    fn bone_heat_at_center() {
        let bone = make_bone([0.0, 0.0, 0.0], 1.0);
        let h = bone_heat([0.0, 0.0, 0.0], &bone, &WeightFalloff::Linear);
        assert!(
            (h - 1.0).abs() < 1e-5,
            "heat at center should be 1.0, got {h}"
        );
    }

    #[test]
    fn bone_heat_at_radius_is_zero() {
        let bone = make_bone([0.0, 0.0, 0.0], 1.0);
        let h = bone_heat([1.0, 0.0, 0.0], &bone, &WeightFalloff::Linear);
        assert!(h < 1e-5, "heat at radius should be ~0, got {h}");
    }

    #[test]
    fn bone_heat_beyond_radius_is_zero() {
        let bone = make_bone([0.0, 0.0, 0.0], 1.0);
        let h = bone_heat([2.0, 0.0, 0.0], &bone, &WeightFalloff::Linear);
        assert!(h == 0.0, "heat beyond radius must be 0, got {h}");
    }

    #[test]
    fn bone_heat_linear_falloff() {
        let bone = make_bone([0.0, 0.0, 0.0], 2.0);
        let h = bone_heat([1.0, 0.0, 0.0], &bone, &WeightFalloff::Linear);
        assert!(
            (h - 0.5).abs() < 1e-5,
            "linear: half-radius => heat 0.5, got {h}"
        );
    }

    #[test]
    fn bone_heat_quadratic_falloff() {
        let bone = make_bone([0.0, 0.0, 0.0], 2.0);
        let h = bone_heat([1.0, 0.0, 0.0], &bone, &WeightFalloff::Quadratic);
        assert!(
            (h - 0.25).abs() < 1e-5,
            "quadratic: half-radius => heat 0.25, got {h}"
        );
    }

    #[test]
    fn bone_heat_gaussian_falloff_center() {
        let bone = make_bone([0.0, 0.0, 0.0], 1.0);
        let h = bone_heat([0.0, 0.0, 0.0], &bone, &WeightFalloff::Gaussian);
        assert!(h > 0.0, "gaussian at center should be positive, got {h}");
    }

    #[test]
    fn normalize_skin_weights_sums_to_one() {
        let mut weights = vec![vec![3.0f32, 1.0, 0.0], vec![0.0, 2.0, 2.0]];
        normalize_skin_weights(&mut weights);
        for row in &weights {
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-5, "sum should be 1.0, got {sum}");
        }
    }

    #[test]
    fn normalize_skin_weights_all_zero_uniform() {
        let mut weights = vec![vec![0.0f32, 0.0, 0.0]];
        normalize_skin_weights(&mut weights);
        for &w in &weights[0] {
            assert!(
                (w - 1.0 / 3.0).abs() < 1e-5,
                "all-zero should give uniform, got {w}"
            );
        }
    }

    #[test]
    fn dominant_bone_argmax() {
        let weights = vec![0.1f32, 0.6, 0.3];
        assert_eq!(
            dominant_bone(&weights),
            1,
            "dominant bone should be index 1"
        );
    }

    #[test]
    fn prune_small_weights_removes_tiny() {
        let mut weights = vec![vec![0.001f32, 0.5, 0.499]];
        prune_small_weights(&mut weights, 0.01);
        assert!(weights[0][0] < 1e-9, "tiny weight should be zeroed");
        let sum: f32 = weights[0].iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "after prune, should still sum to 1"
        );
    }

    #[test]
    fn compute_auto_skin_weights_shape() {
        let (pos, idx) = simple_mesh();
        let bones = vec![
            make_bone([0.0, 0.0, 0.0], 2.0),
            make_bone([1.0, 1.0, 1.0], 2.0),
        ];
        let cfg = SkinWeightConfig {
            diffusion_iterations: 5,
            ..Default::default()
        };
        let result = compute_auto_skin_weights(&pos, &idx, &bones, &cfg);
        assert_eq!(result.vertex_count, pos.len(), "vertex_count mismatch");
        assert_eq!(result.weights.len(), pos.len(), "weights rows mismatch");
        for row in &result.weights {
            assert_eq!(row.len(), 2, "each row should have 2 bone weights");
        }
    }

    #[test]
    fn compute_auto_skin_weights_bone_count() {
        let (pos, idx) = simple_mesh();
        let bones = vec![make_bone([0.0, 0.0, 0.0], 2.0)];
        let cfg = SkinWeightConfig::default();
        let result = compute_auto_skin_weights(&pos, &idx, &bones, &cfg);
        assert_eq!(result.bone_count, 1, "bone_count should match bones arg");
    }

    #[test]
    fn compute_auto_skin_weights_vertex_count() {
        let (pos, idx) = simple_mesh();
        let bones = vec![make_bone([0.0, 0.0, 0.0], 2.0)];
        let cfg = SkinWeightConfig::default();
        let result = compute_auto_skin_weights(&pos, &idx, &bones, &cfg);
        assert_eq!(
            result.vertex_count,
            pos.len(),
            "vertex_count should match positions"
        );
    }

    #[test]
    fn max_influences_per_vertex_counts() {
        let result = AutoSkinResult {
            weights: vec![
                vec![0.8, 0.2, 0.0],
                vec![0.0, 0.5, 0.5],
                vec![0.0, 0.0, 0.0],
            ],
            bone_count: 3,
            vertex_count: 3,
        };
        let influences = max_influences_per_vertex(&result, 0.01);
        assert_eq!(influences[0], 2, "vertex 0 has 2 influences above 0.01");
        assert_eq!(influences[1], 2, "vertex 1 has 2 influences above 0.01");
        assert_eq!(influences[2], 0, "vertex 2 has 0 influences above 0.01");
    }
}
