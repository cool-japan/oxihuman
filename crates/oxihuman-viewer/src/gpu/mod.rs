// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GPU-related types: mesh buffers, context, render pipeline, draw utilities,
//! WGSL shaders, pipeline cache, surface configuration, and bind groups.

pub mod context;
pub mod draw;
pub mod gpu_morph;
pub mod mesh_buffers;
pub mod render_pipeline;

// ── Phase 2 wgpu modules ─────────────────────────────────────────────────────

pub mod wgsl_shaders;

#[cfg(feature = "webgpu")]
pub mod bind_groups;
#[cfg(feature = "webgpu")]
pub mod pipeline_cache;
#[cfg(feature = "webgpu")]
pub mod surface_config;

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use gpu_morph::{
    apply_morph_cpu, pack_deltas, pack_params, pack_positions, MorphDeltaEntry,
    COMPUTE_SHADER_GPU_MORPH,
};
pub use mesh_buffers::*;

#[cfg(feature = "webgpu")]
pub use bind_groups::{
    BindGroupLayouts, CameraBindGroupLayout, FullscreenBindGroupLayout,
    FullscreenBindGroupResources, LightsBindGroupLayout, MaterialBindGroupLayout,
    MaterialBindGroupResources, ModelBindGroupLayout, MorphComputeBindGroupLayout,
    MorphComputeResources, CAMERA_UNIFORM_SIZE, LIGHT_ARRAY_UNIFORM_SIZE, MATERIAL_UNIFORM_SIZE,
    MODEL_UNIFORM_SIZE, MORPH_PARAMS_UNIFORM_SIZE, TONEMAP_PARAMS_SIZE,
};

#[cfg(feature = "webgpu")]
pub use pipeline_cache::{mesh_vertex_buffer_layout, PipelineCache, PipelineKey, VERTEX_STRIDE};

#[cfg(feature = "webgpu")]
pub use surface_config::{
    configure_surface, resize_surface, DepthTexture, MsaaTexture, SurfaceConfig, DEPTH_FORMAT,
    MSAA_SAMPLE_COUNT,
};
