#![allow(dead_code)]
//! Blend masks for per-vertex morph weight control.

/// Describes a named mask region.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaskRegion {
    /// Region name.
    pub name: String,
    /// Start vertex index.
    pub start: usize,
    /// End vertex index (exclusive).
    pub end: usize,
}

/// A per-vertex blend mask.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendMask {
    /// Per-vertex weights in [0, 1].
    pub weights: Vec<f32>,
    /// Named regions.
    pub regions: Vec<MaskRegion>,
}

/// Create a new [`BlendMask`] with all weights set to 1.0.
#[allow(dead_code)]
pub fn new_blend_mask(vertex_count: usize) -> BlendMask {
    BlendMask {
        weights: vec![1.0; vertex_count],
        regions: Vec::new(),
    }
}

/// Set a weight for a specific vertex.
#[allow(dead_code)]
pub fn set_mask_weight(mask: &mut BlendMask, vertex: usize, weight: f32) {
    if let Some(w) = mask.weights.get_mut(vertex) {
        *w = weight.clamp(0.0, 1.0);
    }
}

/// Get the weight for a specific vertex.
#[allow(dead_code)]
pub fn get_mask_weight(mask: &BlendMask, vertex: usize) -> f32 {
    mask.weights.get(vertex).copied().unwrap_or(0.0)
}

/// Return the vertex count of the mask.
#[allow(dead_code)]
pub fn mask_vertex_count(mask: &BlendMask) -> usize {
    mask.weights.len()
}

/// Apply the blend mask to a delta array, scaling each delta by the mask weight.
#[allow(dead_code)]
pub fn apply_blend_mask(mask: &BlendMask, deltas: &mut [[f32; 3]]) {
    for (i, d) in deltas.iter_mut().enumerate() {
        let w = mask.weights.get(i).copied().unwrap_or(0.0);
        d[0] *= w;
        d[1] *= w;
        d[2] *= w;
    }
}

/// Invert the mask (1.0 - weight for each vertex).
#[allow(dead_code)]
pub fn invert_mask(mask: &mut BlendMask) {
    for w in &mut mask.weights {
        *w = 1.0 - *w;
    }
}

/// Compute the union of two masks (max of each weight).
#[allow(dead_code)]
pub fn mask_union(a: &BlendMask, b: &BlendMask) -> BlendMask {
    let len = a.weights.len().max(b.weights.len());
    let mut weights = Vec::with_capacity(len);
    for i in 0..len {
        let wa = a.weights.get(i).copied().unwrap_or(0.0);
        let wb = b.weights.get(i).copied().unwrap_or(0.0);
        weights.push(wa.max(wb));
    }
    BlendMask {
        weights,
        regions: Vec::new(),
    }
}

/// Serialize mask weights to bytes (little-endian f32).
#[allow(dead_code)]
pub fn mask_to_bytes(mask: &BlendMask) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(mask.weights.len() * 4);
    for &w in &mask.weights {
        bytes.extend_from_slice(&w.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_blend_mask() {
        let m = new_blend_mask(10);
        assert_eq!(mask_vertex_count(&m), 10);
        assert!((get_mask_weight(&m, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_get_mask_weight() {
        let mut m = new_blend_mask(5);
        set_mask_weight(&mut m, 2, 0.5);
        assert!((get_mask_weight(&m, 2) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_mask_weight_clamps() {
        let mut m = new_blend_mask(5);
        set_mask_weight(&mut m, 0, 2.0);
        assert!((get_mask_weight(&m, 0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_get_mask_weight_out_of_bounds() {
        let m = new_blend_mask(2);
        assert!((get_mask_weight(&m, 99) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_blend_mask() {
        let mut m = new_blend_mask(2);
        set_mask_weight(&mut m, 0, 0.5);
        set_mask_weight(&mut m, 1, 0.0);
        let mut deltas = [[2.0, 4.0, 6.0], [1.0, 1.0, 1.0]];
        apply_blend_mask(&m, &mut deltas);
        assert!((deltas[0][0] - 1.0).abs() < 1e-6);
        assert!((deltas[1][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert_mask() {
        let mut m = new_blend_mask(3);
        set_mask_weight(&mut m, 0, 0.3);
        invert_mask(&mut m);
        assert!((get_mask_weight(&m, 0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mask_union() {
        let mut a = new_blend_mask(3);
        let mut b = new_blend_mask(3);
        set_mask_weight(&mut a, 0, 0.2);
        set_mask_weight(&mut b, 0, 0.8);
        let result = mask_union(&a, &b);
        assert!((get_mask_weight(&result, 0) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_mask_to_bytes() {
        let m = new_blend_mask(2);
        let bytes = mask_to_bytes(&m);
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn test_empty_mask() {
        let m = new_blend_mask(0);
        assert_eq!(mask_vertex_count(&m), 0);
    }

    #[test]
    fn test_mask_union_different_sizes() {
        let a = new_blend_mask(2);
        let b = new_blend_mask(4);
        let result = mask_union(&a, &b);
        assert_eq!(mask_vertex_count(&result), 4);
    }
}
