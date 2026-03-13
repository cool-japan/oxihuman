// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Core Alembic / Ogawa data types, encoding helpers, validation, and
//! group-builder functions.
//!
//! This module does **not** perform any file I/O; see [`super::alembic_ogawa_io`]
//! for the [`AlembicWriter`] high-level API and disk-write helpers.

use anyhow::{bail, ensure, Result};

// ── Ogawa container primitives ──────────────────────────────────────────────

/// Magic bytes that identify an Ogawa (Alembic) file.
pub(super) const OGAWA_MAGIC: [u8; 8] = [0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];

/// A child entry inside an [`OgawaGroup`].
#[derive(Debug, Clone)]
pub(super) enum OgawaChild {
    /// Reference to another group by index.
    Group(usize),
    /// Inline raw data blob.
    Data(Vec<u8>),
}

/// A group node in the Ogawa container tree.
#[derive(Debug, Clone)]
pub(super) struct OgawaGroup {
    pub(super) children: Vec<OgawaChild>,
}

impl OgawaGroup {
    pub(super) fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub(super) fn add_data(&mut self, data: Vec<u8>) {
        self.children.push(OgawaChild::Data(data));
    }

    pub(super) fn add_group(&mut self, idx: usize) {
        self.children.push(OgawaChild::Group(idx));
    }
}

// ── Public data types ───────────────────────────────────────────────────────

/// Top-level writer that accumulates Alembic objects and serialises them
/// to the Ogawa binary format.
///
/// See [`super::alembic_ogawa_io`] for the full implementation.
pub struct AlembicWriter {
    pub(super) time_sampling: Option<TimeSampling>,
    pub(super) objects: Vec<AbcObject>,
}

/// Time sampling configuration.
#[derive(Debug, Clone)]
pub(super) struct TimeSampling {
    pub(super) start: f64,
    pub(super) dt: f64,
    pub(super) num_samples: usize,
}

/// An object in the Alembic hierarchy.
#[derive(Debug, Clone)]
pub struct AbcObject {
    /// Object name (path component).
    pub name: String,
    /// The schema / kind of this object.
    pub kind: AbcObjectKind,
    /// Child objects.
    pub children: Vec<AbcObject>,
}

/// Supported Alembic object schemas.
#[derive(Debug, Clone)]
pub enum AbcObjectKind {
    /// Transform node.
    Xform(AbcXform),
    /// Polygon mesh.
    PolyMesh(AbcPolyMesh),
    /// Subdivision surface.
    SubD(AbcSubD),
    /// Camera.
    Camera(AbcCamera),
}

/// Transform data (optionally animated).
#[derive(Debug, Clone)]
pub struct AbcXform {
    /// 4x4 column-major matrix (identity = rest pose).
    pub matrix: [f64; 16],
    /// Animated matrices: `(time, matrix)` pairs.
    pub animated_matrices: Vec<(f64, [f64; 16])>,
}

/// Polygon mesh data (optionally animated positions).
#[derive(Debug, Clone)]
pub struct AbcPolyMesh {
    /// Vertex positions.
    pub positions: Vec<[f64; 3]>,
    /// Per-face vertex count.
    pub face_counts: Vec<i32>,
    /// Flattened face-vertex indices.
    pub face_indices: Vec<i32>,
    /// Optional per-vertex normals.
    pub normals: Option<Vec<[f64; 3]>>,
    /// Optional per-vertex UV coordinates.
    pub uvs: Option<Vec<[f64; 2]>>,
    /// Animated position samples: `(time, positions)`.
    pub animated_positions: Vec<(f64, Vec<[f64; 3]>)>,
}

/// Subdivision surface data.
#[derive(Debug, Clone)]
pub struct AbcSubD {
    /// Vertex positions.
    pub positions: Vec<[f64; 3]>,
    /// Per-face vertex count.
    pub face_counts: Vec<i32>,
    /// Flattened face-vertex indices.
    pub face_indices: Vec<i32>,
    /// Crease edge indices (pairs).
    pub crease_indices: Vec<i32>,
    /// Crease lengths.
    pub crease_lengths: Vec<i32>,
    /// Crease sharpness values.
    pub crease_sharpnesses: Vec<f64>,
}

/// Camera parameters.
#[derive(Debug, Clone)]
pub struct AbcCamera {
    /// Focal length in mm.
    pub focal_length: f64,
    /// Near clipping plane distance.
    pub near_clip: f64,
    /// Far clipping plane distance.
    pub far_clip: f64,
    /// Horizontal film aperture in cm.
    pub horizontal_aperture: f64,
    /// Vertical film aperture in cm.
    pub vertical_aperture: f64,
}

// ── Alembic schema string constants ─────────────────────────────────────────

const SCHEMA_XFORM: &str = "AbcGeom_Xform_v3";
const SCHEMA_POLYMESH: &str = "AbcGeom_PolyMesh_v1";
const SCHEMA_SUBD: &str = "AbcGeom_SubD_v1";
const SCHEMA_CAMERA: &str = "AbcGeom_Camera_v1";

pub(super) const ABC_CORE_VERSION: u32 = 1;

// ── Encoding helpers ────────────────────────────────────────────────────────

pub(super) fn encode_u64(val: u64) -> [u8; 8] {
    val.to_le_bytes()
}

pub(super) fn encode_i64(val: i64) -> [u8; 8] {
    val.to_le_bytes()
}

pub(super) fn encode_u32(val: u32) -> [u8; 4] {
    val.to_le_bytes()
}

pub(super) fn encode_i32(val: i32) -> [u8; 4] {
    val.to_le_bytes()
}

pub(super) fn encode_f64(val: f64) -> [u8; 8] {
    val.to_le_bytes()
}

pub(super) fn encode_f64x3_slice(vals: &[[f64; 3]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 24);
    for v in vals {
        buf.extend_from_slice(&encode_f64(v[0]));
        buf.extend_from_slice(&encode_f64(v[1]));
        buf.extend_from_slice(&encode_f64(v[2]));
    }
    buf
}

pub(super) fn encode_f64x2_slice(vals: &[[f64; 2]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 16);
    for v in vals {
        buf.extend_from_slice(&encode_f64(v[0]));
        buf.extend_from_slice(&encode_f64(v[1]));
    }
    buf
}

pub(super) fn encode_f64_slice(vals: &[f64]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 8);
    for &v in vals {
        buf.extend_from_slice(&encode_f64(v));
    }
    buf
}

pub(super) fn encode_i32_slice(vals: &[i32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 4);
    for &v in vals {
        buf.extend_from_slice(&encode_i32(v));
    }
    buf
}

pub(super) fn encode_string(s: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(s.len() + 1);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    buf
}

pub(super) fn encode_matrix(m: &[f64; 16]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);
    for &v in m {
        buf.extend_from_slice(&encode_f64(v));
    }
    buf
}

// ── Internal build context ──────────────────────────────────────────────────

pub(super) struct WriteContext {
    pub(super) groups: Vec<OgawaGroup>,
}

impl WriteContext {
    pub(super) fn alloc_group(&mut self, g: OgawaGroup) -> usize {
        let idx = self.groups.len();
        self.groups.push(g);
        idx
    }
}

// ── Validation ──────────────────────────────────────────────────────────────

pub(super) fn validate_object(obj: &AbcObject) -> Result<()> {
    ensure!(!obj.name.is_empty(), "object name must not be empty");

    match &obj.kind {
        AbcObjectKind::PolyMesh(mesh) => {
            let expected_idx_count: i64 = mesh.face_counts.iter().map(|&c| c as i64).sum();
            ensure!(
                expected_idx_count as usize == mesh.face_indices.len(),
                "PolyMesh '{}': face_counts sum ({}) != face_indices length ({})",
                obj.name,
                expected_idx_count,
                mesh.face_indices.len()
            );
            let n_verts = mesh.positions.len();
            for (si, (_, anim_pos)) in mesh.animated_positions.iter().enumerate() {
                ensure!(
                    anim_pos.len() == n_verts,
                    "PolyMesh '{}': animated sample {} has {} positions, expected {}",
                    obj.name,
                    si,
                    anim_pos.len(),
                    n_verts
                );
            }
            if let Some(ref normals) = mesh.normals {
                ensure!(
                    normals.len() == n_verts,
                    "PolyMesh '{}': normals count ({}) != positions count ({})",
                    obj.name,
                    normals.len(),
                    n_verts
                );
            }
            if let Some(ref uvs) = mesh.uvs {
                ensure!(
                    uvs.len() == n_verts,
                    "PolyMesh '{}': UVs count ({}) != positions count ({})",
                    obj.name,
                    uvs.len(),
                    n_verts
                );
            }
        }
        AbcObjectKind::SubD(subd) => {
            let expected_idx_count: i64 = subd.face_counts.iter().map(|&c| c as i64).sum();
            ensure!(
                expected_idx_count as usize == subd.face_indices.len(),
                "SubD '{}': face_counts sum ({}) != face_indices length ({})",
                obj.name,
                expected_idx_count,
                subd.face_indices.len()
            );
        }
        AbcObjectKind::Camera(cam) => {
            ensure!(
                cam.focal_length > 0.0,
                "Camera '{}': focal_length must be positive",
                obj.name
            );
            ensure!(
                cam.near_clip > 0.0 && cam.far_clip > cam.near_clip,
                "Camera '{}': invalid clip planes (near={}, far={})",
                obj.name,
                cam.near_clip,
                cam.far_clip
            );
        }
        AbcObjectKind::Xform(_) => {}
    }

    for child in &obj.children {
        validate_object(child)?;
    }

    Ok(())
}

// ── Build Ogawa groups from Alembic objects ─────────────────────────────────

pub(super) fn build_archive_metadata(ctx: &mut WriteContext) -> Result<usize> {
    let mut group = OgawaGroup::new();
    group.add_data(encode_string("oxihuman-export"));
    group.add_data(encode_string("Alembic 1.8 (Ogawa)"));
    group.add_data(encode_string("2026-03-11"));
    Ok(ctx.alloc_group(group))
}

pub(super) fn build_time_sampling(
    ctx: &mut WriteContext,
    ts: &Option<TimeSampling>,
) -> Result<usize> {
    let mut group = OgawaGroup::new();

    match ts {
        Some(ts) => {
            group.add_data(encode_u32(2).to_vec());

            let mut default_ts = OgawaGroup::new();
            default_ts.add_data(encode_f64(0.0).to_vec());
            default_ts.add_data(encode_u32(1).to_vec());
            let default_idx = ctx.alloc_group(default_ts);
            group.add_group(default_idx);

            let mut user_ts = OgawaGroup::new();
            user_ts.add_data(encode_f64(ts.start).to_vec());
            user_ts.add_data(encode_f64(ts.dt).to_vec());
            user_ts.add_data(encode_u32(ts.num_samples as u32).to_vec());
            let user_idx = ctx.alloc_group(user_ts);
            group.add_group(user_idx);
        }
        None => {
            group.add_data(encode_u32(1).to_vec());
            let mut default_ts = OgawaGroup::new();
            default_ts.add_data(encode_f64(0.0).to_vec());
            default_ts.add_data(encode_u32(1).to_vec());
            let default_idx = ctx.alloc_group(default_ts);
            group.add_group(default_idx);
        }
    }

    Ok(ctx.alloc_group(group))
}

pub(super) fn build_object_group(
    ctx: &mut WriteContext,
    obj: &AbcObject,
    ts: &Option<TimeSampling>,
) -> Result<usize> {
    let mut group = OgawaGroup::new();

    group.add_data(encode_string(&obj.name));

    let schema_str = match &obj.kind {
        AbcObjectKind::Xform(_) => SCHEMA_XFORM,
        AbcObjectKind::PolyMesh(_) => SCHEMA_POLYMESH,
        AbcObjectKind::SubD(_) => SCHEMA_SUBD,
        AbcObjectKind::Camera(_) => SCHEMA_CAMERA,
    };
    group.add_data(encode_string(schema_str));

    let props_idx = build_properties_group(ctx, &obj.kind, ts)?;
    group.add_group(props_idx);

    for child in &obj.children {
        let child_idx = build_object_group(ctx, child, ts)?;
        group.add_group(child_idx);
    }

    Ok(ctx.alloc_group(group))
}

fn build_properties_group(
    ctx: &mut WriteContext,
    kind: &AbcObjectKind,
    ts: &Option<TimeSampling>,
) -> Result<usize> {
    match kind {
        AbcObjectKind::Xform(xform) => build_xform_properties(ctx, xform, ts),
        AbcObjectKind::PolyMesh(mesh) => build_polymesh_properties(ctx, mesh, ts),
        AbcObjectKind::SubD(subd) => build_subd_properties(ctx, subd),
        AbcObjectKind::Camera(cam) => build_camera_properties(ctx, cam),
    }
}

fn build_xform_properties(
    ctx: &mut WriteContext,
    xform: &AbcXform,
    ts: &Option<TimeSampling>,
) -> Result<usize> {
    let mut group = OgawaGroup::new();

    group.add_data(encode_string(".xform"));

    let ts_idx: u32 = if !xform.animated_matrices.is_empty() && ts.is_some() {
        1
    } else {
        0
    };
    group.add_data(encode_u32(ts_idx).to_vec());

    group.add_data(encode_matrix(&xform.matrix));

    if !xform.animated_matrices.is_empty() {
        let mut anim_group = OgawaGroup::new();
        anim_group.add_data(encode_u32(xform.animated_matrices.len() as u32).to_vec());
        for (time, mat) in &xform.animated_matrices {
            let mut sample_group = OgawaGroup::new();
            sample_group.add_data(encode_f64(*time).to_vec());
            sample_group.add_data(encode_matrix(mat));
            let si = ctx.alloc_group(sample_group);
            anim_group.add_group(si);
        }
        let ai = ctx.alloc_group(anim_group);
        group.add_group(ai);
    }

    Ok(ctx.alloc_group(group))
}

fn build_polymesh_properties(
    ctx: &mut WriteContext,
    mesh: &AbcPolyMesh,
    ts: &Option<TimeSampling>,
) -> Result<usize> {
    let mut group = OgawaGroup::new();

    group.add_data(encode_string(".geom"));

    let ts_idx: u32 = if !mesh.animated_positions.is_empty() && ts.is_some() {
        1
    } else {
        0
    };
    group.add_data(encode_u32(ts_idx).to_vec());

    // Positions (P)
    let pos_data = encode_f64x3_slice(&mesh.positions);
    let mut pos_group = OgawaGroup::new();
    pos_group.add_data(encode_string("P"));
    pos_group.add_data(encode_u32(mesh.positions.len() as u32).to_vec());
    pos_group.add_data(pos_data);
    let pos_idx = ctx.alloc_group(pos_group);
    group.add_group(pos_idx);

    // Face counts (.faceCounts)
    let fc_data = encode_i32_slice(&mesh.face_counts);
    let mut fc_group = OgawaGroup::new();
    fc_group.add_data(encode_string(".faceCounts"));
    fc_group.add_data(encode_u32(mesh.face_counts.len() as u32).to_vec());
    fc_group.add_data(fc_data);
    let fc_idx = ctx.alloc_group(fc_group);
    group.add_group(fc_idx);

    // Face indices (.faceIndices)
    let fi_data = encode_i32_slice(&mesh.face_indices);
    let mut fi_group = OgawaGroup::new();
    fi_group.add_data(encode_string(".faceIndices"));
    fi_group.add_data(encode_u32(mesh.face_indices.len() as u32).to_vec());
    fi_group.add_data(fi_data);
    let fi_idx = ctx.alloc_group(fi_group);
    group.add_group(fi_idx);

    // Normals (N)
    if let Some(ref normals) = mesh.normals {
        let n_data = encode_f64x3_slice(normals);
        let mut n_group = OgawaGroup::new();
        n_group.add_data(encode_string("N"));
        n_group.add_data(encode_u32(normals.len() as u32).to_vec());
        n_group.add_data(n_data);
        let ni = ctx.alloc_group(n_group);
        group.add_group(ni);
    }

    // UVs
    if let Some(ref uvs) = mesh.uvs {
        let uv_data = encode_f64x2_slice(uvs);
        let mut uv_group = OgawaGroup::new();
        uv_group.add_data(encode_string("uv"));
        uv_group.add_data(encode_u32(uvs.len() as u32).to_vec());
        uv_group.add_data(uv_data);
        let uvi = ctx.alloc_group(uv_group);
        group.add_group(uvi);
    }

    // Animated position samples
    if !mesh.animated_positions.is_empty() {
        let mut anim_group = OgawaGroup::new();
        anim_group.add_data(encode_u32(mesh.animated_positions.len() as u32).to_vec());
        for (time, positions) in &mesh.animated_positions {
            let mut sample_group = OgawaGroup::new();
            sample_group.add_data(encode_f64(*time).to_vec());
            sample_group.add_data(encode_u32(positions.len() as u32).to_vec());
            sample_group.add_data(encode_f64x3_slice(positions));
            let si = ctx.alloc_group(sample_group);
            anim_group.add_group(si);
        }
        let ai = ctx.alloc_group(anim_group);
        group.add_group(ai);
    }

    Ok(ctx.alloc_group(group))
}

fn build_subd_properties(ctx: &mut WriteContext, subd: &AbcSubD) -> Result<usize> {
    let mut group = OgawaGroup::new();

    group.add_data(encode_string(".geom"));
    group.add_data(encode_u32(0).to_vec());

    // Positions (P)
    let pos_data = encode_f64x3_slice(&subd.positions);
    let mut pos_group = OgawaGroup::new();
    pos_group.add_data(encode_string("P"));
    pos_group.add_data(encode_u32(subd.positions.len() as u32).to_vec());
    pos_group.add_data(pos_data);
    let pos_idx = ctx.alloc_group(pos_group);
    group.add_group(pos_idx);

    // Face counts
    let fc_data = encode_i32_slice(&subd.face_counts);
    let mut fc_group = OgawaGroup::new();
    fc_group.add_data(encode_string(".faceCounts"));
    fc_group.add_data(encode_u32(subd.face_counts.len() as u32).to_vec());
    fc_group.add_data(fc_data);
    let fc_idx = ctx.alloc_group(fc_group);
    group.add_group(fc_idx);

    // Face indices
    let fi_data = encode_i32_slice(&subd.face_indices);
    let mut fi_group = OgawaGroup::new();
    fi_group.add_data(encode_string(".faceIndices"));
    fi_group.add_data(encode_u32(subd.face_indices.len() as u32).to_vec());
    fi_group.add_data(fi_data);
    let fi_idx = ctx.alloc_group(fi_group);
    group.add_group(fi_idx);

    // Crease indices
    if !subd.crease_indices.is_empty() {
        let ci_data = encode_i32_slice(&subd.crease_indices);
        let mut ci_group = OgawaGroup::new();
        ci_group.add_data(encode_string(".creaseIndices"));
        ci_group.add_data(encode_u32(subd.crease_indices.len() as u32).to_vec());
        ci_group.add_data(ci_data);
        let ci_idx = ctx.alloc_group(ci_group);
        group.add_group(ci_idx);
    }

    // Crease lengths
    if !subd.crease_lengths.is_empty() {
        let cl_data = encode_i32_slice(&subd.crease_lengths);
        let mut cl_group = OgawaGroup::new();
        cl_group.add_data(encode_string(".creaseLengths"));
        cl_group.add_data(encode_u32(subd.crease_lengths.len() as u32).to_vec());
        cl_group.add_data(cl_data);
        let cl_idx = ctx.alloc_group(cl_group);
        group.add_group(cl_idx);
    }

    // Crease sharpnesses
    if !subd.crease_sharpnesses.is_empty() {
        let cs_data = encode_f64_slice(&subd.crease_sharpnesses);
        let mut cs_group = OgawaGroup::new();
        cs_group.add_data(encode_string(".creaseSharpnesses"));
        cs_group.add_data(encode_u32(subd.crease_sharpnesses.len() as u32).to_vec());
        cs_group.add_data(cs_data);
        let cs_idx = ctx.alloc_group(cs_group);
        group.add_group(cs_idx);
    }

    Ok(ctx.alloc_group(group))
}

fn build_camera_properties(ctx: &mut WriteContext, cam: &AbcCamera) -> Result<usize> {
    let mut group = OgawaGroup::new();

    group.add_data(encode_string(".camera"));
    group.add_data(encode_u32(0).to_vec());

    let cam_data = [
        cam.focal_length,
        cam.horizontal_aperture,
        cam.vertical_aperture,
        cam.near_clip,
        cam.far_clip,
    ];
    let mut props_group = OgawaGroup::new();
    props_group.add_data(encode_string(".coreProperties"));
    props_group.add_data(encode_u32(cam_data.len() as u32).to_vec());
    props_group.add_data(encode_f64_slice(&cam_data));
    let pi = ctx.alloc_group(props_group);
    group.add_group(pi);

    Ok(ctx.alloc_group(group))
}

// ── Ogawa serialization ─────────────────────────────────────────────────────

/// Serialize the group tree into a valid Ogawa binary stream.
pub(super) fn serialize_ogawa(groups: &[OgawaGroup], root_idx: usize) -> Result<Vec<u8>> {
    ensure!(
        root_idx < groups.len(),
        "root group index {} out of range ({})",
        root_idx,
        groups.len()
    );

    let mut buf: Vec<u8> = Vec::with_capacity(4096);

    // 1. Magic
    buf.extend_from_slice(&OGAWA_MAGIC);

    // 2. Root group offset placeholder (will be patched)
    let root_offset_pos = buf.len();
    buf.extend_from_slice(&[0u8; 8]);

    // Post-order traversal so child offsets are known before parent
    let order = topological_order(groups, root_idx)?;
    let mut group_offsets: Vec<Option<u64>> = vec![None; groups.len()];

    for &gi in &order {
        let g = &groups[gi];
        let child_count = g.children.len() as u64;

        // Serialize Data children inline, record offsets for Group children
        let mut child_offsets: Vec<u64> = Vec::with_capacity(g.children.len());

        for child in &g.children {
            match child {
                OgawaChild::Data(data) => {
                    let data_offset = buf.len() as u64;
                    let neg_len = -(data.len() as i64);
                    buf.extend_from_slice(&encode_i64(neg_len));
                    buf.extend_from_slice(data);
                    child_offsets.push(data_offset);
                }
                OgawaChild::Group(idx) => {
                    let offset = group_offsets[*idx].ok_or_else(|| {
                        anyhow::anyhow!(
                            "internal error: group {} not yet serialized when referenced by group {}",
                            idx,
                            gi
                        )
                    })?;
                    child_offsets.push(offset);
                }
            }
        }

        // Write the group header
        let group_offset = buf.len() as u64;
        group_offsets[gi] = Some(group_offset);

        buf.extend_from_slice(&encode_u64(child_count));
        for &co in &child_offsets {
            buf.extend_from_slice(&encode_u64(co));
        }
    }

    // Patch root group offset
    let root_off = group_offsets[root_idx]
        .ok_or_else(|| anyhow::anyhow!("internal error: root group was not serialized"))?;
    let root_off_bytes = encode_u64(root_off);
    buf[root_offset_pos..root_offset_pos + 8].copy_from_slice(&root_off_bytes);

    Ok(buf)
}

/// Compute a post-order traversal of the group DAG.
pub(super) fn topological_order(groups: &[OgawaGroup], root: usize) -> Result<Vec<usize>> {
    let mut visited = vec![false; groups.len()];
    let mut on_stack = vec![false; groups.len()];
    let mut order = Vec::with_capacity(groups.len());

    fn dfs(
        groups: &[OgawaGroup],
        idx: usize,
        visited: &mut [bool],
        on_stack: &mut [bool],
        order: &mut Vec<usize>,
    ) -> Result<()> {
        if visited[idx] {
            return Ok(());
        }
        if on_stack[idx] {
            bail!("cycle detected in Ogawa group graph at index {}", idx);
        }
        on_stack[idx] = true;

        for child in &groups[idx].children {
            if let OgawaChild::Group(child_idx) = child {
                if *child_idx >= groups.len() {
                    bail!(
                        "group {} references out-of-range child group {}",
                        idx,
                        child_idx
                    );
                }
                dfs(groups, *child_idx, visited, on_stack, order)?;
            }
        }

        on_stack[idx] = false;
        visited[idx] = true;
        order.push(idx);
        Ok(())
    }

    dfs(groups, root, &mut visited, &mut on_stack, &mut order)?;

    Ok(order)
}

// ── Convenience constructors ────────────────────────────────────────────────

/// Create an identity 4x4 matrix (column-major).
pub fn identity_matrix() -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Create a translation matrix (column-major).
pub fn translation_matrix(tx: f64, ty: f64, tz: f64) -> [f64; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, tx, ty, tz, 1.0,
    ]
}

/// Create a uniform scale matrix (column-major).
pub fn scale_matrix(s: f64) -> [f64; 16] {
    [
        s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Create a simple unit cube `AbcPolyMesh` (8 vertices, 6 quad faces).
pub fn unit_cube_polymesh() -> AbcPolyMesh {
    AbcPolyMesh {
        positions: vec![
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ],
        face_counts: vec![4, 4, 4, 4, 4, 4],
        face_indices: vec![
            0, 1, 2, 3, 4, 7, 6, 5, 0, 3, 7, 4, 1, 5, 6, 2, 3, 2, 6, 7, 0, 4, 5, 1,
        ],
        normals: None,
        uvs: None,
        animated_positions: Vec::new(),
    }
}

/// Validate the Ogawa magic bytes in a buffer.
pub fn validate_ogawa_magic(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == OGAWA_MAGIC
}

/// Read the root group offset from an Ogawa buffer (bytes 8..16).
pub fn read_root_offset(data: &[u8]) -> Option<u64> {
    if data.len() < 16 {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&data[8..16]);
    Some(u64::from_le_bytes(bytes))
}

/// Read a group header at the given offset.
///
/// Returns `(child_count, child_offsets)` or `None` if malformed.
pub fn read_group_at(data: &[u8], offset: u64) -> Option<(u64, Vec<u64>)> {
    let off = offset as usize;
    if off + 8 > data.len() {
        return None;
    }

    let mut count_bytes = [0u8; 8];
    count_bytes.copy_from_slice(&data[off..off + 8]);
    let count_raw = i64::from_le_bytes(count_bytes);

    if count_raw < 0 {
        return None;
    }

    let count = count_raw as u64;
    let offsets_start = off + 8;
    let offsets_end = offsets_start + (count as usize) * 8;
    if offsets_end > data.len() {
        return None;
    }

    let mut child_offsets = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let base = offsets_start + i * 8;
        let mut ob = [0u8; 8];
        ob.copy_from_slice(&data[base..base + 8]);
        child_offsets.push(u64::from_le_bytes(ob));
    }

    Some((count, child_offsets))
}

/// Read a data node at the given offset.
///
/// Returns the data bytes or `None` if this is not a data node.
pub fn read_data_at(data: &[u8], offset: u64) -> Option<Vec<u8>> {
    let off = offset as usize;
    if off + 8 > data.len() {
        return None;
    }

    let mut count_bytes = [0u8; 8];
    count_bytes.copy_from_slice(&data[off..off + 8]);
    let count_raw = i64::from_le_bytes(count_bytes);

    if count_raw >= 0 {
        return None;
    }

    let byte_len = (-count_raw) as usize;
    let data_start = off + 8;
    let data_end = data_start + byte_len;
    if data_end > data.len() {
        return None;
    }

    Some(data[data_start..data_end].to_vec())
}
