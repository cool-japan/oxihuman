// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Real-time viewer for OxiHuman meshes.
//!
//! This crate provides the data structures and (when the `webgpu` feature is
//! enabled) the wgpu render loop for previewing morphed human meshes in a
//! window or headless surface. The public API is intentionally thin: most
//! downstream consumers will use the [`Viewer`] facade and the
//! [`ViewerConfig`] configuration struct.
//!
//! # Architecture
//!
//! - [`Scene`] / [`SceneNode`] — a lightweight scene graph with transforms and lights.
//! - [`RenderPipelineDescriptor`] — describes vertex layout, blend state, depth/stencil, and cull mode.
//! - [`CameraState`] — orbit camera with pan/zoom.
//! - [`MeshUploadBuffer`] — staging buffer for GPU mesh data.
//! - [`Viewer`] — top-level render loop (stub; wgpu integration in Phase 2).
//!
//! # Feature flags
//!
//! | Flag | Effect |
//! |---|---|
//! | `webgpu` (default off) | Enables full wgpu rendering via `wgpu` crate |

pub mod material;
pub mod pipeline;
pub mod scene;

pub use material::{
    color_to_hex, lerp_material, material_to_gltf_json, Color4, MaterialLibrary, PbrMaterial,
};
pub use pipeline::{
    default_mesh_pipeline, pipeline_summary, standard_vertex_layout, transparent_pipeline,
    validate_pipeline, vertex_format_size, wireframe_pipeline, BlendFactor, BlendState,
    CompareFunction, CullMode, DepthStencilState, IndexFormat, PrimitiveTopology,
    RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat,
};
pub use scene::{
    default_scene, mat4_identity, mat4_multiply, Light, LightKind, NodeContent, Scene, SceneNode,
    Transform,
};

pub mod camera;
pub mod gpu;
pub mod lighting_presets;
pub mod render_loop;
pub mod scene_state;
pub mod screenshot;

pub use camera::CameraState;
pub use gpu::MeshUploadBuffer;
pub use lighting_presets::LightingPreset;
pub use render_loop::Viewer;
pub use scene_state::{ViewerConfig, ViewerStats};
pub use screenshot::{ImageBuffer, ScreenshotCapture};

// ── Phase 2 modules: event loop, morph updater, LOD v2, render stats v3 ──────

pub mod event_loop;
pub mod lod_manager_v2;
pub mod morph_updater;
pub mod render_stats_v3;

pub use event_loop::{
    headless_window_state, tick_headless, FrameTiming, InputState, OrbitCameraController,
    WindowState,
};
pub use lod_manager_v2::{
    build_lod_chain, default_lod_configs, DrawParams, LodConfig, LodLevelV2, LodManagerV2, LodMesh,
    LodTransition, Mesh,
};
pub use morph_updater::{zero_positions, MorphSlider, MorphTargetDeltas, MorphUpdater};
pub use render_stats_v3::{FrameTimer, RenderStatsSnapshot, RenderStatsV3};

include!("_mods_part1.rs");
include!("_mods_part2.rs");
include!("_mods_part3.rs");
include!("_mods_part4.rs");
