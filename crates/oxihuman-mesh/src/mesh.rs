// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use oxihuman_morph::engine::MeshBuffers as MorphMeshBuffers;

/// Final mesh buffers ready for export or GPU upload.
///
/// Constructed from [`oxihuman_morph::engine::MeshBuffers`] via
/// [`MeshBuffers::from_morph`], which initialises tangents to a default value.
/// Call [`crate::normals::compute_normals`] and
/// [`crate::normals::compute_tangents`] on the result before exporting
/// to formats that require correct lighting vectors.
///
/// # Safety flag
///
/// `has_suit` must be `true` before calling GLB/GLTF exporters.  Set it after
/// applying the suit mesh via [`crate::suit::ensure_suit_mesh`].
///
/// # Layout
///
/// All per-vertex arrays (`positions`, `normals`, `tangents`, `uvs`, `colors`)
/// share the same length. `indices` is a flat triangle list (length % 3 == 0).
#[derive(Debug, Clone)]
pub struct MeshBuffers {
    /// Per-vertex XYZ positions in world space.
    pub positions: Vec<[f32; 3]>,
    /// Per-vertex unit normals (XYZ).
    pub normals: Vec<[f32; 3]>,
    /// Per-vertex tangents (XYZW, W = bitangent sign: ±1).
    pub tangents: Vec<[f32; 4]>,
    /// Per-vertex UV texture coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Triangle index list (groups of 3, CCW winding).
    pub indices: Vec<u32>,
    /// Optional per-vertex RGBA color (each channel in 0..1).
    pub colors: Option<Vec<[f32; 4]>>,
    /// Safety flag: exporters refuse to write dressed formats if `false`.
    pub has_suit: bool,
}

impl MeshBuffers {
    /// Create from the morph engine's raw output.
    pub fn from_morph(src: MorphMeshBuffers) -> Self {
        let tangents = vec![[1.0f32, 0.0, 0.0, 1.0]; src.positions.len()];
        MeshBuffers {
            positions: src.positions,
            normals: src.normals,
            tangents,
            uvs: src.uvs,
            indices: src.indices,
            colors: None,
            has_suit: src.has_suit,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn sample_morph_mesh() -> MB {
        MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        }
    }

    #[test]
    fn from_morph_preserves_data() {
        let m = MeshBuffers::from_morph(sample_morph_mesh());
        assert_eq!(m.vertex_count(), 3);
        assert_eq!(m.face_count(), 1);
        assert_eq!(m.tangents.len(), 3);
    }
}
