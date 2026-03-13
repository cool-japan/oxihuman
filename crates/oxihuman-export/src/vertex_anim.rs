// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Vertex animation export: sequence of mesh frames as GLTF morph target animation.

use anyhow::Result;
use bytemuck::cast_slice;
use oxihuman_mesh::MeshBuffers;
use serde_json::json;
use std::io::Write;
use std::path::Path;

// GLB constants
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

// ── AnimFrame ─────────────────────────────────────────────────────────────────

/// A single frame in a vertex animation.
#[allow(dead_code)]
pub struct AnimFrame {
    pub time: f32,
    pub positions: Vec<[f32; 3]>,
}

impl AnimFrame {
    #[allow(dead_code)]
    pub fn new(time: f32, positions: Vec<[f32; 3]>) -> Self {
        Self { time, positions }
    }

    /// Compute per-vertex delta from a base mesh.
    #[allow(dead_code)]
    pub fn deltas_from_base(&self, base: &[[f32; 3]]) -> Vec<[f32; 3]> {
        self.positions
            .iter()
            .zip(base.iter())
            .map(|(p, b)| [p[0] - b[0], p[1] - b[1], p[2] - b[2]])
            .collect()
    }
}

// ── VertexAnimation ───────────────────────────────────────────────────────────

/// A vertex animation sequence.
#[allow(dead_code)]
pub struct VertexAnimation {
    pub name: String,
    pub frames: Vec<AnimFrame>,
    pub fps: f32,
}

impl VertexAnimation {
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, fps: f32) -> Self {
        Self {
            name: name.into(),
            frames: Vec::new(),
            fps,
        }
    }

    #[allow(dead_code)]
    pub fn add_frame(&mut self, frame: AnimFrame) {
        self.frames.push(frame);
    }

    #[allow(dead_code)]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Duration: time of last frame minus time of first frame.
    #[allow(dead_code)]
    pub fn duration(&self) -> f32 {
        if self.frames.len() < 2 {
            return 0.0;
        }
        self.frames.last().map_or(0.0, |f| f.time) - self.frames.first().map_or(0.0, |f| f.time)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Compute all frame deltas relative to the first frame as the base.
    #[allow(dead_code)]
    pub fn frame_deltas(&self) -> Vec<Vec<[f32; 3]>> {
        if self.frames.is_empty() {
            return Vec::new();
        }
        let base = &self.frames[0].positions;
        self.frames
            .iter()
            .map(|f| f.deltas_from_base(base))
            .collect()
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Append bytes, then pad to 4-byte alignment. Returns (offset, original_len).
fn append_aligned(buf: &mut Vec<u8>, data: &[u8]) -> (usize, usize) {
    let offset = buf.len();
    let length = data.len();
    buf.extend_from_slice(data);
    while !buf.len().is_multiple_of(4) {
        buf.push(0x00);
    }
    (offset, length)
}

/// Flatten `[[f32;3]]` into bytes.
fn positions_to_bytes(positions: &[[f32; 3]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(positions.len() * 12);
    for p in positions {
        for &c in p {
            buf.extend_from_slice(&c.to_le_bytes());
        }
    }
    buf
}

/// Write GLB binary to file from JSON bytes and BIN bytes.
fn write_glb(path: &Path, json_bytes: &[u8], bin_data: &[u8]) -> Result<()> {
    let mut json_padded = json_bytes.to_vec();
    while !json_padded.len().is_multiple_of(4) {
        json_padded.push(b' ');
    }

    let mut bin_padded = bin_data.to_vec();
    while !bin_padded.len().is_multiple_of(4) {
        bin_padded.push(0x00);
    }

    let json_chunk_len = json_padded.len() as u32;
    let bin_chunk_len = bin_padded.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;

    // GLB header (12 bytes)
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;

    // JSON chunk
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_padded)?;

    // BIN chunk
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_padded)?;

    Ok(())
}

// ── export_vertex_anim_glb ────────────────────────────────────────────────────

/// Export a mesh with a vertex animation as a GLB file using GLTF morph targets.
///
/// Each animation frame becomes a morph target with a weight keyframe in the
/// animation. The base mesh uses `base_mesh.positions` (frame 0 of the sequence
/// provides the reference when deltas are computed via `VertexAnimation::frame_deltas`).
/// The animation drives the morph target weights over time (one active at a time).
///
/// BIN layout:
///   `[base POSITION][base NORMAL][base UV][base INDEX]`
///   `[delta POSITION frame 0][delta POSITION frame 1]`...
///   `[timestamps][weights]`
#[allow(dead_code)]
pub fn export_vertex_anim_glb(
    base_mesh: &MeshBuffers,
    anim: &VertexAnimation,
    path: &Path,
) -> Result<()> {
    let n_verts = base_mesh.positions.len();
    let n_idx = base_mesh.indices.len();
    let n_frames = anim.frame_count();

    // ── Build BIN ─────────────────────────────────────────────────────────────
    let pos_bytes: &[u8] = cast_slice(&base_mesh.positions);
    let norm_bytes: &[u8] = cast_slice(&base_mesh.normals);
    let uv_bytes: &[u8] = cast_slice(&base_mesh.uvs);
    let idx_bytes: &[u8] = cast_slice(&base_mesh.indices);

    let mut bin: Vec<u8> = Vec::new();

    let (pos_offset, pos_len) = append_aligned(&mut bin, pos_bytes);
    let (norm_offset, norm_len) = append_aligned(&mut bin, norm_bytes);
    let (uv_offset, uv_len) = append_aligned(&mut bin, uv_bytes);
    let (idx_offset, idx_len) = append_aligned(&mut bin, idx_bytes);

    // Frame deltas
    let all_deltas = anim.frame_deltas();
    let mut delta_sections: Vec<(usize, usize)> = Vec::with_capacity(n_frames);
    for deltas in &all_deltas {
        let bytes = positions_to_bytes(deltas);
        let sec = append_aligned(&mut bin, &bytes);
        delta_sections.push(sec);
    }

    // Timestamps (one per frame)
    let mut times_bytes: Vec<u8> = Vec::with_capacity(n_frames * 4);
    for frame in &anim.frames {
        times_bytes.extend_from_slice(&frame.time.to_le_bytes());
    }
    let (times_offset, times_len) = append_aligned(&mut bin, &times_bytes);

    // Weights: n_frames rows × n_frames columns (one-hot: active target per keyframe)
    // At keyframe k, weight[k] = 1.0 and all others = 0.0
    let n_weights = n_frames * n_frames;
    let mut weights_bytes: Vec<u8> = Vec::with_capacity(n_weights * 4);
    for k in 0..n_frames {
        for t in 0..n_frames {
            let w: f32 = if t == k { 1.0 } else { 0.0 };
            weights_bytes.extend_from_slice(&w.to_le_bytes());
        }
    }
    let (weights_offset, weights_len) = append_aligned(&mut bin, &weights_bytes);

    // ── Build GLTF JSON ───────────────────────────────────────────────────────
    // Accessor/bufferView layout:
    //   0: base POSITION (VEC3)
    //   1: base NORMAL   (VEC3)
    //   2: base UV       (VEC2)
    //   3: INDEX         (SCALAR u32)
    //   4..4+n_frames-1: delta POSITION per frame (VEC3)
    //   4+n_frames:      timestamps (SCALAR)
    //   4+n_frames+1:    weights    (SCALAR)

    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();

    let push_bv_acc = |buffer_views: &mut Vec<serde_json::Value>,
                       accessors: &mut Vec<serde_json::Value>,
                       offset: usize,
                       byte_len: usize,
                       component_type: u32,
                       count: usize,
                       type_str: &str| {
        let bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": byte_len
        }));
        accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": component_type,
            "count": count,
            "type": type_str
        }));
    };

    // 0: base POSITION
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        pos_offset,
        pos_len,
        5126,
        n_verts,
        "VEC3",
    );
    // 1: base NORMAL
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        norm_offset,
        norm_len,
        5126,
        n_verts,
        "VEC3",
    );
    // 2: base UV
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        uv_offset,
        uv_len,
        5126,
        n_verts,
        "VEC2",
    );
    // 3: INDEX
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        idx_offset,
        idx_len,
        5125,
        n_idx,
        "SCALAR",
    );

    // 4..4+n_frames: delta POSITION per frame
    let mut morph_targets: Vec<serde_json::Value> = Vec::with_capacity(n_frames);
    for &(d_offset, d_len) in &delta_sections {
        let acc_idx = accessors.len();
        push_bv_acc(
            &mut buffer_views,
            &mut accessors,
            d_offset,
            d_len,
            5126,
            n_verts,
            "VEC3",
        );
        morph_targets.push(json!({ "POSITION": acc_idx }));
    }

    // timestamps accessor
    let times_acc_idx = accessors.len();
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        times_offset,
        times_len,
        5126,
        n_frames,
        "SCALAR",
    );

    // weights accessor: count = n_frames * n_frames
    let weights_acc_idx = accessors.len();
    push_bv_acc(
        &mut buffer_views,
        &mut accessors,
        weights_offset,
        weights_len,
        5126,
        n_frames * n_frames,
        "SCALAR",
    );

    let initial_weights: Vec<f32> = vec![0.0_f32; n_frames];

    let animations = if n_frames > 0 {
        json!([{
            "name": anim.name,
            "samplers": [{
                "input": times_acc_idx,
                "output": weights_acc_idx,
                "interpolation": "LINEAR"
            }],
            "channels": [{
                "sampler": 0,
                "target": { "node": 0, "path": "weights" }
            }]
        }])
    } else {
        json!([])
    };

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "oxihuman-export" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "name": anim.name,
            "weights": initial_weights,
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1,
                    "TEXCOORD_0": 2
                },
                "indices": 3,
                "targets": morph_targets
            }]
        }],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": bin.len() }],
        "animations": animations
    });

    let json_bytes = serde_json::to_vec(&gltf)?;
    write_glb(path, &json_bytes, &bin)
}

// ── export_morph_pair_glb ─────────────────────────────────────────────────────

/// Export just 2 frames as a "morph from A to B" GLB.
///
/// `mesh_a` is the base mesh. `mesh_b` positions define the morph target.
/// At t=0, weight=0 (mesh_a). At t=`duration`, weight=1 (mesh_b).
#[allow(dead_code)]
pub fn export_morph_pair_glb(
    mesh_a: &MeshBuffers,
    mesh_b: &MeshBuffers,
    duration: f32,
    path: &Path,
) -> Result<()> {
    let n_verts = mesh_a.positions.len();
    let n_idx = mesh_a.indices.len();

    // Compute deltas: mesh_b.positions - mesh_a.positions
    let deltas: Vec<[f32; 3]> = mesh_a
        .positions
        .iter()
        .zip(mesh_b.positions.iter())
        .map(|(a, b)| [b[0] - a[0], b[1] - a[1], b[2] - a[2]])
        .collect();

    // ── Build BIN ─────────────────────────────────────────────────────────────
    let pos_bytes: &[u8] = cast_slice(&mesh_a.positions);
    let norm_bytes: &[u8] = cast_slice(&mesh_a.normals);
    let uv_bytes: &[u8] = cast_slice(&mesh_a.uvs);
    let idx_bytes: &[u8] = cast_slice(&mesh_a.indices);
    let delta_bytes = positions_to_bytes(&deltas);

    let mut bin: Vec<u8> = Vec::new();
    let (pos_offset, pos_len) = append_aligned(&mut bin, pos_bytes);
    let (norm_offset, norm_len) = append_aligned(&mut bin, norm_bytes);
    let (uv_offset, uv_len) = append_aligned(&mut bin, uv_bytes);
    let (idx_offset, idx_len) = append_aligned(&mut bin, idx_bytes);
    let (delta_offset, delta_len) = append_aligned(&mut bin, &delta_bytes);

    // Timestamps: t=0 and t=duration
    let mut times_bytes: Vec<u8> = Vec::with_capacity(8);
    times_bytes.extend_from_slice(&0.0_f32.to_le_bytes());
    times_bytes.extend_from_slice(&duration.to_le_bytes());
    let (times_offset, times_len) = append_aligned(&mut bin, &times_bytes);

    // Weights: [0.0] at t=0, [1.0] at t=duration  (1 morph target)
    let mut weights_bytes: Vec<u8> = Vec::with_capacity(8);
    weights_bytes.extend_from_slice(&0.0_f32.to_le_bytes());
    weights_bytes.extend_from_slice(&1.0_f32.to_le_bytes());
    let (weights_offset, weights_len) = append_aligned(&mut bin, &weights_bytes);

    // ── Build GLTF JSON ───────────────────────────────────────────────────────
    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();

    let push_bv_acc_mp = |buffer_views: &mut Vec<serde_json::Value>,
                          accessors: &mut Vec<serde_json::Value>,
                          offset: usize,
                          byte_len: usize,
                          component_type: u32,
                          count: usize,
                          type_str: &str| {
        let bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": byte_len
        }));
        accessors.push(json!({
            "bufferView": bv_idx,
            "componentType": component_type,
            "count": count,
            "type": type_str
        }));
    };

    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        pos_offset,
        pos_len,
        5126,
        n_verts,
        "VEC3",
    ); // 0: POSITION
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        norm_offset,
        norm_len,
        5126,
        n_verts,
        "VEC3",
    ); // 1: NORMAL
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        uv_offset,
        uv_len,
        5126,
        n_verts,
        "VEC2",
    ); // 2: TEXCOORD_0
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        idx_offset,
        idx_len,
        5125,
        n_idx,
        "SCALAR",
    ); // 3: INDEX
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        delta_offset,
        delta_len,
        5126,
        n_verts,
        "VEC3",
    ); // 4: delta POSITION
    let times_acc_idx = accessors.len();
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        times_offset,
        times_len,
        5126,
        2,
        "SCALAR",
    ); // 5: timestamps
    let weights_acc_idx = accessors.len();
    push_bv_acc_mp(
        &mut buffer_views,
        &mut accessors,
        weights_offset,
        weights_len,
        5126,
        2,
        "SCALAR",
    ); // 6: weights

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "oxihuman-export" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{
            "name": "morph_pair",
            "weights": [0.0_f32],
            "primitives": [{
                "attributes": {
                    "POSITION": 0,
                    "NORMAL": 1,
                    "TEXCOORD_0": 2
                },
                "indices": 3,
                "targets": [{ "POSITION": 4 }]
            }]
        }],
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": bin.len() }],
        "animations": [{
            "name": "morph",
            "samplers": [{
                "input": times_acc_idx,
                "output": weights_acc_idx,
                "interpolation": "LINEAR"
            }],
            "channels": [{
                "sampler": 0,
                "target": { "node": 0, "path": "weights" }
            }]
        }]
    });

    let json_bytes = serde_json::to_vec(&gltf)?;
    write_glb(path, &json_bytes, &bin)
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    fn triangle_a() -> MeshBuffers {
        make_mesh(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }

    fn triangle_b() -> MeshBuffers {
        make_mesh(vec![[0.1, 0.0, 0.0], [1.1, 0.0, 0.0], [0.1, 1.0, 0.0]])
    }

    // ── Test 1 ────────────────────────────────────────────────────────────────
    #[test]
    fn anim_frame_new_fields() {
        let positions = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let frame = AnimFrame::new(0.5, positions.clone());
        assert_eq!(frame.time, 0.5);
        assert_eq!(frame.positions, positions);
    }

    // ── Test 2 ────────────────────────────────────────────────────────────────
    #[test]
    fn anim_frame_deltas_from_base_zero_when_same() {
        let positions = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let frame = AnimFrame::new(0.0, positions.clone());
        let deltas = frame.deltas_from_base(&positions);
        for d in &deltas {
            assert_eq!(d, &[0.0_f32, 0.0, 0.0]);
        }
    }

    // ── Test 3 ────────────────────────────────────────────────────────────────
    #[test]
    fn anim_frame_deltas_correct_offset() {
        let base = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let positions = vec![[0.5, 0.1, 0.2], [1.5, 0.1, 0.2]];
        let frame = AnimFrame::new(1.0, positions);
        let deltas = frame.deltas_from_base(&base);
        assert!((deltas[0][0] - 0.5).abs() < 1e-6);
        assert!((deltas[0][1] - 0.1).abs() < 1e-6);
        assert!((deltas[0][2] - 0.2).abs() < 1e-6);
        assert!((deltas[1][0] - 0.5).abs() < 1e-6);
    }

    // ── Test 4 ────────────────────────────────────────────────────────────────
    #[test]
    fn vertex_animation_frame_count() {
        let mut anim = VertexAnimation::new("walk", 24.0);
        assert_eq!(anim.frame_count(), 0);
        anim.add_frame(AnimFrame::new(0.0, vec![[0.0, 0.0, 0.0]]));
        anim.add_frame(AnimFrame::new(1.0, vec![[1.0, 0.0, 0.0]]));
        assert_eq!(anim.frame_count(), 2);
    }

    // ── Test 5 ────────────────────────────────────────────────────────────────
    #[test]
    fn vertex_animation_duration() {
        let mut anim = VertexAnimation::new("run", 30.0);
        // empty → 0
        assert_eq!(anim.duration(), 0.0);
        anim.add_frame(AnimFrame::new(0.5, vec![[0.0, 0.0, 0.0]]));
        // single frame → 0
        assert_eq!(anim.duration(), 0.0);
        anim.add_frame(AnimFrame::new(2.5, vec![[1.0, 0.0, 0.0]]));
        assert!((anim.duration() - 2.0).abs() < 1e-6);
    }

    // ── Test 6 ────────────────────────────────────────────────────────────────
    #[test]
    fn vertex_animation_frame_deltas_length() {
        let mut anim = VertexAnimation::new("test", 24.0);
        anim.add_frame(AnimFrame::new(0.0, vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]));
        anim.add_frame(AnimFrame::new(1.0, vec![[0.1, 0.0, 0.0], [1.1, 0.0, 0.0]]));
        anim.add_frame(AnimFrame::new(2.0, vec![[0.2, 0.0, 0.0], [1.2, 0.0, 0.0]]));
        let deltas = anim.frame_deltas();
        assert_eq!(deltas.len(), 3);
        // Each delta slice has 2 verts
        for d in &deltas {
            assert_eq!(d.len(), 2);
        }
        // First frame delta should be all zeros (relative to itself)
        for v in &deltas[0] {
            assert_eq!(v, &[0.0_f32, 0.0, 0.0]);
        }
    }

    // ── Test 7 ────────────────────────────────────────────────────────────────
    #[test]
    fn export_morph_pair_creates_file() {
        let mesh_a = triangle_a();
        let mesh_b = triangle_b();
        let path = Path::new("/tmp/test_vertex_anim_pair.glb");
        export_morph_pair_glb(&mesh_a, &mesh_b, 1.0, path).expect("export failed");
        assert!(path.exists(), "GLB file was not created");
        let meta = std::fs::metadata(path).expect("should succeed");
        assert!(meta.len() > 0, "GLB file is empty");
        std::fs::remove_file(path).ok();
    }

    // ── Test 8 ────────────────────────────────────────────────────────────────
    #[test]
    fn export_morph_pair_valid_glb_header() {
        let mesh_a = triangle_a();
        let mesh_b = triangle_b();
        let path = Path::new("/tmp/test_vertex_anim_pair_header.glb");
        export_morph_pair_glb(&mesh_a, &mesh_b, 2.0, path).expect("export failed");
        let bytes = std::fs::read(path).expect("should succeed");
        assert!(bytes.len() >= 12, "file too short");
        // GLB magic: "glTF" = 0x46546C67 LE
        assert_eq!(&bytes[0..4], &[0x67, 0x6C, 0x54, 0x46], "wrong GLB magic");
        // Version = 2
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("should succeed"));
        assert_eq!(version, 2);
        std::fs::remove_file(Path::new("/tmp/test_vertex_anim_pair_header.glb")).ok();
    }

    // ── Test 9 ────────────────────────────────────────────────────────────────
    #[test]
    fn export_vertex_anim_glb_creates_file() {
        let base_mesh = triangle_a();
        let mut anim = VertexAnimation::new("test_anim", 24.0);
        anim.add_frame(AnimFrame::new(
            0.0,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ));
        anim.add_frame(AnimFrame::new(
            1.0,
            vec![[0.1, 0.0, 0.0], [1.1, 0.0, 0.0], [0.1, 1.0, 0.0]],
        ));
        let path = Path::new("/tmp/test_vertex_anim_seq.glb");
        export_vertex_anim_glb(&base_mesh, &anim, path).expect("export failed");
        assert!(path.exists(), "GLB file was not created");
        let meta = std::fs::metadata(path).expect("should succeed");
        assert!(meta.len() > 0, "GLB file is empty");
        std::fs::remove_file(path).ok();
    }

    // ── Test 10 ───────────────────────────────────────────────────────────────
    #[test]
    fn export_vertex_anim_glb_valid_header() {
        let base_mesh = triangle_a();
        let mut anim = VertexAnimation::new("hdr_test", 24.0);
        anim.add_frame(AnimFrame::new(
            0.0,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        ));
        anim.add_frame(AnimFrame::new(
            0.5,
            vec![[0.2, 0.0, 0.0], [1.2, 0.0, 0.0], [0.2, 1.0, 0.0]],
        ));
        let path = Path::new("/tmp/test_vertex_anim_seq_header.glb");
        export_vertex_anim_glb(&base_mesh, &anim, path).expect("export failed");
        let bytes = std::fs::read(path).expect("should succeed");
        assert!(bytes.len() >= 12);
        assert_eq!(&bytes[0..4], &[0x67, 0x6C, 0x54, 0x46], "wrong GLB magic");
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("should succeed"));
        assert_eq!(version, 2);
        std::fs::remove_file(path).ok();
    }

    // ── Test 11 ───────────────────────────────────────────────────────────────
    #[test]
    fn export_vertex_anim_larger_than_base_glb() {
        // A vertex anim GLB should be larger than a base-only GLB because it
        // contains morph target delta data + animation data.
        let base_mesh = triangle_a();

        // Export base-only morph pair (2 keyframes)
        let mesh_b = triangle_b();
        let pair_path = Path::new("/tmp/test_vertex_anim_larger_pair.glb");
        export_morph_pair_glb(&base_mesh, &mesh_b, 1.0, pair_path).expect("pair export failed");
        let pair_size = std::fs::metadata(pair_path).expect("should succeed").len();

        // Export multi-frame vertex anim (3 frames = more data)
        let mut anim = VertexAnimation::new("multi", 24.0);
        for i in 0..3 {
            anim.add_frame(AnimFrame::new(
                i as f32 * 0.5,
                vec![
                    [i as f32 * 0.1, 0.0, 0.0],
                    [1.0 + i as f32 * 0.1, 0.0, 0.0],
                    [0.0, 1.0 + i as f32 * 0.1, 0.0],
                ],
            ));
        }
        let seq_path = Path::new("/tmp/test_vertex_anim_larger_seq.glb");
        export_vertex_anim_glb(&base_mesh, &anim, seq_path).expect("seq export failed");
        let seq_size = std::fs::metadata(seq_path).expect("should succeed").len();

        // The 3-frame animation has 3 morph targets + 3x3 weight matrix,
        // whereas the pair has 1 morph target + 2 weights; seq must be larger.
        assert!(
            seq_size > pair_size,
            "3-frame seq ({seq_size}) should be larger than morph pair ({pair_size})"
        );

        std::fs::remove_file(pair_path).ok();
        std::fs::remove_file(seq_path).ok();
    }
}
