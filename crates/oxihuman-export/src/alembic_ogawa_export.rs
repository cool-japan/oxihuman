// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Alembic Ogawa binary format writer.
//!
//! Produces valid `.abc` files using the Ogawa binary container format.
//! Supports mesh, subdivision surface, transform (Xform), and camera objects,
//! including animated (time-sampled) data.
//!
//! # Ogawa format overview
//!
//! - **Magic**: `0xFF 0x00 0x00 0x00 0x00 0x01 0x00 0x00` (8 bytes)
//! - **Root group offset**: u64 LE immediately after magic
//! - **Groups**: `count` (u64 LE), then `count` offsets (each u64 LE) to children
//! - **Data**: negative count means data; `abs(count)` = byte size, followed by raw bytes
//! - All offsets are absolute from start of file

use anyhow::{bail, ensure, Result};

// ── Ogawa container primitives ──────────────────────────────────────────────

/// Magic bytes that identify an Ogawa (Alembic) file.
const OGAWA_MAGIC: [u8; 8] = [0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];

/// A child entry inside an [`OgawaGroup`].
#[derive(Debug, Clone)]
enum OgawaChild {
    /// Reference to another group by index.
    Group(usize),
    /// Inline raw data blob.
    Data(Vec<u8>),
}

/// A group node in the Ogawa container tree.
#[derive(Debug, Clone)]
struct OgawaGroup {
    children: Vec<OgawaChild>,
}

impl OgawaGroup {
    fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    fn add_data(&mut self, data: Vec<u8>) {
        self.children.push(OgawaChild::Data(data));
    }

    fn add_group(&mut self, idx: usize) {
        self.children.push(OgawaChild::Group(idx));
    }
}

// ── Public data types ───────────────────────────────────────────────────────

/// Top-level writer that accumulates Alembic objects and serialises them
/// to the Ogawa binary format.
pub struct AlembicWriter {
    time_sampling: Option<TimeSampling>,
    objects: Vec<AbcObject>,
}

/// Time sampling configuration.
#[derive(Debug, Clone)]
struct TimeSampling {
    start: f64,
    dt: f64,
    num_samples: usize,
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

const ABC_CORE_VERSION: u32 = 1;

// ── Encoding helpers ────────────────────────────────────────────────────────

fn encode_u64(val: u64) -> [u8; 8] {
    val.to_le_bytes()
}

fn encode_i64(val: i64) -> [u8; 8] {
    val.to_le_bytes()
}

fn encode_u32(val: u32) -> [u8; 4] {
    val.to_le_bytes()
}

fn encode_i32(val: i32) -> [u8; 4] {
    val.to_le_bytes()
}

fn encode_f64(val: f64) -> [u8; 8] {
    val.to_le_bytes()
}

fn encode_f64x3_slice(vals: &[[f64; 3]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 24);
    for v in vals {
        buf.extend_from_slice(&encode_f64(v[0]));
        buf.extend_from_slice(&encode_f64(v[1]));
        buf.extend_from_slice(&encode_f64(v[2]));
    }
    buf
}

fn encode_f64x2_slice(vals: &[[f64; 2]]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 16);
    for v in vals {
        buf.extend_from_slice(&encode_f64(v[0]));
        buf.extend_from_slice(&encode_f64(v[1]));
    }
    buf
}

fn encode_i32_slice(vals: &[i32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 4);
    for &v in vals {
        buf.extend_from_slice(&encode_i32(v));
    }
    buf
}

fn encode_f64_slice(vals: &[f64]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(vals.len() * 8);
    for &v in vals {
        buf.extend_from_slice(&encode_f64(v));
    }
    buf
}

fn encode_string(s: &str) -> Vec<u8> {
    let mut buf = Vec::with_capacity(s.len() + 1);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    buf
}

fn encode_matrix(m: &[f64; 16]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);
    for &v in m {
        buf.extend_from_slice(&encode_f64(v));
    }
    buf
}

// ── AlembicWriter implementation ────────────────────────────────────────────

impl AlembicWriter {
    /// Create a new empty writer.
    pub fn new() -> Self {
        Self {
            time_sampling: None,
            objects: Vec::new(),
        }
    }

    /// Add an object (and its subtree) to the archive.
    pub fn add_object(&mut self, obj: &AbcObject) -> Result<()> {
        validate_object(obj)?;
        self.objects.push(obj.clone());
        Ok(())
    }

    /// Configure time sampling for animated data.
    ///
    /// - `start`: start time in seconds
    /// - `dt`: time step between samples
    /// - `num_samples`: total number of time samples
    pub fn set_time_sampling(&mut self, start: f64, dt: f64, num_samples: usize) -> Result<()> {
        ensure!(dt > 0.0, "time step `dt` must be positive, got {}", dt);
        ensure!(
            num_samples > 0,
            "num_samples must be at least 1, got {}",
            num_samples
        );
        self.time_sampling = Some(TimeSampling {
            start,
            dt,
            num_samples,
        });
        Ok(())
    }

    /// Serialize all added objects into a valid Ogawa binary `.abc` file.
    pub fn export(&self) -> Result<Vec<u8>> {
        ensure!(!self.objects.is_empty(), "no objects added to export");

        let mut ctx = WriteContext { groups: Vec::new() };

        // Build the archive metadata group
        let meta_group_idx = build_archive_metadata(&mut ctx)?;

        // Build time sampling group
        let ts_group_idx = build_time_sampling(&mut ctx, &self.time_sampling)?;

        // Build object hierarchy starting from root
        let mut child_group_indices = Vec::new();
        for obj in &self.objects {
            let idx = build_object_group(&mut ctx, obj, &self.time_sampling)?;
            child_group_indices.push(idx);
        }

        // Build root object group (contains all top-level objects)
        let root_obj_idx = {
            let mut root_group = OgawaGroup::new();
            root_group.add_data(encode_string(""));
            root_group.add_data(encode_string("AbcObject_v1"));
            root_group.add_group(meta_group_idx);
            root_group.add_group(ts_group_idx);
            for &child_idx in &child_group_indices {
                root_group.add_group(child_idx);
            }
            ctx.alloc_group(root_group)
        };

        // Build the file-level root group
        let file_root_idx = {
            let mut file_root = OgawaGroup::new();
            let mut ver_data = Vec::with_capacity(4);
            ver_data.extend_from_slice(&encode_u32(ABC_CORE_VERSION));
            file_root.add_data(ver_data);
            file_root.add_group(root_obj_idx);
            ctx.alloc_group(file_root)
        };

        serialize_ogawa(&ctx.groups, file_root_idx)
    }
}

impl Default for AlembicWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ── Internal build context ──────────────────────────────────────────────────

struct WriteContext {
    groups: Vec<OgawaGroup>,
}

impl WriteContext {
    fn alloc_group(&mut self, g: OgawaGroup) -> usize {
        let idx = self.groups.len();
        self.groups.push(g);
        idx
    }
}

// ── Validation ──────────────────────────────────────────────────────────────

fn validate_object(obj: &AbcObject) -> Result<()> {
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

fn build_archive_metadata(ctx: &mut WriteContext) -> Result<usize> {
    let mut group = OgawaGroup::new();
    group.add_data(encode_string("oxihuman-export"));
    group.add_data(encode_string("Alembic 1.8 (Ogawa)"));
    group.add_data(encode_string("2026-03-11"));
    Ok(ctx.alloc_group(group))
}

fn build_time_sampling(ctx: &mut WriteContext, ts: &Option<TimeSampling>) -> Result<usize> {
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

fn build_object_group(
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
fn serialize_ogawa(groups: &[OgawaGroup], root_idx: usize) -> Result<Vec<u8>> {
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
fn topological_order(groups: &[OgawaGroup], root: usize) -> Result<Vec<usize>> {
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

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ogawa_magic_valid() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cube".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_ogawa_magic_invalid() {
        assert!(!validate_ogawa_magic(b"NOT_ABC\x00"));
    }

    #[test]
    fn test_root_offset_present() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "test".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        let offset = read_root_offset(&data).unwrap();
        assert!(offset >= 16, "root offset should be past the header");
        assert!((offset as usize) < data.len(), "root offset within file");
    }

    #[test]
    fn test_root_group_readable() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "mesh".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        let offset = read_root_offset(&data).unwrap();
        let (count, _offsets) = read_group_at(&data, offset).unwrap();
        assert!(count >= 2, "root group child count: {}", count);
    }

    #[test]
    fn test_export_xform() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "xform_node".into(),
                kind: AbcObjectKind::Xform(AbcXform {
                    matrix: translation_matrix(1.0, 2.0, 3.0),
                    animated_matrices: vec![],
                }),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 16);
    }

    #[test]
    fn test_export_camera() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cam".into(),
                kind: AbcObjectKind::Camera(AbcCamera {
                    focal_length: 50.0,
                    near_clip: 0.1,
                    far_clip: 1000.0,
                    horizontal_aperture: 3.6,
                    vertical_aperture: 2.4,
                }),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_subd() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "subd_mesh".into(),
                kind: AbcObjectKind::SubD(AbcSubD {
                    positions: vec![
                        [-1.0, -1.0, 0.0],
                        [1.0, -1.0, 0.0],
                        [1.0, 1.0, 0.0],
                        [-1.0, 1.0, 0.0],
                    ],
                    face_counts: vec![4],
                    face_indices: vec![0, 1, 2, 3],
                    crease_indices: vec![0, 1],
                    crease_lengths: vec![2],
                    crease_sharpnesses: vec![3.0],
                }),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_with_normals_and_uvs() {
        let mut mesh = unit_cube_polymesh();
        mesh.normals = Some(vec![[0.0, 0.0, 1.0]; 8]);
        mesh.uvs = Some(vec![[0.0, 0.0]; 8]);

        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "textured".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 500);
    }

    #[test]
    fn test_export_animated_mesh() {
        let base = unit_cube_polymesh();
        let animated_positions = vec![
            (0.0, base.positions.clone()),
            (
                1.0 / 24.0,
                base.positions
                    .iter()
                    .map(|p| [p[0] + 0.1, p[1], p[2]])
                    .collect(),
            ),
        ];
        let mesh = AbcPolyMesh {
            animated_positions,
            ..base
        };

        let mut writer = AlembicWriter::new();
        writer.set_time_sampling(0.0, 1.0 / 24.0, 2).unwrap();
        writer
            .add_object(&AbcObject {
                name: "anim_cube".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_export_animated_xform() {
        let mut writer = AlembicWriter::new();
        writer.set_time_sampling(0.0, 1.0 / 24.0, 3).unwrap();
        writer
            .add_object(&AbcObject {
                name: "moving".into(),
                kind: AbcObjectKind::Xform(AbcXform {
                    matrix: identity_matrix(),
                    animated_matrices: vec![
                        (0.0, translation_matrix(0.0, 0.0, 0.0)),
                        (1.0 / 24.0, translation_matrix(1.0, 0.0, 0.0)),
                        (2.0 / 24.0, translation_matrix(2.0, 0.0, 0.0)),
                    ],
                }),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_hierarchy() {
        let mesh = AbcObject {
            name: "body".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        };
        let xform = AbcObject {
            name: "root_xform".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: identity_matrix(),
                animated_matrices: vec![],
            }),
            children: vec![mesh],
        };

        let mut writer = AlembicWriter::new();
        writer.add_object(&xform).unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 200);
    }

    #[test]
    fn test_multiple_objects() {
        let mut writer = AlembicWriter::new();
        for i in 0..5 {
            writer
                .add_object(&AbcObject {
                    name: format!("mesh_{}", i),
                    kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                    children: vec![],
                })
                .unwrap();
        }
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
    }

    #[test]
    fn test_empty_writer_fails() {
        let writer = AlembicWriter::new();
        assert!(writer.export().is_err());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_face_indices_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad".into(),
            kind: AbcObjectKind::PolyMesh(AbcPolyMesh {
                positions: vec![[0.0, 0.0, 0.0]; 3],
                face_counts: vec![3],
                face_indices: vec![0, 1],
                normals: None,
                uvs: None,
                animated_positions: vec![],
            }),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_camera_rejected() {
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad_cam".into(),
            kind: AbcObjectKind::Camera(AbcCamera {
                focal_length: -10.0,
                near_clip: 0.1,
                far_clip: 100.0,
                horizontal_aperture: 3.6,
                vertical_aperture: 2.4,
            }),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_time_sampling_rejected() {
        let mut writer = AlembicWriter::new();
        assert!(writer.set_time_sampling(0.0, 0.0, 10).is_err());
        assert!(writer.set_time_sampling(0.0, 1.0, 0).is_err());
    }

    #[test]
    fn test_data_node_read() {
        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "obj".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();

        let root_off = read_root_offset(&data).unwrap();
        let (count, offsets) = read_group_at(&data, root_off).unwrap();
        assert!(count > 0);

        // First child of root should be version data
        let version_data = read_data_at(&data, offsets[0]);
        assert!(version_data.is_some());
        let vd = version_data.unwrap();
        assert_eq!(vd.len(), 4);
        let ver = u32::from_le_bytes([vd[0], vd[1], vd[2], vd[3]]);
        assert_eq!(ver, ABC_CORE_VERSION);
    }

    #[test]
    fn test_identity_matrix_values() {
        let m = identity_matrix();
        assert!((m[0] - 1.0).abs() < f64::EPSILON);
        assert!((m[5] - 1.0).abs() < f64::EPSILON);
        assert!((m[10] - 1.0).abs() < f64::EPSILON);
        assert!((m[15] - 1.0).abs() < f64::EPSILON);
        assert!((m[1]).abs() < f64::EPSILON);
        assert!((m[4]).abs() < f64::EPSILON);
    }

    #[test]
    fn test_translation_matrix_values() {
        let m = translation_matrix(5.0, 10.0, 15.0);
        assert!((m[12] - 5.0).abs() < f64::EPSILON);
        assert!((m[13] - 10.0).abs() < f64::EPSILON);
        assert!((m[14] - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scale_matrix_values() {
        let m = scale_matrix(2.5);
        assert!((m[0] - 2.5).abs() < f64::EPSILON);
        assert!((m[5] - 2.5).abs() < f64::EPSILON);
        assert!((m[10] - 2.5).abs() < f64::EPSILON);
        assert!((m[15] - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mismatched_animated_positions_rejected() {
        let mut mesh = unit_cube_polymesh();
        mesh.animated_positions = vec![(0.0, mesh.positions.clone()), (1.0, vec![[0.0; 3]; 5])];
        let mut writer = AlembicWriter::new();
        let result = writer.add_object(&AbcObject {
            name: "bad_anim".into(),
            kind: AbcObjectKind::PolyMesh(mesh),
            children: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_mismatched_normals_rejected() {
        let mut mesh = unit_cube_polymesh();
        mesh.normals = Some(vec![[0.0, 0.0, 1.0]; 3]);
        let mut writer = AlembicWriter::new();
        assert!(writer
            .add_object(&AbcObject {
                name: "bad_n".into(),
                kind: AbcObjectKind::PolyMesh(mesh),
                children: vec![],
            })
            .is_err());
    }

    #[test]
    fn test_file_to_disk_round_trip() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_alembic_ogawa_export.abc");

        let mut writer = AlembicWriter::new();
        writer
            .add_object(&AbcObject {
                name: "cube".into(),
                kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
                children: vec![],
            })
            .unwrap();
        let data = writer.export().unwrap();

        std::fs::write(&path, &data).unwrap();
        let read_back = std::fs::read(&path).unwrap();
        assert_eq!(data, read_back);
        assert!(validate_ogawa_magic(&read_back));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_nested_hierarchy_deep() {
        let leaf = AbcObject {
            name: "leaf_mesh".into(),
            kind: AbcObjectKind::PolyMesh(unit_cube_polymesh()),
            children: vec![],
        };
        let level2 = AbcObject {
            name: "level2".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: translation_matrix(0.0, 1.0, 0.0),
                animated_matrices: vec![],
            }),
            children: vec![leaf],
        };
        let level1 = AbcObject {
            name: "level1".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: scale_matrix(2.0),
                animated_matrices: vec![],
            }),
            children: vec![level2],
        };
        let root = AbcObject {
            name: "scene_root".into(),
            kind: AbcObjectKind::Xform(AbcXform {
                matrix: identity_matrix(),
                animated_matrices: vec![],
            }),
            children: vec![level1],
        };

        let mut writer = AlembicWriter::new();
        writer.add_object(&root).unwrap();
        let data = writer.export().unwrap();
        assert!(validate_ogawa_magic(&data));
        assert!(data.len() > 300);
    }

    #[test]
    fn test_unit_cube_topology() {
        let mesh = unit_cube_polymesh();
        assert_eq!(mesh.positions.len(), 8);
        assert_eq!(mesh.face_counts.len(), 6);
        assert!(mesh.face_counts.iter().all(|&c| c == 4));
        assert_eq!(mesh.face_indices.len(), 24);
    }
}
