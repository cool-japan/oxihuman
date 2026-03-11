// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::mesh::MeshBuffers;
use anyhow::{bail, Result};

/// Safety function: ensure the mesh has a suit layer applied.
/// Returns Err if `has_suit` is false — exporters call this before writing.
pub fn ensure_suit_mesh(buf: &MeshBuffers) -> Result<()> {
    if !buf.has_suit {
        bail!("mesh does not have a suit mesh applied — export refused for safety");
    }
    Ok(())
}

/// Mark a mesh as having the suit applied (after suit geometry is integrated).
pub fn apply_suit_flag(buf: &mut MeshBuffers) {
    buf.has_suit = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn bare_mesh() -> MyMesh {
        MyMesh::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0]],
            normals: vec![[0.0, 1.0, 0.0]],
            uvs: vec![[0.0, 0.0]],
            indices: vec![],
            has_suit: false,
        })
    }

    #[test]
    fn bare_mesh_fails_suit_check() {
        let m = bare_mesh();
        assert!(ensure_suit_mesh(&m).is_err());
    }

    #[test]
    fn suit_applied_passes_check() {
        let mut m = bare_mesh();
        apply_suit_flag(&mut m);
        assert!(ensure_suit_mesh(&m).is_ok());
    }
}
