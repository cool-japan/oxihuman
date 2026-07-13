// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Centralised bodysuit export gate.
//!
//! Every human-facing exporter entry point in this crate MUST call
//! [`ensure_export_allowed`] before serialising a
//! [`oxihuman_mesh::MeshBuffers`].  The gate reuses
//! [`oxihuman_mesh::suit::ensure_suit_mesh`] semantics: a mesh whose
//! `has_suit` flag is `false` is refused with a clear error so that an
//! unclothed human mesh can never be written to disk or returned to callers.
//!
//! # Gated entry points (invariant surface)
//!
//! The complete list of gated, human-facing exporter entry points.  The
//! invariant test-suite iterates this list — keep it in sync when adding
//! exporters:
//!
//! | Module | Gated entry points |
//! |---|---|
//! | `glb.rs` | [`crate::glb::build_glb_bytes`], [`crate::glb::export_glb`], [`crate::glb::export_glb_with_material`], [`crate::glb::build_glb_with_meta_bytes`], [`crate::glb::export_glb_with_meta`], [`crate::glb::build_glb_with_skeleton_bytes`], [`crate::glb::export_glb_with_skeleton`] |
//! | `gltf_sep.rs` | [`crate::gltf_sep::export_gltf_sep`] |
//! | `obj.rs` | [`crate::obj::mesh_to_obj_string`], [`crate::obj::export_obj`] |
//! | `obj_mtl.rs` | [`crate::obj_mtl::export_obj_mtl`] |
//! | `stl.rs` | [`crate::stl::mesh_to_stl_ascii`], [`crate::stl::export_stl_ascii`], [`crate::stl::encode_stl_binary`], [`crate::stl::export_stl_binary`] |
//! | `ascii_stl_export.rs` | [`crate::ascii_stl_export::export_ascii_stl_mesh`] |
//! | `binary_stl_export.rs` | [`crate::binary_stl_export::encode_binary_stl_mesh`] |
//! | `vrm_export.rs` | [`crate::vrm_export::VrmExporter::set_mesh_buffers`] (gate at the `MeshBuffers` boundary; `VrmExporter::export` re-checks the recorded verdict) |
//! | `collada.rs` | [`crate::collada::export_collada`], [`crate::collada::export_collada_scene`] |
//! | `usd.rs` | [`crate::usd::export_usda`], [`crate::usd::export_usda_scene`] |
//! | `usd_anim.rs` | [`crate::usd_anim::export_usda_animated`] |
//! | `fmt_3mf.rs` | [`crate::fmt_3mf::export_3mf`] |
//! | `ply.rs` | [`crate::ply::export_ply`], [`crate::ply::export_mesh_as_point_cloud`] |
//! | `fbx_binary.rs` | [`crate::fbx_binary::export_mesh_fbx_binary`] |
//! | `x3d.rs` | [`crate::x3d::export_x3d`], [`crate::x3d::export_x3d_scene`], [`crate::x3d::build_x3d`], [`crate::x3d::build_x3d_scene`] |
//! | `alembic_ogawa_io.rs` | [`crate::AlembicWriter::from_mesh_buffers`], [`crate::AlembicWriter::from_mesh_sequence`] |
//! | `lod_export.rs` | [`crate::lod_export::export_lod_pack`], [`crate::lod_export::export_lod_pack_with_stats`] |
//! | `auto_export.rs` | [`crate::export_auto`], [`crate::export_with_options`] (single gate in the internal router) |
//!
//! Raw-slice encoders (e.g. `ascii_stl_export::render_ascii_stl`,
//! `binary_stl_export::mesh_to_binary_stl`, `VrmExporter::set_mesh`,
//! `ply::export_point_cloud_ply`) accept arbitrary geometry (bare position /
//! normal / color slices) with no human-mesh provenance and therefore cannot
//! check `has_suit`; they are low-level building blocks. Human meshes must
//! always flow through one of the `MeshBuffers`-taking entry points above —
//! e.g. `ply::export_mesh_as_point_cloud` (gated) wraps
//! `ply::export_point_cloud_ply` (ungated) for exactly this reason.
//!
//! `three_mf_export.rs` (a second, more feature-complete 3MF exporter with its
//! own gated `ThreeMfExporter::add_mesh_buffers_object`) was found to be a
//! **phantom module**: it was never `mod`-declared in `lib.rs` / any
//! `_export_partN.rs`, so it was never compiled and has been deleted. The only
//! compiled 3MF path is `fmt_3mf::export_3mf`, gated above.
//!
//! `x3d::build_x3d` / `x3d::build_x3d_scene` accept `&MeshBuffers` directly
//! (unlike the raw-slice encoders) and, like the other lower-level
//! `MeshBuffers`-taking builders in this crate (cf. `stl::mesh_to_stl_ascii`),
//! are now gated too: both call [`ensure_export_allowed`] before building any
//! XML, so a caller can no longer obtain an unsuited mesh's X3D XML in memory
//! without writing a file. `export_x3d` / `export_x3d_scene` call them
//! internally and also gate up front at the file-I/O boundary; the resulting
//! double check is intentionally cheap (a `Vec` length check plus a boolean
//! read) and kept for defense-in-depth / clear error attribution at each
//! layer.
//!
//! # Known gaps (tracked debt — NOT yet on the invariant surface)
//!
//! None currently tracked. The `x3d::build_x3d` / `x3d::build_x3d_scene` gap
//! noted in earlier waves has been closed (see above).

use anyhow::Result;
use oxihuman_mesh::mesh::MeshBuffers;
use oxihuman_mesh::suit::ensure_suit_mesh;

/// The single bodysuit gate: refuse any mesh whose `has_suit` flag is false.
///
/// Returns `Err` with a clear message when the mesh has no suit layer
/// applied; every human-facing exporter entry point calls this before
/// serialising.
#[inline]
pub fn ensure_export_allowed(mesh: &MeshBuffers) -> Result<()> {
    ensure_suit_mesh(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn mesh(has_suit: bool) -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit,
        })
    }

    #[test]
    fn gate_refuses_unsuited_mesh() {
        assert!(ensure_export_allowed(&mesh(false)).is_err());
    }

    #[test]
    fn gate_allows_suited_mesh() {
        assert!(ensure_export_allowed(&mesh(true)).is_ok());
    }
}
