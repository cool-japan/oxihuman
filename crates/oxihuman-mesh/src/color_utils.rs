// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Uniform vertex-color helpers.

use crate::mesh::MeshBuffers;

/// Set a uniform RGBA color on every vertex of the mesh.
///
/// After calling this, `mesh.colors` will be `Some(vec![rgba; n_verts])`.
/// Existing color data (if any) is replaced.
pub fn set_uniform_color(mesh: &mut MeshBuffers, rgba: [f32; 4]) {
    let n = mesh.positions.len();
    mesh.colors = Some(vec![rgba; n]);
}

#[cfg(test)]
mod colors_tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn three_vert_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    #[test]
    fn set_uniform_color_sets_all_verts() {
        let mut mesh = three_vert_mesh();
        set_uniform_color(&mut mesh, [1.0, 0.5, 0.0, 1.0]);
        let colors = mesh.colors.as_ref().expect("colors should be Some");
        assert_eq!(colors.len(), mesh.positions.len());
    }

    #[test]
    fn color_rgba_values_correct() {
        let mut mesh = three_vert_mesh();
        let rgba = [0.2f32, 0.4, 0.6, 0.8];
        set_uniform_color(&mut mesh, rgba);
        let colors = mesh.colors.as_ref().expect("colors should be Some");
        for c in colors {
            assert!((c[0] - 0.2).abs() < 1e-6, "R mismatch");
            assert!((c[1] - 0.4).abs() < 1e-6, "G mismatch");
            assert!((c[2] - 0.6).abs() < 1e-6, "B mismatch");
            assert!((c[3] - 0.8).abs() < 1e-6, "A mismatch");
        }
    }

    #[test]
    fn no_color_mesh_colors_none() {
        let mesh = three_vert_mesh();
        assert!(
            mesh.colors.is_none(),
            "from_morph should produce colors=None"
        );
    }
}
