// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh processing, topology, and geometry algorithms for OxiHuman.
//!
//! This crate sits between the morph engine and the export pipeline. It takes
//! raw [`oxihuman_morph::engine::MeshBuffers`] output and enriches it with
//! recomputed normals, tangents, optional vertex colors, UV atlasing, LOD
//! decimation, Catmull-Clark subdivision, cloth/skeleton skinning, geodesic
//! distances, and a suite of topology repair routines.
//!
//! # Key types
//!
//! - [`MeshBuffers`] — the canonical mesh representation used by exporters.
//! - [`Skeleton`] / [`Joint`] — rig hierarchy for linear blend skinning.
//! - [`SkinWeights`] — per-vertex bone weights.
//! - [`BodyMeasurements`] — computed anthropometric measurements from a mesh.
//!
//! # Example: build and repair a mesh
//!
//! ```rust
//! use oxihuman_mesh::mesh::MeshBuffers;
//! use oxihuman_mesh::repair::repair_mesh;
//! use oxihuman_morph::engine::MeshBuffers as MorphBuffers;
//!
//! let morph_out = MorphBuffers {
//!     positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
//!     normals:   vec![[0.0, 0.0, 1.0]; 3],
//!     uvs:       vec![[0.0, 0.0]; 3],
//!     indices:   vec![0, 1, 2],
//!     has_suit:  false,
//! };
//! let mut mesh = MeshBuffers::from_morph(morph_out);
//! let report = repair_mesh(&mut mesh);
//! assert!(report.degenerate_faces_removed == 0 || report.duplicate_faces_removed == 0);
//! ```

// ---------------------------------------------------------------------------
// Color utilities (set_uniform_color + tests)
// ---------------------------------------------------------------------------
pub mod color_utils;
pub use color_utils::set_uniform_color;

// ---------------------------------------------------------------------------
// Module batch A — core mesh types, normals, repair, connectivity, geodesic
// (lines 38-134 and 191-500 of the original lib.rs)
// ---------------------------------------------------------------------------
include!("mods_a.rs");

// ---------------------------------------------------------------------------
// Module batch B — sampling, curvature, AO, BVH, marching-cubes, remesh …
// (lines 501-1000 of the original lib.rs)
// ---------------------------------------------------------------------------
include!("mods_b.rs");

// ---------------------------------------------------------------------------
// Module batch C — mesh_lscm, mesh_tet, mesh_pca, mesh_progressive, …
// (lines 1001-1600 of the original lib.rs)
// ---------------------------------------------------------------------------
include!("mods_c.rs");

// ---------------------------------------------------------------------------
// Module batch D — mesh_vertex_*, mesh_uv_*, cloth, hair, modifiers …
// (lines 1601-2500 of the original lib.rs)
// ---------------------------------------------------------------------------
include!("mods_d.rs");

// ---------------------------------------------------------------------------
// Module batch E — dissolve, flatten, constraints, tools, query modules …
// (lines 2501-3323 of the original lib.rs)
// ---------------------------------------------------------------------------
include!("mods_e.rs");
