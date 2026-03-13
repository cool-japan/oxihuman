// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export pipeline for OxiHuman — 50+ geometry, animation, and texture formats.
//!
//! This crate translates [`oxihuman_mesh::MeshBuffers`] into a wide array of
//! output formats. The primary entry point for most users is
//! [`export_auto`], which infers the format from the file extension. For batch
//! pipelines use [`batch_export`] or the async-friendly [`ExportJobQueue`].
//!
//! # Supported format families
//!
//! | Family | Key functions |
//! |---|---|
//! | glTF/GLB | [`export_glb`], [`export_gltf_sep`], [`export_glb_blend_shapes`] |
//! | OBJ / MTL | [`export_obj`], [`export_obj_mtl`] |
//! | COLLADA | [`export_collada`], [`export_collada_scene`] |
//! | STL | [`export_stl_binary`], [`export_stl_ascii`] |
//! | USD / USDZ | [`export_usda`], [`package_usdz`] |
//! | Alembic | [`AlembicWriter`] (Ogawa-compatible binary writer) |
//! | Point cache | [`export_pc2`], [`export_mdd`], [`export_point_cache`] |
//! | VRM | [`build_vrm_extensions_json`] |
//! | 3MF | [`export_3mf`] |
//! | Streaming | [`stream_mesh_positions`] |
//!
//! # Quick start
//!
//! ```rust,no_run
//! use oxihuman_export::export_auto;
//! use oxihuman_mesh::MeshBuffers;
//! use std::path::Path;
//!
//! fn export_human(mesh: &MeshBuffers) -> anyhow::Result<()> {
//!     export_auto(mesh, Path::new("/tmp/human.glb"))
//! }
//! ```

// Part 1: Core 3D formats — glTF/GLB, OBJ, COLLADA, STL, USD, Alembic,
//         point-cache, streaming, texture/material helpers, misc geometry I/O.
include!("_export_part1.rs");

// Part 2: Rigging, deformation & material/texture-map exports
//         (blend_mask → geo_modifier, ~88 modules).
include!("_export_part2.rs");

// Part 3: More deformation, collision, curve, animation exports
//         (geo_modifier → svg_path, ~88 modules).
include!("_export_part3.rs");

// Part 4: Web/streaming/shader/serialisation formats
//         (svg_path → avro/parquet/arrow/wav, ~88 modules).
include!("_export_part4.rs");

// Part 5: Protocol & audio exports — MIDI, OSC, DMX, ROS, geo-data formats,
//         image formats, rendering-related exports (~88 modules).
include!("_export_part5.rs");

// Part 6: Body, biometric, skin, ML-model & biomechanics exports
//         (galvanic → opensim_ik, ~87 modules).
include!("_export_part6.rs");
