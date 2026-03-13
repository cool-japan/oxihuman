//! Cloth blending and wrapping morphs for body-aware cloth deformation.

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ClothBlendConfig {
    /// Minimum distance to push cloth from body surface.
    pub min_offset: f32,
    /// Maximum push distance for collision resolution.
    pub max_push: f32,
    /// Number of Laplacian smoothing iterations.
    pub smooth_iterations: u32,
    /// Blend weight threshold below which a layer is ignored.
    pub weight_threshold: f32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ClothLayer {
    /// Unique layer identifier.
    pub id: u32,
    /// Blend weight [0..1].
    pub weight: f32,
    /// Vertex positions (each vertex is [x, y, z]).
    pub vertices: Vec<[f32; 3]>,
    /// Rest-pose vertex positions.
    pub rest_vertices: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ClothBlendResult {
    /// Blended vertex positions.
    pub vertices: Vec<[f32; 3]>,
    /// Per-vertex blend energy (stretch measure).
    pub energy: Vec<f32>,
    /// Total blend energy sum.
    pub total_energy: f32,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Return a default `ClothBlendConfig`.
#[allow(dead_code)]
pub fn default_cloth_blend_config() -> ClothBlendConfig {
    ClothBlendConfig {
        min_offset: 0.002,
        max_push: 0.05,
        smooth_iterations: 3,
        weight_threshold: 1e-4,
    }
}

/// Create a new `ClothLayer` with the given vertices (used as both current and rest).
#[allow(dead_code)]
pub fn new_cloth_layer(id: u32, vertices: Vec<[f32; 3]>) -> ClothLayer {
    ClothLayer {
        id,
        weight: 1.0,
        rest_vertices: vertices.clone(),
        vertices,
    }
}

// ---------------------------------------------------------------------------
// Blend operations
// ---------------------------------------------------------------------------

/// Weighted blend of multiple cloth layer vertex positions.
/// Layers with weight below `config.weight_threshold` are skipped.
/// Returns the blended vertex array; returns empty if no layers or vertex mismatch.
#[allow(dead_code)]
pub fn blend_cloth_layers(layers: &[ClothLayer], config: &ClothBlendConfig) -> Vec<[f32; 3]> {
    if layers.is_empty() {
        return Vec::new();
    }
    let n = layers[0].vertices.len();
    let mut out = vec![[0.0_f32; 3]; n];
    let mut weight_sum = 0.0_f32;

    for layer in layers {
        if layer.weight < config.weight_threshold {
            continue;
        }
        if layer.vertices.len() != n {
            continue;
        }
        for (o, v) in out.iter_mut().zip(layer.vertices.iter()) {
            o[0] += v[0] * layer.weight;
            o[1] += v[1] * layer.weight;
            o[2] += v[2] * layer.weight;
        }
        weight_sum += layer.weight;
    }

    if weight_sum > 1e-8 {
        let inv = 1.0 / weight_sum;
        for o in &mut out {
            o[0] *= inv;
            o[1] *= inv;
            o[2] *= inv;
        }
    }
    out
}

/// Push each cloth vertex outward from the body centre by `offset` units.
/// `centre` is the body origin used as reference.
#[allow(dead_code)]
pub fn apply_body_offset(vertices: &mut [[f32; 3]], centre: [f32; 3], offset: f32) {
    for v in vertices.iter_mut() {
        let dx = v[0] - centre[0];
        let dy = v[1] - centre[1];
        let dz = v[2] - centre[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt();
        if len > 1e-8 {
            let scale = offset / len;
            v[0] += dx * scale;
            v[1] += dy * scale;
            v[2] += dz * scale;
        }
    }
}

/// Simple sphere-capsule collision push: push cloth vertices outside the
/// sphere defined by `centre` and `radius`.
#[allow(dead_code)]
pub fn cloth_collision_push(
    vertices: &mut [[f32; 3]],
    centre: [f32; 3],
    radius: f32,
    max_push: f32,
) {
    for v in vertices.iter_mut() {
        let dx = v[0] - centre[0];
        let dy = v[1] - centre[1];
        let dz = v[2] - centre[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < radius && dist > 1e-8 {
            let push = (radius - dist).min(max_push);
            let inv = 1.0 / dist;
            v[0] += dx * inv * push;
            v[1] += dy * inv * push;
            v[2] += dz * inv * push;
        }
    }
}

/// Laplacian smooth blend weights across a cloth layer using a ring adjacency
/// approximation (treats each vertex as connected to its two neighbours in order).
#[allow(dead_code)]
pub fn smooth_cloth_blend(weights: &[f32], iterations: u32) -> Vec<f32> {
    if weights.is_empty() {
        return Vec::new();
    }
    let n = weights.len();
    let mut cur = weights.to_vec();
    let mut tmp = vec![0.0_f32; n];
    for _ in 0..iterations {
        for i in 0..n {
            let prev = if i == 0 { n - 1 } else { i - 1 };
            let next = if i + 1 == n { 0 } else { i + 1 };
            tmp[i] = (cur[prev] + cur[i] + cur[next]) / 3.0;
        }
        cur.copy_from_slice(&tmp);
    }
    cur
}

// ---------------------------------------------------------------------------
// Layer accessors
// ---------------------------------------------------------------------------

/// Return the number of layers.
#[allow(dead_code)]
pub fn cloth_layer_count(layers: &[ClothLayer]) -> usize {
    layers.len()
}

/// Compute per-vertex stretch energy as Euclidean distance from rest positions.
/// Returns a `ClothBlendResult`.
#[allow(dead_code)]
pub fn cloth_blend_energy(layer: &ClothLayer) -> ClothBlendResult {
    let n = layer.vertices.len().min(layer.rest_vertices.len());
    let mut energy = Vec::with_capacity(n);
    let mut total = 0.0_f32;
    for i in 0..n {
        let v = &layer.vertices[i];
        let r = &layer.rest_vertices[i];
        let e = {
            let dx = v[0] - r[0];
            let dy = v[1] - r[1];
            let dz = v[2] - r[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        };
        energy.push(e);
        total += e;
    }
    ClothBlendResult {
        vertices: layer.vertices.clone(),
        energy,
        total_energy: total,
    }
}

/// Normalize blend weights in a layer slice so that they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_cloth_weights(layers: &mut [ClothLayer]) {
    let sum: f32 = layers.iter().map(|l| l.weight).sum();
    if sum > 1e-8 {
        for l in layers.iter_mut() {
            l.weight /= sum;
        }
    }
}

/// Set the weight of a specific layer by id.
#[allow(dead_code)]
pub fn set_layer_weight(layers: &mut [ClothLayer], id: u32, weight: f32) {
    for l in layers.iter_mut() {
        if l.id == id {
            l.weight = weight.clamp(0.0, 1.0);
            return;
        }
    }
}

/// Get the weight of a specific layer by id.  Returns `None` if not found.
#[allow(dead_code)]
pub fn get_layer_weight(layers: &[ClothLayer], id: u32) -> Option<f32> {
    layers.iter().find(|l| l.id == id).map(|l| l.weight)
}

/// Reset all vertices in the layer to rest positions.
#[allow(dead_code)]
pub fn cloth_to_rest(layer: &mut ClothLayer) {
    layer.vertices = layer.rest_vertices.clone();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_layer(id: u32, n: usize, weight: f32) -> ClothLayer {
        let verts: Vec<[f32; 3]> = (0..n).map(|i| [i as f32 * 0.1, 0.0, 0.0]).collect();
        let mut l = new_cloth_layer(id, verts);
        l.weight = weight;
        l
    }

    #[test]
    fn test_default_cloth_blend_config() {
        let c = default_cloth_blend_config();
        assert!(c.min_offset >= 0.0);
        assert!(c.max_push > 0.0);
        assert!(c.smooth_iterations > 0);
    }

    #[test]
    fn test_new_cloth_layer() {
        let l = new_cloth_layer(0, vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert_eq!(l.vertices.len(), 2);
        assert_eq!(l.rest_vertices.len(), 2);
        assert_eq!(l.id, 0);
        assert!((l.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_cloth_layers_single() {
        let cfg = default_cloth_blend_config();
        let l = make_layer(0, 3, 1.0);
        let blended = blend_cloth_layers(std::slice::from_ref(&l), &cfg);
        assert_eq!(blended.len(), 3);
        assert!((blended[0][0] - l.vertices[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_blend_cloth_layers_average() {
        let cfg = default_cloth_blend_config();
        let mut la = make_layer(0, 1, 1.0);
        let mut lb = make_layer(1, 1, 1.0);
        la.vertices[0][0] = 0.0;
        lb.vertices[0][0] = 2.0;
        let blended = blend_cloth_layers(&[la, lb], &cfg);
        assert!((blended[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_cloth_layers_empty() {
        let cfg = default_cloth_blend_config();
        let blended = blend_cloth_layers(&[], &cfg);
        assert!(blended.is_empty());
    }

    #[test]
    fn test_blend_cloth_layers_skips_low_weight() {
        let cfg = default_cloth_blend_config();
        let mut la = make_layer(0, 1, 1.0);
        let mut lb = make_layer(1, 1, 0.0); // below threshold
        la.vertices[0][0] = 0.0;
        lb.vertices[0][0] = 10.0;
        let blended = blend_cloth_layers(&[la, lb], &cfg);
        assert!(
            blended[0][0].abs() < 1e-5,
            "zero-weight layer should be skipped"
        );
    }

    #[test]
    fn test_apply_body_offset() {
        let mut verts = vec![[1.0_f32, 0.0, 0.0]];
        apply_body_offset(&mut verts, [0.0, 0.0, 0.0], 0.1);
        assert!(verts[0][0] > 1.0, "vertex should be pushed outward");
    }

    #[test]
    fn test_cloth_collision_push() {
        let mut verts = vec![[0.5_f32, 0.0, 0.0]]; // inside sphere of r=1
        cloth_collision_push(&mut verts, [0.0, 0.0, 0.0], 1.0, 1.0);
        assert!(
            verts[0][0] >= 1.0 - 1e-4,
            "should be pushed to sphere surface"
        );
    }

    #[test]
    fn test_cloth_collision_push_outside() {
        let mut verts = vec![[2.0_f32, 0.0, 0.0]]; // already outside sphere
        cloth_collision_push(&mut verts, [0.0, 0.0, 0.0], 1.0, 1.0);
        assert!(
            (verts[0][0] - 2.0).abs() < 1e-6,
            "vertex outside sphere should not move"
        );
    }

    #[test]
    fn test_smooth_cloth_blend_uniform() {
        let weights = vec![0.5_f32; 5];
        let smoothed = smooth_cloth_blend(&weights, 2);
        assert_eq!(smoothed.len(), 5);
        for w in &smoothed {
            assert!((w - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn test_smooth_cloth_blend_empty() {
        let smoothed = smooth_cloth_blend(&[], 3);
        assert!(smoothed.is_empty());
    }

    #[test]
    fn test_cloth_layer_count() {
        let layers: Vec<ClothLayer> = (0..4).map(|i| make_layer(i, 2, 1.0)).collect();
        assert_eq!(cloth_layer_count(&layers), 4);
    }

    #[test]
    fn test_cloth_blend_energy_rest() {
        let l = new_cloth_layer(0, vec![[0.0, 0.0, 0.0]; 3]);
        let res = cloth_blend_energy(&l);
        assert!(res.total_energy < 1e-6, "energy should be zero at rest");
    }

    #[test]
    fn test_cloth_blend_energy_displaced() {
        let mut l = new_cloth_layer(0, vec![[0.0, 0.0, 0.0]]);
        l.vertices[0][0] = 1.0;
        let res = cloth_blend_energy(&l);
        assert!((res.total_energy - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_cloth_weights() {
        let mut layers: Vec<ClothLayer> = (0..3).map(|i| make_layer(i, 1, 2.0)).collect();
        normalize_cloth_weights(&mut layers[..]);
        let sum: f32 = layers.iter().map(|l| l.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_set_get_layer_weight() {
        let mut layers = vec![make_layer(5, 1, 0.5)];
        set_layer_weight(&mut layers[..], 5, 0.8);
        assert!((get_layer_weight(&layers, 5).expect("should succeed") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_get_layer_weight_missing() {
        let layers: Vec<ClothLayer> = vec![];
        assert!(get_layer_weight(&layers, 99).is_none());
    }

    #[test]
    fn test_cloth_to_rest() {
        let mut l = new_cloth_layer(0, vec![[0.0, 0.0, 0.0]]);
        l.vertices[0][0] = 5.0;
        cloth_to_rest(&mut l);
        assert!((l.vertices[0][0] - 0.0).abs() < 1e-6);
    }
}
