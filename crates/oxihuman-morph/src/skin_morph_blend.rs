//! Blend skin deformation and morph target deformation with per-vertex influence weights.
//!
//! Each vertex carries a blend weight that determines the ratio of skin
//! deformation versus morph target deformation applied at runtime.

#![allow(dead_code)]

/// Configuration for the skin–morph blend system.
#[derive(Debug, Clone)]
pub struct SkinMorphBlendConfig {
    /// Global skin deformation weight (0.0–1.0).
    pub global_skin_weight: f32,
    /// Global morph target weight (0.0–1.0).
    pub global_morph_weight: f32,
    /// Maximum number of vertices supported.
    pub max_vertices: usize,
}

/// A single vertex with skin and morph displacement plus blend weights.
#[derive(Debug, Clone)]
pub struct SkinMorphVertex {
    /// Position contributed by skinning (bone-driven deformation).
    pub skin_pos: [f32; 3],
    /// Position contributed by morph target.
    pub morph_pos: [f32; 3],
    /// Per-vertex skin influence weight (0.0–1.0).
    pub skin_weight: f32,
    /// Per-vertex morph influence weight (0.0–1.0).
    pub morph_weight: f32,
}

/// Result of a skin–morph blend operation.
#[derive(Debug, Clone)]
pub struct SkinMorphBlendResult {
    /// Final blended positions for each vertex.
    pub positions: Vec<[f32; 3]>,
    /// Number of vertices processed.
    pub vertex_count: usize,
}

/// Manages the collection of vertices for blending.
#[derive(Debug, Clone)]
pub struct SkinMorphBlend {
    config: SkinMorphBlendConfig,
    vertices: Vec<SkinMorphVertex>,
}

/// Build a default `SkinMorphBlendConfig`.
#[allow(dead_code)]
pub fn default_skin_morph_blend_config() -> SkinMorphBlendConfig {
    SkinMorphBlendConfig {
        global_skin_weight: 1.0,
        global_morph_weight: 1.0,
        max_vertices: 65536,
    }
}

/// Create a new `SkinMorphBlend`.
#[allow(dead_code)]
pub fn new_skin_morph_blend(config: SkinMorphBlendConfig) -> SkinMorphBlend {
    SkinMorphBlend {
        config,
        vertices: Vec::new(),
    }
}

/// Add a vertex with skin/morph positions and per-vertex weights.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn smb_add_vertex(
    smb: &mut SkinMorphBlend,
    skin_pos: [f32; 3],
    morph_pos: [f32; 3],
    skin_weight: f32,
    morph_weight: f32,
) {
    if smb.vertices.len() >= smb.config.max_vertices {
        return;
    }
    smb.vertices.push(SkinMorphVertex {
        skin_pos,
        morph_pos,
        skin_weight: skin_weight.clamp(0.0, 1.0),
        morph_weight: morph_weight.clamp(0.0, 1.0),
    });
}

/// Blend all vertices and return the result.
#[allow(dead_code)]
pub fn smb_blend(smb: &SkinMorphBlend) -> SkinMorphBlendResult {
    let gsk = smb.config.global_skin_weight;
    let gmk = smb.config.global_morph_weight;
    let positions: Vec<[f32; 3]> = smb
        .vertices
        .iter()
        .map(|v| {
            let sw = v.skin_weight * gsk;
            let mw = v.morph_weight * gmk;
            let total = sw + mw;
            if total <= f32::EPSILON {
                v.skin_pos
            } else {
                let ns = sw / total;
                let nm = mw / total;
                [
                    v.skin_pos[0] * ns + v.morph_pos[0] * nm,
                    v.skin_pos[1] * ns + v.morph_pos[1] * nm,
                    v.skin_pos[2] * ns + v.morph_pos[2] * nm,
                ]
            }
        })
        .collect();
    let vertex_count = positions.len();
    SkinMorphBlendResult {
        positions,
        vertex_count,
    }
}

/// Return the number of vertices stored.
#[allow(dead_code)]
pub fn smb_vertex_count(smb: &SkinMorphBlend) -> usize {
    smb.vertices.len()
}

/// Return the skin weight for vertex at `index`, or 0.0 if out of range.
#[allow(dead_code)]
pub fn smb_skin_weight(smb: &SkinMorphBlend, index: usize) -> f32 {
    smb.vertices.get(index).map(|v| v.skin_weight).unwrap_or(0.0)
}

/// Return the morph weight for vertex at `index`, or 0.0 if out of range.
#[allow(dead_code)]
pub fn smb_morph_weight(smb: &SkinMorphBlend, index: usize) -> f32 {
    smb.vertices.get(index).map(|v| v.morph_weight).unwrap_or(0.0)
}

/// Serialize the blend state to a JSON string.
#[allow(dead_code)]
pub fn smb_to_json(smb: &SkinMorphBlend) -> String {
    format!(
        "{{\"vertex_count\":{},\"global_skin_weight\":{},\"global_morph_weight\":{}}}",
        smb.vertices.len(),
        smb.config.global_skin_weight,
        smb.config.global_morph_weight
    )
}

/// Remove all vertices.
#[allow(dead_code)]
pub fn smb_clear(smb: &mut SkinMorphBlend) {
    smb.vertices.clear();
}

/// Reset all per-vertex weights to 0.5 / 0.5.
#[allow(dead_code)]
pub fn smb_reset_weights(smb: &mut SkinMorphBlend) {
    for v in &mut smb.vertices {
        v.skin_weight = 0.5;
        v.morph_weight = 0.5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_smb() -> SkinMorphBlend {
        new_skin_morph_blend(default_skin_morph_blend_config())
    }

    #[test]
    fn test_add_vertex_and_count() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0, 0.0);
        assert_eq!(smb_vertex_count(&smb), 1);
    }

    #[test]
    fn test_blend_full_skin_weight() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [1.0, 2.0, 3.0], [9.0, 9.0, 9.0], 1.0, 0.0);
        let result = smb_blend(&smb);
        assert!((result.positions[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_full_morph_weight() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0, 0.0, 0.0], [5.0, 5.0, 5.0], 0.0, 1.0);
        let result = smb_blend(&smb);
        assert!((result.positions[0][0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_equal_weights() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.5, 0.5);
        let result = smb_blend(&smb);
        assert!((result.positions[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_skin_weight_accessor() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 0.7, 0.3);
        assert!((smb_skin_weight(&smb, 0) - 0.7).abs() < 1e-5);
        assert_eq!(smb_skin_weight(&smb, 99), 0.0);
    }

    #[test]
    fn test_morph_weight_accessor() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 0.2, 0.8);
        assert!((smb_morph_weight(&smb, 0) - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 1.0, 0.0);
        smb_clear(&mut smb);
        assert_eq!(smb_vertex_count(&smb), 0);
    }

    #[test]
    fn test_reset_weights() {
        let mut smb = make_smb();
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 1.0, 0.0);
        smb_reset_weights(&mut smb);
        assert!((smb_skin_weight(&smb, 0) - 0.5).abs() < 1e-5);
        assert!((smb_morph_weight(&smb, 0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json_contains_vertex_count() {
        let smb = make_smb();
        let json = smb_to_json(&smb);
        assert!(json.contains("vertex_count"));
    }

    #[test]
    fn test_max_vertices_enforced() {
        let cfg = SkinMorphBlendConfig {
            global_skin_weight: 1.0,
            global_morph_weight: 1.0,
            max_vertices: 2,
        };
        let mut smb = new_skin_morph_blend(cfg);
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 1.0, 0.0);
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 1.0, 0.0);
        smb_add_vertex(&mut smb, [0.0; 3], [0.0; 3], 1.0, 0.0); // ignored
        assert_eq!(smb_vertex_count(&smb), 2);
    }
}
