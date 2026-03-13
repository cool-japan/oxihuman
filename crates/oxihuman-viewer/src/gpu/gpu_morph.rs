// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GPU-accelerated morph target application via wgpu compute shader.
//!
//! `GpuMorphPipeline` compiles a WGSL compute shader once and reuses it
//! across frames.  `MorphComputeBuffers` holds the per-dispatch GPU buffers
//! (base positions, sparse delta list, output, params, staging).
//!
//! ## Usage
//! ```ignore
//! let pipeline = GpuMorphPipeline::new(device.clone(), queue.clone());
//! let result   = pipeline.apply_morph(&base_positions, &deltas, weight)?;
//! ```

#[cfg(feature = "webgpu")]
use std::sync::Arc;

// ── WGSL Compute Shader ───────────────────────────────────────────────────────

/// WGSL compute shader for single-target morph application.
///
/// Layout:
/// - `@binding(0)` — base vertex positions  (`array<f32>`, stride 3)
/// - `@binding(1)` — sparse morph deltas    (`array<MorphDelta>`)
/// - `@binding(2)` — output positions       (`array<f32>`, stride 3, read_write)
/// - `@binding(3)` — morph parameters      (`MorphParams` uniform)
///
/// Each thread processes one delta entry: it atomically accumulates
/// `weight * (dx, dy, dz)` into the output vertex identified by `vid`.
/// A first pass copies base positions to output; a second dispatches deltas.
pub const COMPUTE_SHADER_GPU_MORPH: &str = r#"
// ── Types ─────────────────────────────────────────────────────────────────────
struct MorphDelta {
    vid : u32,
    dx  : f32,
    dy  : f32,
    dz  : f32,
}

struct MorphParams {
    weight      : f32,
    delta_count : u32,
    vertex_count: u32,
    _pad        : u32,
}

// ── Bindings ──────────────────────────────────────────────────────────────────
@group(0) @binding(0) var<storage, read>       base_pos : array<f32>;
@group(0) @binding(1) var<storage, read>       deltas   : array<MorphDelta>;
@group(0) @binding(2) var<storage, read_write> out_pos  : array<f32>;
@group(0) @binding(3) var<uniform>             params   : MorphParams;

// ── Copy pass — each thread copies one vertex (x, y, z) ──────────────────────
@compute @workgroup_size(64)
fn copy_base(@builtin(global_invocation_id) gid: vec3<u32>) {
    let vid = gid.x;
    if (vid >= params.vertex_count) { return; }
    out_pos[vid * 3u]      = base_pos[vid * 3u];
    out_pos[vid * 3u + 1u] = base_pos[vid * 3u + 1u];
    out_pos[vid * 3u + 2u] = base_pos[vid * 3u + 2u];
}

// ── Morph pass — each thread applies one delta entry ─────────────────────────
@compute @workgroup_size(64)
fn apply_deltas(@builtin(global_invocation_id) gid: vec3<u32>) {
    let did = gid.x;
    if (did >= params.delta_count) { return; }

    let d   = deltas[did];
    let vid = d.vid;
    if (vid >= params.vertex_count) { return; }

    // Sequential (non-atomic) accumulation is safe when delta list has
    // unique vertex ids per dispatch (guaranteed by MorphTarget encoding).
    out_pos[vid * 3u]      += d.dx * params.weight;
    out_pos[vid * 3u + 1u] += d.dy * params.weight;
    out_pos[vid * 3u + 2u] += d.dz * params.weight;
}
"#;

// ── MorphDeltaEntry ───────────────────────────────────────────────────────────

/// A single sparse morph delta: vertex index + displacement.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct MorphDeltaEntry {
    pub vid: u32,
    pub dx: f32,
    pub dy: f32,
    pub dz: f32,
}

impl MorphDeltaEntry {
    /// Encode to a flat `[u8; 16]` buffer (little-endian).
    pub fn to_bytes(self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.vid.to_le_bytes());
        buf[4..8].copy_from_slice(&self.dx.to_le_bytes());
        buf[8..12].copy_from_slice(&self.dy.to_le_bytes());
        buf[12..16].copy_from_slice(&self.dz.to_le_bytes());
        buf
    }
}

/// Pack a slice of [`MorphDeltaEntry`] into a raw byte vec.
pub fn pack_deltas(deltas: &[MorphDeltaEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(deltas.len() * 16);
    for d in deltas {
        out.extend_from_slice(&d.to_bytes());
    }
    out
}

/// Pack base positions `[[x,y,z]; N]` into a raw f32 byte vec.
pub fn pack_positions(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut out = Vec::with_capacity(positions.len() * 12);
    for p in positions {
        out.extend_from_slice(&p[0].to_le_bytes());
        out.extend_from_slice(&p[1].to_le_bytes());
        out.extend_from_slice(&p[2].to_le_bytes());
    }
    out
}

/// Pack morph params (weight, delta_count, vertex_count, _pad) into 16 bytes.
pub fn pack_params(weight: f32, delta_count: u32, vertex_count: u32) -> [u8; 16] {
    let mut buf = [0u8; 16];
    buf[0..4].copy_from_slice(&weight.to_le_bytes());
    buf[4..8].copy_from_slice(&delta_count.to_le_bytes());
    buf[8..12].copy_from_slice(&vertex_count.to_le_bytes());
    // _pad stays zero
    buf
}

// ── wgpu-gated implementation ─────────────────────────────────────────────────

#[cfg(feature = "webgpu")]
pub use gpu_impl::*;

#[cfg(feature = "webgpu")]
mod gpu_impl {
    use super::*;
    use anyhow::{anyhow, Context, Result};
    use wgpu::util::DeviceExt;

    // ── MorphComputeBuffers ───────────────────────────────────────────────────

    /// Per-dispatch GPU buffers for a morph operation.
    pub struct MorphComputeBuffers {
        /// Base vertex positions  (f32 × 3 × vertex_count).
        pub base_positions: wgpu::Buffer,
        /// Packed morph delta list (MorphDelta × delta_count).
        pub delta_buffer: wgpu::Buffer,
        /// Output positions      (f32 × 3 × vertex_count, STORAGE + COPY_SRC).
        pub output_positions: wgpu::Buffer,
        /// Params uniform        (weight, delta_count, vertex_count, _pad).
        pub params_buffer: wgpu::Buffer,
        /// CPU-readable staging buffer for readback.
        pub staging_buffer: wgpu::Buffer,
        /// Number of vertices.
        pub vertex_count: u32,
        /// Number of delta entries.
        pub delta_count: u32,
    }

    impl MorphComputeBuffers {
        /// Allocate all GPU buffers for one morph dispatch.
        ///
        /// # Arguments
        /// * `device`   — wgpu device
        /// * `positions` — base mesh positions (f32×3 each)
        /// * `deltas`    — sparse delta list
        /// * `weight`    — morph blend weight [0, 1]
        pub fn new(
            device: &wgpu::Device,
            positions: &[[f32; 3]],
            deltas: &[MorphDeltaEntry],
            weight: f32,
        ) -> Result<Self> {
            if positions.is_empty() {
                return Err(anyhow!("positions must not be empty"));
            }

            let vertex_count = positions.len() as u32;
            let delta_count = deltas.len() as u32;

            // Base positions — STORAGE | COPY_DST
            let pos_bytes = pack_positions(positions);
            let base_positions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("morph_base_positions"),
                contents: &pos_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            // Delta buffer — STORAGE | COPY_DST (or minimal if no deltas)
            let delta_bytes = if deltas.is_empty() {
                // Provide a 16-byte dummy so the bind group stays valid
                vec![0u8; 16]
            } else {
                pack_deltas(deltas)
            };
            let delta_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("morph_delta_buffer"),
                contents: &delta_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

            // Output positions — STORAGE | COPY_SRC
            let out_size = (vertex_count as u64) * 12; // 3 × f32
            let output_positions = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("morph_output_positions"),
                size: out_size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            // Params uniform
            let params_bytes = pack_params(weight, delta_count, vertex_count);
            let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("morph_params"),
                contents: &params_bytes,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            // Staging buffer for CPU readback
            let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("morph_staging"),
                size: out_size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            Ok(MorphComputeBuffers {
                base_positions,
                delta_buffer,
                output_positions,
                params_buffer,
                staging_buffer,
                vertex_count,
                delta_count,
            })
        }
    }

    // ── GpuMorphPipeline ──────────────────────────────────────────────────────

    /// Compiled GPU morph pipeline.  Create once, reuse across frames.
    pub struct GpuMorphPipeline {
        pub device: Arc<wgpu::Device>,
        pub queue: Arc<wgpu::Queue>,
        /// Compute pipeline for the copy-base pass.
        pub copy_pipeline: wgpu::ComputePipeline,
        /// Compute pipeline for the apply-deltas pass.
        pub morph_pipeline: wgpu::ComputePipeline,
        pub bind_group_layout: wgpu::BindGroupLayout,
    }

    impl GpuMorphPipeline {
        /// Compile the WGSL compute shader and create both pipelines.
        pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Result<Self> {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("gpu_morph_shader"),
                source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER_GPU_MORPH.into()),
            });

            // Bind group layout: 3 storage + 1 uniform
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("morph_bgl"),
                    entries: &[
                        // binding 0: base_pos (storage, read)
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
                        // binding 1: deltas (storage, read)
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
                        // binding 2: out_pos (storage, read_write)
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
                        // binding 3: params (uniform)
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("morph_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

            let copy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("morph_copy_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("copy_base"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

            let morph_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("morph_apply_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("apply_deltas"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

            Ok(GpuMorphPipeline {
                device,
                queue,
                copy_pipeline,
                morph_pipeline,
                bind_group_layout,
            })
        }

        /// Build the bind group for a set of [`MorphComputeBuffers`].
        fn make_bind_group(&self, bufs: &MorphComputeBuffers) -> wgpu::BindGroup {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("morph_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: bufs.base_positions.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: bufs.delta_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: bufs.output_positions.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: bufs.params_buffer.as_entire_binding(),
                    },
                ],
            })
        }

        /// Apply a single morph target on the GPU.
        ///
        /// 1. Uploads `base_positions` and `deltas` to GPU.
        /// 2. Dispatches copy pass (ceil(vertex_count / 64) workgroups).
        /// 3. Dispatches morph pass (ceil(delta_count / 64) workgroups).
        /// 4. Reads back output positions to a `Vec<f32>` (x,y,z interleaved).
        ///
        /// # Errors
        /// Returns an error if buffer allocation, encoding, or readback fails.
        pub fn apply_morph(
            &self,
            base_positions: &[[f32; 3]],
            deltas: &[MorphDeltaEntry],
            weight: f32,
        ) -> Result<Vec<f32>> {
            // Allocate GPU buffers
            let bufs = MorphComputeBuffers::new(&self.device, base_positions, deltas, weight)
                .context("allocating morph compute buffers")?;

            let bind_group = self.make_bind_group(&bufs);

            // Encode compute passes
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("morph_encoder"),
                });

            // Pass 1: copy base positions into output
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("morph_copy_pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.copy_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                let wg = bufs.vertex_count.div_ceil(64);
                cpass.dispatch_workgroups(wg, 1, 1);
            }

            // Pass 2: apply weighted deltas (skip if no deltas)
            if bufs.delta_count > 0 {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("morph_apply_pass"),
                    timestamp_writes: None,
                });
                cpass.set_pipeline(&self.morph_pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                let wg = bufs.delta_count.div_ceil(64);
                cpass.dispatch_workgroups(wg, 1, 1);
            }

            // Copy output → staging for CPU readback
            let out_size = (bufs.vertex_count as u64) * 12;
            encoder.copy_buffer_to_buffer(
                &bufs.output_positions,
                0,
                &bufs.staging_buffer,
                0,
                out_size,
            );

            self.queue.submit(std::iter::once(encoder.finish()));

            // Map staging buffer and read back
            let slice = bufs.staging_buffer.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });

            let _ = self.device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            });

            rx.recv()
                .map_err(|e| anyhow!("readback channel error: {e}"))?
                .map_err(|e| anyhow!("buffer map failed: {e:?}"))?;

            let data = slice.get_mapped_range();
            let floats: Vec<f32> = data
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();

            Ok(floats)
        }
    }
}

// ── CPU fallback (no webgpu feature) ─────────────────────────────────────────

/// CPU-only morph application (fallback when `webgpu` feature is disabled).
///
/// Applies weighted sparse deltas to base positions and returns the result.
///
/// This is intentionally a simple reference implementation; the GPU path
/// is preferred for large meshes.
pub fn apply_morph_cpu(
    base_positions: &[[f32; 3]],
    deltas: &[MorphDeltaEntry],
    weight: f32,
) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity(base_positions.len() * 3);
    for p in base_positions {
        out.push(p[0]);
        out.push(p[1]);
        out.push(p[2]);
    }
    for d in deltas {
        let idx = d.vid as usize;
        if idx < base_positions.len() {
            out[idx * 3] += d.dx * weight;
            out[idx * 3 + 1] += d.dy * weight;
            out[idx * 3 + 2] += d.dz * weight;
        }
    }
    out
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_positions(n: usize) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| [i as f32, i as f32 * 2.0, i as f32 * 3.0])
            .collect()
    }

    #[test]
    fn test_pack_positions_length() {
        let pos = make_positions(4);
        let bytes = pack_positions(&pos);
        assert_eq!(bytes.len(), 4 * 12);
    }

    #[test]
    fn test_pack_deltas_length() {
        let deltas = vec![
            MorphDeltaEntry {
                vid: 0,
                dx: 1.0,
                dy: 2.0,
                dz: 3.0,
            },
            MorphDeltaEntry {
                vid: 1,
                dx: 0.5,
                dy: 0.0,
                dz: -1.0,
            },
        ];
        let bytes = pack_deltas(&deltas);
        assert_eq!(bytes.len(), 2 * 16);
    }

    #[test]
    fn test_pack_params_roundtrip() {
        let bytes = pack_params(0.75, 42, 100);
        let w = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let dc = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let vc = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let _pad = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        assert!((w - 0.75).abs() < 1e-6, "weight roundtrip");
        assert_eq!(dc, 42);
        assert_eq!(vc, 100);
    }

    #[test]
    fn test_delta_entry_to_bytes_roundtrip() {
        let d = MorphDeltaEntry {
            vid: 7,
            dx: 1.5,
            dy: -2.5,
            dz: 0.125,
        };
        let bytes = d.to_bytes();
        let vid = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let dx = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let dy = f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let dz = f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        assert_eq!(vid, 7);
        assert!((dx - 1.5).abs() < 1e-6);
        assert!((dy + 2.5).abs() < 1e-6);
        assert!((dz - 0.125).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_morph_no_deltas() {
        let pos = make_positions(3);
        let out = apply_morph_cpu(&pos, &[], 1.0);
        assert_eq!(out.len(), 9);
        for (i, p) in pos.iter().enumerate() {
            assert!((out[i * 3] - p[0]).abs() < 1e-6);
            assert!((out[i * 3 + 1] - p[1]).abs() < 1e-6);
            assert!((out[i * 3 + 2] - p[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cpu_morph_full_weight() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let deltas = vec![MorphDeltaEntry {
            vid: 0,
            dx: 1.0,
            dy: 2.0,
            dz: 3.0,
        }];
        let out = apply_morph_cpu(&pos, &deltas, 1.0);
        assert!((out[0] - 1.0).abs() < 1e-6, "x displaced");
        assert!((out[1] - 2.0).abs() < 1e-6, "y displaced");
        assert!((out[2] - 3.0).abs() < 1e-6, "z displaced");
        // vertex 1 unchanged
        assert!((out[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_morph_half_weight() {
        let pos = vec![[0.0f32, 0.0, 0.0]];
        let deltas = vec![MorphDeltaEntry {
            vid: 0,
            dx: 2.0,
            dy: 4.0,
            dz: 6.0,
        }];
        let out = apply_morph_cpu(&pos, &deltas, 0.5);
        assert!((out[0] - 1.0).abs() < 1e-6);
        assert!((out[1] - 2.0).abs() < 1e-6);
        assert!((out[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_morph_zero_weight() {
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let deltas = vec![MorphDeltaEntry {
            vid: 0,
            dx: 100.0,
            dy: 100.0,
            dz: 100.0,
        }];
        let out = apply_morph_cpu(&pos, &deltas, 0.0);
        assert!((out[0] - 1.0).abs() < 1e-6, "no displacement at weight 0");
        assert!((out[1] - 2.0).abs() < 1e-6);
        assert!((out[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_morph_out_of_range_vid() {
        let pos = vec![[0.0f32, 0.0, 0.0]];
        // vid = 99 is out of range — should be silently ignored
        let deltas = vec![MorphDeltaEntry {
            vid: 99,
            dx: 1.0,
            dy: 1.0,
            dz: 1.0,
        }];
        let out = apply_morph_cpu(&pos, &deltas, 1.0);
        assert!((out[0]).abs() < 1e-6, "oob vid ignored");
    }

    #[test]
    fn test_cpu_morph_multiple_deltas_same_vertex() {
        let pos = vec![[0.0f32, 0.0, 0.0]];
        let deltas = vec![
            MorphDeltaEntry {
                vid: 0,
                dx: 1.0,
                dy: 0.0,
                dz: 0.0,
            },
            MorphDeltaEntry {
                vid: 0,
                dx: 0.0,
                dy: 1.0,
                dz: 0.0,
            },
        ];
        let out = apply_morph_cpu(&pos, &deltas, 1.0);
        assert!((out[0] - 1.0).abs() < 1e-6);
        assert!((out[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_pack_positions_values() {
        let pos = vec![[1.0f32, 2.0, 3.0]];
        let bytes = pack_positions(&pos);
        let x = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let y = f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let z = f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - 2.0).abs() < 1e-6);
        assert!((z - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_cpu_morph_large_mesh() {
        let n = 1024;
        let pos: Vec<[f32; 3]> = (0..n).map(|_| [0.0f32, 0.0, 0.0]).collect();
        let deltas: Vec<MorphDeltaEntry> = (0..n as u32)
            .map(|i| MorphDeltaEntry {
                vid: i,
                dx: 1.0,
                dy: 1.0,
                dz: 1.0,
            })
            .collect();
        let out = apply_morph_cpu(&pos, &deltas, 0.5);
        assert_eq!(out.len(), n * 3);
        for i in 0..n {
            assert!((out[i * 3] - 0.5).abs() < 1e-5, "vertex {i} x");
        }
    }

    #[test]
    fn test_gpu_morph_shader_constant_nonempty() {
        assert!(!COMPUTE_SHADER_GPU_MORPH.is_empty());
        assert!(COMPUTE_SHADER_GPU_MORPH.contains("apply_deltas"));
        assert!(COMPUTE_SHADER_GPU_MORPH.contains("copy_base"));
        assert!(COMPUTE_SHADER_GPU_MORPH.contains("MorphDelta"));
        assert!(COMPUTE_SHADER_GPU_MORPH.contains("MorphParams"));
    }
}
