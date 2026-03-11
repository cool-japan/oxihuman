// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bind group layouts and bind group creation for the OxiHuman wgpu pipeline.
//!
//! This module provides strongly-typed layout objects for every bind group
//! used by the render pipeline, along with helper structs to build the
//! corresponding [`wgpu::BindGroup`]s from concrete GPU resources.
//!
//! # Layout overview
//!
//! | Group | Name                 | Bindings                                              |
//! |-------|----------------------|-------------------------------------------------------|
//! | 0     | Camera               | 0: CameraUniform (uniform)                            |
//! | 1     | Model                | 0: ModelUniform (uniform)                             |
//! | 2     | Material + Textures  | 0: MaterialUniform, 1-6: textures + samplers          |
//! | 3     | Lights               | 0: LightArray (uniform)                               |
//! | –     | MorphCompute         | 0-3: storage buffers + params uniform (compute only)  |
//! | –     | Fullscreen / Tonemap | 0-2: HDR texture + sampler + TonemapParams            |

use std::sync::Arc;

use anyhow::Result;

// ── GPU-side uniform structs (bytemuck-compatible sizes) ─────────────────────
//
// These are documentation references only; the actual byte layout is defined
// in the WGSL shaders.  The sizes here are used for `min_binding_size` checks.

/// Size of the camera uniform block in bytes.
/// ```text
/// view(64) + proj(64) + view_proj(64) + eye_pos(16) + near_far(8) + _pad(8) = 224
/// ```
pub const CAMERA_UNIFORM_SIZE: u64 = 224;

/// Size of the model uniform block in bytes.
/// ```text
/// model(64) + model_inv_t(64) = 128
/// ```
pub const MODEL_UNIFORM_SIZE: u64 = 128;

/// Size of the material uniform block in bytes.
/// ```text
/// albedo(16) + metallic(4) + roughness(4) + _pad(8) + emissive(16) = 48
/// ```
pub const MATERIAL_UNIFORM_SIZE: u64 = 48;

/// Size of the light array uniform block in bytes.
/// ```text
/// point_lights: 8 × (16+16) = 256
/// dir_light:    16+16 = 32
/// num_points:   4
/// _pad:         12
/// total:        304
/// ```
pub const LIGHT_ARRAY_UNIFORM_SIZE: u64 = 304;

/// Size of the morph params uniform block in bytes.
/// ```text
/// num_vertices(4) + num_targets(4) + _pad(8) + weights(64 × 4) = 272
/// ```
pub const MORPH_PARAMS_UNIFORM_SIZE: u64 = 272;

/// Size of the tonemap params uniform block in bytes.
/// ```text
/// exposure(4) + gamma(4) + _pad(8) = 16
/// ```
pub const TONEMAP_PARAMS_SIZE: u64 = 16;

// ── CameraBindGroupLayout ─────────────────────────────────────────────────────

/// Bind group layout for group 0: camera uniform buffer.
pub struct CameraBindGroupLayout(pub wgpu::BindGroupLayout);

impl CameraBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(CAMERA_UNIFORM_SIZE),
                },
                count: None,
            }],
        });
        Self(layout)
    }

    /// Create a [`wgpu::BindGroup`] from a camera uniform buffer.
    pub fn bind(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &self.0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(bg)
    }
}

// ── ModelBindGroupLayout ──────────────────────────────────────────────────────

/// Bind group layout for group 1: per-object model transform.
pub struct ModelBindGroupLayout(pub wgpu::BindGroupLayout);

impl ModelBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("model_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(MODEL_UNIFORM_SIZE),
                },
                count: None,
            }],
        });
        Self(layout)
    }

    pub fn bind(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("model_bg"),
            layout: &self.0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(bg)
    }
}

// ── MaterialBindGroupLayout ───────────────────────────────────────────────────

/// Resources required to construct a material bind group.
pub struct MaterialBindGroupResources<'a> {
    pub material_buffer: &'a wgpu::Buffer,
    pub albedo_texture: &'a wgpu::TextureView,
    pub albedo_sampler: &'a wgpu::Sampler,
    pub normal_texture: &'a wgpu::TextureView,
    pub normal_sampler: &'a wgpu::Sampler,
    pub metallic_roughness_tex: &'a wgpu::TextureView,
    pub metallic_roughness_smp: &'a wgpu::Sampler,
}

/// Bind group layout for group 2:
/// `MaterialUniform + albedo tex/smp + normal tex/smp + metallic_roughness tex/smp`.
pub struct MaterialBindGroupLayout(pub wgpu::BindGroupLayout);

impl MaterialBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let entries = &[
            // 0: MaterialUniform
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(MATERIAL_UNIFORM_SIZE),
                },
                count: None,
            },
            // 1: albedo texture
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // 2: albedo sampler
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // 3: normal texture
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // 4: normal sampler
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // 5: metallic_roughness texture
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // 6: metallic_roughness sampler
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material_bgl"),
            entries,
        });
        Self(layout)
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        res: &MaterialBindGroupResources<'_>,
    ) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("material_bg"),
            layout: &self.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: res.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(res.albedo_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(res.albedo_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(res.normal_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(res.normal_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(res.metallic_roughness_tex),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(res.metallic_roughness_smp),
                },
            ],
        });
        Ok(bg)
    }
}

// ── LightsBindGroupLayout ─────────────────────────────────────────────────────

/// Bind group layout for group 3: light array uniform.
pub struct LightsBindGroupLayout(pub wgpu::BindGroupLayout);

impl LightsBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lights_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(LIGHT_ARRAY_UNIFORM_SIZE),
                },
                count: None,
            }],
        });
        Self(layout)
    }

    pub fn bind(&self, device: &wgpu::Device, buffer: &wgpu::Buffer) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights_bg"),
            layout: &self.0,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        Ok(bg)
    }
}

// ── MorphComputeBindGroupLayout ───────────────────────────────────────────────

/// Resources required to construct a morph compute bind group.
pub struct MorphComputeResources<'a> {
    /// Input position buffer (storage, read-only).
    pub in_positions: &'a wgpu::Buffer,
    /// Morph delta buffer (storage, read-only).
    pub morph_deltas: &'a wgpu::Buffer,
    /// Output position buffer (storage, read-write).
    pub out_positions: &'a wgpu::Buffer,
    /// Morph params uniform buffer.
    pub params: &'a wgpu::Buffer,
}

/// Bind group layout for the GPU morph compute pass.
pub struct MorphComputeBindGroupLayout(pub wgpu::BindGroupLayout);

impl MorphComputeBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let entries = &[
            // 0: in_positions (read-only storage)
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // 1: morph_deltas (read-only storage)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // 2: out_positions (read-write storage)
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // 3: morph params (uniform)
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(MORPH_PARAMS_UNIFORM_SIZE),
                },
                count: None,
            },
        ];

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("morph_compute_bgl"),
            entries,
        });
        Self(layout)
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        res: &MorphComputeResources<'_>,
    ) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("morph_compute_bg"),
            layout: &self.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: res.in_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: res.morph_deltas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: res.out_positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: res.params.as_entire_binding(),
                },
            ],
        });
        Ok(bg)
    }
}

// ── FullscreenBindGroupLayout (tonemap pass) ──────────────────────────────────

/// Resources required to construct a fullscreen / tonemap bind group.
pub struct FullscreenBindGroupResources<'a> {
    pub hdr_texture: &'a wgpu::TextureView,
    pub hdr_sampler: &'a wgpu::Sampler,
    pub tonemap_params: &'a wgpu::Buffer,
}

/// Bind group layout for the fullscreen tonemapping pass (group 0).
pub struct FullscreenBindGroupLayout(pub wgpu::BindGroupLayout);

impl FullscreenBindGroupLayout {
    pub fn create(device: &wgpu::Device) -> Self {
        let entries = &[
            // 0: HDR texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // 1: HDR sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // 2: TonemapParams uniform
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(TONEMAP_PARAMS_SIZE),
                },
                count: None,
            },
        ];

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fullscreen_bgl"),
            entries,
        });
        Self(layout)
    }

    pub fn bind(
        &self,
        device: &wgpu::Device,
        res: &FullscreenBindGroupResources<'_>,
    ) -> Result<wgpu::BindGroup> {
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fullscreen_bg"),
            layout: &self.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(res.hdr_texture),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(res.hdr_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: res.tonemap_params.as_entire_binding(),
                },
            ],
        });
        Ok(bg)
    }
}

// ── BindGroupLayouts — collected set ─────────────────────────────────────────

/// All bind group layouts collected in one place for easy cloning into
/// [`crate::gpu::PipelineCache`].
pub struct BindGroupLayouts {
    pub camera: Arc<wgpu::BindGroupLayout>,
    pub model: Arc<wgpu::BindGroupLayout>,
    pub material: Arc<wgpu::BindGroupLayout>,
    pub lights: Arc<wgpu::BindGroupLayout>,
    pub morph_compute: Arc<wgpu::BindGroupLayout>,
    pub fullscreen: Arc<wgpu::BindGroupLayout>,
}

impl BindGroupLayouts {
    /// Create all bind group layouts from a device.
    pub fn create(device: &wgpu::Device) -> Self {
        Self {
            camera: Arc::new(CameraBindGroupLayout::create(device).0),
            model: Arc::new(ModelBindGroupLayout::create(device).0),
            material: Arc::new(MaterialBindGroupLayout::create(device).0),
            lights: Arc::new(LightsBindGroupLayout::create(device).0),
            morph_compute: Arc::new(MorphComputeBindGroupLayout::create(device).0),
            fullscreen: Arc::new(FullscreenBindGroupLayout::create(device).0),
        }
    }
}
