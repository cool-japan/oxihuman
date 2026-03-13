// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pipeline caching and creation for the OxiHuman wgpu render pipeline.
//!
//! [`PipelineCache`] lazily creates and caches [`wgpu::RenderPipeline`] objects
//! keyed by [`PipelineKey`].  All pipelines share the same vertex buffer layout
//! (position + normal + UV + tangent, interleaved).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::gpu::wgsl_shaders::{
    FRAGMENT_SHADER_PBR, FRAGMENT_SHADER_SHADOW, FRAGMENT_SHADER_TONEMAP,
    FRAGMENT_SHADER_WIREFRAME, VERTEX_SHADER_FULLSCREEN, VERTEX_SHADER_PBR, VERTEX_SHADER_SHADOW,
    VERTEX_SHADER_WIREFRAME,
};

// ── Vertex stride & attribute offsets ─────────────────────────────────────────

/// Byte size of one interleaved vertex:
/// `position (12) + normal (12) + uv (8) + tangent (16) = 48 bytes`.
pub const VERTEX_STRIDE: u64 = 48;

/// Attribute: position at offset 0 (Float32x3).
const ATTR_POSITION: wgpu::VertexAttribute = wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x3,
    offset: 0,
    shader_location: 0,
};
/// Attribute: normal at offset 12 (Float32x3).
const ATTR_NORMAL: wgpu::VertexAttribute = wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x3,
    offset: 12,
    shader_location: 1,
};
/// Attribute: UV at offset 24 (Float32x2).
const ATTR_UV: wgpu::VertexAttribute = wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 24,
    shader_location: 2,
};
/// Attribute: tangent at offset 32 (Float32x4).
const ATTR_TANGENT: wgpu::VertexAttribute = wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x4,
    offset: 32,
    shader_location: 3,
};

/// Returns the interleaved vertex buffer layout used by all mesh pipelines.
pub fn mesh_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[ATTR_POSITION, ATTR_NORMAL, ATTR_UV, ATTR_TANGENT],
    }
}

// ── PipelineKey ───────────────────────────────────────────────────────────────

/// Discriminant used to look up a cached pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineKey {
    /// Opaque PBR geometry pass.
    Pbr,
    /// Alpha-blended PBR geometry pass.
    PbrAlpha,
    /// Wireframe debug overlay.
    Wireframe,
    /// Shadow map depth pass.
    Shadow,
    /// Fullscreen post-processing (tonemapping).
    Fullscreen,
}

// ── PipelineCache ─────────────────────────────────────────────────────────────

/// Lazily creates and caches [`wgpu::RenderPipeline`]s by [`PipelineKey`].
///
/// All pipelines within a cache share the same:
/// * colour attachment format (`surface_format`)
/// * depth format (`depth_format`)
/// * bind group layouts supplied at construction time
pub struct PipelineCache {
    pipelines: HashMap<PipelineKey, Arc<wgpu::RenderPipeline>>,
    /// Bind group layout for group 0 (camera).
    camera_bgl: Arc<wgpu::BindGroupLayout>,
    /// Bind group layout for group 1 (model transform).
    model_bgl: Arc<wgpu::BindGroupLayout>,
    /// Bind group layout for group 2 (material + textures).
    material_bgl: Arc<wgpu::BindGroupLayout>,
    /// Bind group layout for group 3 (lights).
    lights_bgl: Arc<wgpu::BindGroupLayout>,
}

impl PipelineCache {
    /// Create an empty cache with the provided bind group layouts.
    pub fn new(
        camera_bgl: Arc<wgpu::BindGroupLayout>,
        model_bgl: Arc<wgpu::BindGroupLayout>,
        material_bgl: Arc<wgpu::BindGroupLayout>,
        lights_bgl: Arc<wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            pipelines: HashMap::new(),
            camera_bgl,
            model_bgl,
            material_bgl,
            lights_bgl,
        }
    }

    /// Return a reference to the cached pipeline for `key`, creating it first
    /// if not yet present.
    pub fn get_or_create(
        &mut self,
        key: PipelineKey,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<&wgpu::RenderPipeline> {
        if !self.pipelines.contains_key(&key) {
            let pipeline = self
                .create_pipeline(key, device, surface_format, depth_format)
                .with_context(|| format!("creating pipeline {:?}", key))?;
            self.pipelines.insert(key, Arc::new(pipeline));
        }
        Ok(self.pipelines[&key].as_ref())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn create_pipeline(
        &self,
        key: PipelineKey,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline> {
        match key {
            PipelineKey::Pbr => self.build_pbr(device, surface_format, depth_format, false),
            PipelineKey::PbrAlpha => self.build_pbr(device, surface_format, depth_format, true),
            PipelineKey::Wireframe => self.build_wireframe(device, surface_format, depth_format),
            PipelineKey::Shadow => self.build_shadow(device, depth_format),
            PipelineKey::Fullscreen => self.build_fullscreen(device, surface_format),
        }
    }

    /// Compile a shader module from a WGSL source string.
    fn make_shader(device: &wgpu::Device, label: &str, src: &str) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(src.into()),
        })
    }

    /// PBR pipeline layout shared between opaque and alpha passes.
    fn pbr_pipeline_layout(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pbr_pipeline_layout"),
            bind_group_layouts: &[
                &self.camera_bgl,
                &self.model_bgl,
                &self.material_bgl,
                &self.lights_bgl,
            ],
            immediate_size: 0,
        })
    }

    fn build_pbr(
        &self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        alpha_blend: bool,
    ) -> Result<wgpu::RenderPipeline> {
        let vs = Self::make_shader(device, "pbr_vs", VERTEX_SHADER_PBR);
        let fs = Self::make_shader(device, "pbr_fs", FRAGMENT_SHADER_PBR);
        let layout = self.pbr_pipeline_layout(device);

        let blend = if alpha_blend {
            Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
            })
        } else {
            Some(wgpu::BlendState::REPLACE)
        };

        let depth_write = !alpha_blend;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(if alpha_blend {
                "pbr_alpha_pipeline"
            } else {
                "pbr_pipeline"
            }),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("vs_main"),
                buffers: &[mesh_vertex_buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: depth_write,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4, // 4x MSAA
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }

    fn build_wireframe(
        &self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline> {
        let vs = Self::make_shader(device, "wireframe_vs", VERTEX_SHADER_WIREFRAME);
        let fs = Self::make_shader(device, "wireframe_fs", FRAGMENT_SHADER_WIREFRAME);

        // Wireframe uses only camera + model + a small color uniform (group 2).
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wireframe_pipeline_layout"),
            bind_group_layouts: &[
                &self.camera_bgl,
                &self.model_bgl,
                &self.material_bgl, // reused for wireframe color param
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("wireframe_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("vs_wireframe"),
                buffers: &[mesh_vertex_buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("fs_wireframe"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }

    fn build_shadow(
        &self,
        device: &wgpu::Device,
        depth_format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline> {
        let vs = Self::make_shader(device, "shadow_vs", VERTEX_SHADER_SHADOW);
        let fs = Self::make_shader(device, "shadow_fs", FRAGMENT_SHADER_SHADOW);

        // Shadow pass: shadow uniform (group 0) + model (group 1).
        let shadow_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_pipeline_layout"),
            bind_group_layouts: &[&shadow_bgl, &self.model_bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("vs_shadow"),
                buffers: &[mesh_vertex_buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("fs_shadow"),
                targets: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Front), // front-face culling reduces shadow acne
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }

    fn build_fullscreen(
        &self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Result<wgpu::RenderPipeline> {
        let vs = Self::make_shader(device, "fullscreen_vs", VERTEX_SHADER_FULLSCREEN);
        let fs = Self::make_shader(device, "tonemap_fs", FRAGMENT_SHADER_TONEMAP);

        // Fullscreen pass: HDR texture + sampler + tonemap params (group 0).
        let fs_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fullscreen_bgl"),
            entries: &[
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fullscreen_pipeline_layout"),
            bind_group_layouts: &[&fs_bgl],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fullscreen_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("fs_tonemap"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(pipeline)
    }
}
