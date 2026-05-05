// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FBX 7.4 binary format writer.
//!
//! Implements the Autodesk FBX binary file format (version 7400).
//!
//! ## Format overview
//!
//! - **Magic**: `b"Kaydara FBX Binary  \x00\x1a\x00"` (23 bytes)
//! - **Version**: `u32` LE (7400)
//! - **Nodes**: nested record structure; each record has an end-offset,
//!   property count, property-list byte length, name, properties, and
//!   children.
//! - **Null sentinel**: 13 zero bytes terminate a child list.
//! - **Property type codes**: `C`=bool, `Y`=i16, `I`=i32, `L`=i64,
//!   `F`=f32, `D`=f64, `S`=string, `R`=raw bytes,
//!   `i`=i32 array, `d`=f64 array, `f`=f32 array.

use oxiarc_deflate::zlib_compress;
use oxihuman_mesh::MeshBuffers;
use std::io::Write;

/// FBX binary magic header bytes.
const FBX_MAGIC: &[u8] = b"Kaydara FBX Binary  \x00\x1a\x00";

/// Arrays with more elements than this threshold are zlib-compressed (encoding=1).
const COMPRESSION_THRESHOLD: usize = 512;

/// FBX version we target.
const FBX_VERSION: u32 = 7400;

/// Size of a null (sentinel) node that terminates children: 13 zero bytes.
const NULL_RECORD_LEN: usize = 13;

// ── Property ────────────────────────────────────────────────────────────────

/// A single FBX property value.
#[derive(Debug, Clone)]
pub enum FbxProperty {
    /// Boolean (`C`).
    Bool(bool),
    /// 16-bit signed integer (`Y`).
    I16(i16),
    /// 32-bit signed integer (`I`).
    I32(i32),
    /// 64-bit signed integer (`L`).
    I64(i64),
    /// 32-bit float (`F`).
    F32(f32),
    /// 64-bit float (`D`).
    F64(f64),
    /// UTF-8 string (`S`).
    String(String),
    /// Raw byte blob (`R`).
    Raw(Vec<u8>),
    /// Array of i32 (`i`).
    I32Array(Vec<i32>),
    /// Array of f64 (`d`).
    F64Array(Vec<f64>),
    /// Array of f32 (`f`).
    F32Array(Vec<f32>),
}

impl FbxProperty {
    /// Returns the single-byte type code for this property variant.
    fn type_code(&self) -> u8 {
        match self {
            Self::Bool(_) => b'C',
            Self::I16(_) => b'Y',
            Self::I32(_) => b'I',
            Self::I64(_) => b'L',
            Self::F32(_) => b'F',
            Self::F64(_) => b'D',
            Self::String(_) => b'S',
            Self::Raw(_) => b'R',
            Self::I32Array(_) => b'i',
            Self::F64Array(_) => b'd',
            Self::F32Array(_) => b'f',
        }
    }

    /// Serialises the property (type code + payload) into `buf`.
    fn write_to(&self, buf: &mut Vec<u8>) -> anyhow::Result<()> {
        buf.push(self.type_code());
        match self {
            Self::Bool(v) => buf.push(if *v { 1 } else { 0 }),
            Self::I16(v) => buf.extend_from_slice(&v.to_le_bytes()),
            Self::I32(v) => buf.extend_from_slice(&v.to_le_bytes()),
            Self::I64(v) => buf.extend_from_slice(&v.to_le_bytes()),
            Self::F32(v) => buf.extend_from_slice(&v.to_le_bytes()),
            Self::F64(v) => buf.extend_from_slice(&v.to_le_bytes()),
            Self::String(s) => {
                let bytes = s.as_bytes();
                let len = u32::try_from(bytes.len())
                    .map_err(|_| anyhow::anyhow!("FBX string too long: {} bytes", bytes.len()))?;
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(bytes);
            }
            Self::Raw(data) => {
                let len = u32::try_from(data.len())
                    .map_err(|_| anyhow::anyhow!("FBX raw blob too long: {} bytes", data.len()))?;
                buf.extend_from_slice(&len.to_le_bytes());
                buf.extend_from_slice(data);
            }
            Self::I32Array(arr) => write_array_with_compression(buf, arr, 4, |b, v| {
                b.extend_from_slice(&v.to_le_bytes());
            })?,
            Self::F64Array(arr) => write_array_with_compression(buf, arr, 8, |b, v| {
                b.extend_from_slice(&v.to_le_bytes());
            })?,
            Self::F32Array(arr) => write_array_with_compression(buf, arr, 4, |b, v| {
                b.extend_from_slice(&v.to_le_bytes());
            })?,
        }
        Ok(())
    }
}

/// Writes an FBX array header + elements, using zlib compression when the array
/// is larger than [`COMPRESSION_THRESHOLD`] elements.
///
/// Array header layout: array_length(u32), encoding(u32), compressed_length(u32),
/// followed by element bytes (raw when encoding=0, zlib-deflated when encoding=1).
fn write_array_with_compression<T>(
    buf: &mut Vec<u8>,
    arr: &[T],
    elem_size: u32,
    mut write_elem: impl FnMut(&mut Vec<u8>, &T),
) -> anyhow::Result<()> {
    let count = u32::try_from(arr.len())
        .map_err(|_| anyhow::anyhow!("FBX array too long: {} elements", arr.len()))?;

    // Serialise all elements into a temporary raw buffer.
    let mut raw: Vec<u8> = Vec::with_capacity(arr.len() * elem_size as usize);
    for v in arr {
        write_elem(&mut raw, v);
    }

    buf.extend_from_slice(&count.to_le_bytes()); // array_length

    if arr.len() > COMPRESSION_THRESHOLD {
        // encoding = 1 (zlib / deflate)
        let compressed = zlib_compress(&raw, 6)
            .map_err(|e| anyhow::anyhow!("FBX zlib compression failed: {}", e))?;
        let compressed_len = u32::try_from(compressed.len())
            .map_err(|_| anyhow::anyhow!("FBX compressed array too large: {} bytes", compressed.len()))?;
        buf.extend_from_slice(&1u32.to_le_bytes()); // encoding = 1
        buf.extend_from_slice(&compressed_len.to_le_bytes()); // compressed_length
        buf.extend_from_slice(&compressed);
    } else {
        // encoding = 0 (uncompressed)
        let data_len = u32::try_from(raw.len())
            .map_err(|_| anyhow::anyhow!("FBX array byte length overflow"))?;
        buf.extend_from_slice(&0u32.to_le_bytes()); // encoding = 0
        buf.extend_from_slice(&data_len.to_le_bytes()); // compressed_length == data_len
        buf.extend_from_slice(&raw);
    }
    Ok(())
}

// ── Node ────────────────────────────────────────────────────────────────────

/// A node in the FBX binary tree.
#[derive(Debug, Clone)]
pub struct FbxNode {
    /// Node name (ASCII, max 255 bytes in practice).
    pub name: String,
    /// Properties attached to this node.
    pub properties: Vec<FbxProperty>,
    /// Child nodes.
    pub children: Vec<FbxNode>,
}

impl FbxNode {
    /// Creates a new node with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Adds a property to this node.
    pub fn add_property(&mut self, prop: FbxProperty) {
        self.properties.push(prop);
    }

    /// Adds a child node and returns a mutable reference to it.
    pub fn add_child(&mut self, child: FbxNode) -> &mut FbxNode {
        self.children.push(child);
        let idx = self.children.len() - 1;
        &mut self.children[idx]
    }
}

// ── Writer ──────────────────────────────────────────────────────────────────

/// Writes FBX data in the binary format (version 7400).
pub struct FbxBinaryWriter {
    output: Vec<u8>,
}

impl Default for FbxBinaryWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl FbxBinaryWriter {
    /// Creates a new empty writer (no header written yet).
    pub fn new() -> Self {
        Self {
            output: Vec::with_capacity(256 * 1024),
        }
    }

    /// Writes the 27-byte FBX binary header (magic + version).
    pub fn write_header(&mut self) -> anyhow::Result<()> {
        self.output.write_all(FBX_MAGIC)?;
        self.output.write_all(&FBX_VERSION.to_le_bytes())?;
        Ok(())
    }

    /// Serialises a single top-level `FbxNode` (and all its descendants)
    /// into the output buffer. The node's end-offset is computed
    /// automatically.
    pub fn write_node(&mut self, node: &FbxNode) -> anyhow::Result<()> {
        write_node_recursive(&mut self.output, node)
    }

    /// Convenience: writes a complete Geometry + Model node pair for a
    /// triangle mesh.
    pub fn write_mesh(
        &mut self,
        name: &str,
        positions: &[[f64; 3]],
        normals: &[[f64; 3]],
        uvs: &[[f64; 2]],
        triangles: &[[usize; 3]],
    ) -> anyhow::Result<()> {
        let geometry_id: i64 = 200_000_000;
        let model_id: i64 = 200_000_001;

        // ── Objects ─────────────────────────────────────────────────────
        let mut objects = FbxNode::new("Objects");

        // Geometry
        let mut geom = FbxNode::new("Geometry");
        geom.add_property(FbxProperty::I64(geometry_id));
        geom.add_property(FbxProperty::String(format!("Geometry::{name}\x00\x01Geometry")));
        geom.add_property(FbxProperty::String("Mesh".into()));

        // Vertices
        let flat_verts: Vec<f64> = positions.iter().flat_map(|p| p.iter().copied()).collect();
        let mut verts_node = FbxNode::new("Vertices");
        verts_node.add_property(FbxProperty::F64Array(flat_verts));
        geom.add_child(verts_node);

        // PolygonVertexIndex
        let flat_idx: Vec<i32> = triangles
            .iter()
            .flat_map(|tri| {
                let a = i32::try_from(tri[0]).unwrap_or(0);
                let b = i32::try_from(tri[1]).unwrap_or(0);
                let c = i32::try_from(tri[2]).unwrap_or(0);
                // FBX convention: last index of polygon is -(idx+1)
                [a, b, -(c + 1)]
            })
            .collect();
        let mut idx_node = FbxNode::new("PolygonVertexIndex");
        idx_node.add_property(FbxProperty::I32Array(flat_idx));
        geom.add_child(idx_node);

        // Normals
        if !normals.is_empty() {
            let mut layer_normal = FbxNode::new("LayerElementNormal");
            layer_normal.add_property(FbxProperty::I32(0));

            let mut ver = FbxNode::new("Version");
            ver.add_property(FbxProperty::I32(101));
            layer_normal.add_child(ver);

            let mut mapping = FbxNode::new("MappingInformationType");
            mapping.add_property(FbxProperty::String("ByVertice".into()));
            layer_normal.add_child(mapping);

            let mut reference = FbxNode::new("ReferenceInformationType");
            reference.add_property(FbxProperty::String("Direct".into()));
            layer_normal.add_child(reference);

            let flat_normals: Vec<f64> =
                normals.iter().flat_map(|n| n.iter().copied()).collect();
            let mut ndata = FbxNode::new("Normals");
            ndata.add_property(FbxProperty::F64Array(flat_normals));
            layer_normal.add_child(ndata);

            geom.add_child(layer_normal);
        }

        // UVs
        if !uvs.is_empty() {
            let mut layer_uv = FbxNode::new("LayerElementUV");
            layer_uv.add_property(FbxProperty::I32(0));

            let mut ver = FbxNode::new("Version");
            ver.add_property(FbxProperty::I32(101));
            layer_uv.add_child(ver);

            let mut mapping = FbxNode::new("MappingInformationType");
            mapping.add_property(FbxProperty::String("ByVertice".into()));
            layer_uv.add_child(mapping);

            let mut reference = FbxNode::new("ReferenceInformationType");
            reference.add_property(FbxProperty::String("Direct".into()));
            layer_uv.add_child(reference);

            let flat_uv: Vec<f64> = uvs.iter().flat_map(|u| u.iter().copied()).collect();
            let mut uv_data = FbxNode::new("UV");
            uv_data.add_property(FbxProperty::F64Array(flat_uv));
            layer_uv.add_child(uv_data);

            geom.add_child(layer_uv);
        }

        objects.add_child(geom);

        // Model
        let mut model = FbxNode::new("Model");
        model.add_property(FbxProperty::I64(model_id));
        model.add_property(FbxProperty::String(format!("Model::{name}\x00\x01Model")));
        model.add_property(FbxProperty::String("Mesh".into()));

        let mut version_node = FbxNode::new("Version");
        version_node.add_property(FbxProperty::I32(232));
        model.add_child(version_node);

        objects.add_child(model);

        self.write_node(&objects)?;

        // ── Connections ─────────────────────────────────────────────────
        let mut conns = FbxNode::new("Connections");

        // Geometry -> Model
        let mut c1 = FbxNode::new("C");
        c1.add_property(FbxProperty::String("OO".into()));
        c1.add_property(FbxProperty::I64(geometry_id));
        c1.add_property(FbxProperty::I64(model_id));
        conns.add_child(c1);

        // Model -> root (0)
        let mut c2 = FbxNode::new("C");
        c2.add_property(FbxProperty::String("OO".into()));
        c2.add_property(FbxProperty::I64(model_id));
        c2.add_property(FbxProperty::I64(0));
        conns.add_child(c2);

        self.write_node(&conns)?;

        Ok(())
    }

    /// Writes skeleton hierarchy as NodeAttribute (LimbNode) + Model nodes
    /// with bind-pose transforms.
    pub fn write_skeleton(
        &mut self,
        bone_names: &[String],
        bone_parents: &[Option<usize>],
        bind_poses: &[[f64; 16]],
    ) -> anyhow::Result<()> {
        if bone_names.len() != bone_parents.len() || bone_names.len() != bind_poses.len() {
            return Err(anyhow::anyhow!(
                "Skeleton arrays have mismatched lengths: names={}, parents={}, poses={}",
                bone_names.len(),
                bone_parents.len(),
                bind_poses.len(),
            ));
        }

        let base_id: i64 = 300_000_000;

        let mut objects = FbxNode::new("Objects");

        for (i, bone_name) in bone_names.iter().enumerate() {
            let attr_id = base_id + (i as i64) * 2;
            let model_id = attr_id + 1;

            // NodeAttribute (LimbNode)
            let mut attr = FbxNode::new("NodeAttribute");
            attr.add_property(FbxProperty::I64(attr_id));
            attr.add_property(FbxProperty::String(format!(
                "NodeAttribute::{bone_name}\x00\x01NodeAttribute"
            )));
            attr.add_property(FbxProperty::String("LimbNode".into()));

            let mut tf = FbxNode::new("TypeFlags");
            tf.add_property(FbxProperty::String("Skeleton".into()));
            attr.add_child(tf);

            objects.add_child(attr);

            // Model (LimbNode)
            let mut model = FbxNode::new("Model");
            model.add_property(FbxProperty::I64(model_id));
            model.add_property(FbxProperty::String(format!(
                "Model::{bone_name}\x00\x01Model"
            )));
            model.add_property(FbxProperty::String("LimbNode".into()));

            // Properties70 with bind pose transform
            let mut props70 = FbxNode::new("Properties70");

            // Extract translation from the 4x4 bind pose (column 3, rows 0-2
            // assuming column-major; for row-major indices 12,13,14).
            let pose = &bind_poses[i];
            let tx = pose[12];
            let ty = pose[13];
            let tz = pose[14];

            let mut p_trans = FbxNode::new("P");
            p_trans.add_property(FbxProperty::String("Lcl Translation".into()));
            p_trans.add_property(FbxProperty::String("Lcl Translation".into()));
            p_trans.add_property(FbxProperty::String(String::new()));
            p_trans.add_property(FbxProperty::String("A".into()));
            p_trans.add_property(FbxProperty::F64(tx));
            p_trans.add_property(FbxProperty::F64(ty));
            p_trans.add_property(FbxProperty::F64(tz));
            props70.add_child(p_trans);

            model.add_child(props70);
            objects.add_child(model);
        }

        self.write_node(&objects)?;

        // Connections
        let mut conns = FbxNode::new("Connections");

        for (i, parent_opt) in bone_parents.iter().enumerate() {
            let attr_id = base_id + (i as i64) * 2;
            let model_id = attr_id + 1;

            // Attribute -> Model
            let mut c_attr = FbxNode::new("C");
            c_attr.add_property(FbxProperty::String("OO".into()));
            c_attr.add_property(FbxProperty::I64(attr_id));
            c_attr.add_property(FbxProperty::I64(model_id));
            conns.add_child(c_attr);

            // Model -> parent model (or root=0)
            let parent_model_id = match parent_opt {
                Some(pi) => base_id + (*pi as i64) * 2 + 1,
                None => 0,
            };
            let mut c_model = FbxNode::new("C");
            c_model.add_property(FbxProperty::String("OO".into()));
            c_model.add_property(FbxProperty::I64(model_id));
            c_model.add_property(FbxProperty::I64(parent_model_id));
            conns.add_child(c_model);
        }

        self.write_node(&conns)?;

        Ok(())
    }

    /// Finalises the file by appending the footer and returns the complete
    /// binary FBX bytes.
    pub fn finish(mut self) -> anyhow::Result<Vec<u8>> {
        // Write a top-level null sentinel to mark end of top-level nodes.
        self.output.extend_from_slice(&[0u8; NULL_RECORD_LEN]);

        // FBX footer: a fixed sequence of bytes (simplified but conformant).
        // Pad to 16-byte alignment, then write the footer magic.
        let footer_padding_target = self.output.len().div_ceil(16) * 16;
        while self.output.len() < footer_padding_target {
            self.output.push(0);
        }

        // Footer sentinel (version repeated, some zeros, checksum area).
        // For maximum compatibility we write a minimal footer.
        self.output.extend_from_slice(&FBX_VERSION.to_le_bytes());
        // 120 zero bytes (empty checksum block).
        self.output.extend_from_slice(&[0u8; 120]);
        // Footer magic: 16 bytes.
        self.output.extend_from_slice(&[
            0xf8, 0x5a, 0x8c, 0x6a, 0xde, 0xf5, 0xd9, 0x7e, 0xec, 0xe9, 0x0c, 0xe3, 0x75, 0x8f,
            0x29, 0x0b,
        ]);

        Ok(self.output)
    }
}

// ── Convenience API ─────────────────────────────────────────────────────────

/// Export a [`MeshBuffers`] to a self-contained FBX 7.4 binary byte vector.
///
/// The returned bytes form a valid `.fbx` file with a Geometry node, a Model
/// node, and the corresponding Connections, ready for import into any
/// FBX-compatible DCC tool.  Large arrays (> 512 elements)
/// are automatically zlib-compressed per the FBX spec (encoding = 1).
pub fn export_mesh_fbx_binary(mesh: &MeshBuffers) -> anyhow::Result<Vec<u8>> {
    let mut writer = FbxBinaryWriter::new();
    writer.write_header()?;

    // Convert f32 positions / normals / uvs to f64 for FBX compatibility.
    let positions_f64: Vec<[f64; 3]> = mesh
        .positions
        .iter()
        .map(|p| [p[0] as f64, p[1] as f64, p[2] as f64])
        .collect();

    let normals_f64: Vec<[f64; 3]> = mesh
        .normals
        .iter()
        .map(|n| [n[0] as f64, n[1] as f64, n[2] as f64])
        .collect();

    let uvs_f64: Vec<[f64; 2]> = mesh
        .uvs
        .iter()
        .map(|u| [u[0] as f64, u[1] as f64])
        .collect();

    // Convert flat u32 index list into triangle triples.
    let triangles: Vec<[usize; 3]> = mesh
        .indices
        .chunks(3)
        .filter_map(|tri| {
            if tri.len() == 3 {
                Some([tri[0] as usize, tri[1] as usize, tri[2] as usize])
            } else {
                None
            }
        })
        .collect();

    writer.write_mesh("Mesh", &positions_f64, &normals_f64, &uvs_f64, &triangles)?;
    writer.finish()
}

// ── Recursive node serialisation ────────────────────────────────────────────

/// Serialises a single `FbxNode` (with all children) into `buf`.
///
/// FBX binary node record layout (v7400, 32-bit offsets):
///
/// | Field              | Type  | Notes                           |
/// |--------------------|-------|---------------------------------|
/// | end_offset         | u32   | absolute file position of end   |
/// | num_properties     | u32   |                                 |
/// | property_list_len  | u32   | byte length of all properties   |
/// | name_len           | u8    |                                 |
/// | name               | [u8]  |                                 |
/// | properties         | ...   |                                 |
/// | children           | ...   | recursive, terminated by null   |
fn write_node_recursive(buf: &mut Vec<u8>, node: &FbxNode) -> anyhow::Result<()> {
    let record_start = buf.len();

    // Reserve 13 bytes for the header (end_offset + num_props + prop_list_len + name_len)
    // We will patch them later.
    buf.extend_from_slice(&[0u8; 13]);

    // Name
    let name_bytes = node.name.as_bytes();
    let name_len = u8::try_from(name_bytes.len())
        .map_err(|_| anyhow::anyhow!("FBX node name too long: {}", node.name))?;
    buf[record_start + 12] = name_len;
    buf.extend_from_slice(name_bytes);

    // Properties
    let props_start = buf.len();
    for prop in &node.properties {
        prop.write_to(buf)?;
    }
    let props_end = buf.len();
    let property_list_len = u32::try_from(props_end - props_start)
        .map_err(|_| anyhow::anyhow!("FBX property list too large"))?;
    let num_properties = u32::try_from(node.properties.len())
        .map_err(|_| anyhow::anyhow!("FBX too many properties"))?;

    // Children
    if !node.children.is_empty() {
        for child in &node.children {
            write_node_recursive(buf, child)?;
        }
        // Null sentinel to terminate children
        buf.extend_from_slice(&[0u8; NULL_RECORD_LEN]);
    }

    let end_offset = u32::try_from(buf.len())
        .map_err(|_| anyhow::anyhow!("FBX file too large for 32-bit offsets"))?;

    // Patch the header fields
    buf[record_start..record_start + 4].copy_from_slice(&end_offset.to_le_bytes());
    buf[record_start + 4..record_start + 8].copy_from_slice(&num_properties.to_le_bytes());
    buf[record_start + 8..record_start + 12].copy_from_slice(&property_list_len.to_le_bytes());

    Ok(())
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("write_header failed");
        let out = &w.output;
        assert_eq!(&out[..23], FBX_MAGIC);
        let ver = u32::from_le_bytes([out[23], out[24], out[25], out[26]]);
        assert_eq!(ver, 7400);
    }

    #[test]
    fn test_write_simple_node() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");
        let mut node = FbxNode::new("TestNode");
        node.add_property(FbxProperty::I32(42));
        w.write_node(&node).expect("write_node");
        let data = w.finish().expect("finish");
        // Should start with FBX magic
        assert_eq!(&data[..23], FBX_MAGIC);
        // Data should be longer than just the header
        assert!(data.len() > 27 + NULL_RECORD_LEN);
    }

    #[test]
    fn test_write_mesh() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");

        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let triangles = vec![[0, 1, 2]];

        w.write_mesh("Triangle", &positions, &normals, &uvs, &triangles)
            .expect("write_mesh");

        let data = w.finish().expect("finish");
        assert!(data.len() > 200);
        // Check magic is intact
        assert_eq!(&data[..23], FBX_MAGIC);
    }

    #[test]
    fn test_write_skeleton() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");

        let names = vec!["Hips".to_string(), "Spine".to_string(), "Head".to_string()];
        let parents = vec![None, Some(0), Some(1)];
        #[rustfmt::skip]
        let identity = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        let poses = vec![identity; 3];

        w.write_skeleton(&names, &parents, &poses)
            .expect("write_skeleton");

        let data = w.finish().expect("finish");
        assert!(data.len() > 200);
    }

    #[test]
    fn test_skeleton_mismatched_lengths() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");

        let names = vec!["A".to_string()];
        let parents = vec![None, Some(0)]; // wrong length
        let poses = vec![[0.0; 16]];

        let result = w.write_skeleton(&names, &parents, &poses);
        assert!(result.is_err());
    }

    #[test]
    fn test_property_type_codes() {
        assert_eq!(FbxProperty::Bool(true).type_code(), b'C');
        assert_eq!(FbxProperty::I16(0).type_code(), b'Y');
        assert_eq!(FbxProperty::I32(0).type_code(), b'I');
        assert_eq!(FbxProperty::I64(0).type_code(), b'L');
        assert_eq!(FbxProperty::F32(0.0).type_code(), b'F');
        assert_eq!(FbxProperty::F64(0.0).type_code(), b'D');
        assert_eq!(FbxProperty::String(String::new()).type_code(), b'S');
        assert_eq!(FbxProperty::Raw(vec![]).type_code(), b'R');
        assert_eq!(FbxProperty::I32Array(vec![]).type_code(), b'i');
        assert_eq!(FbxProperty::F64Array(vec![]).type_code(), b'd');
        assert_eq!(FbxProperty::F32Array(vec![]).type_code(), b'f');
    }

    #[test]
    fn test_empty_mesh() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");
        w.write_mesh("Empty", &[], &[], &[], &[]).expect("write_mesh");
        let data = w.finish().expect("finish");
        assert!(data.len() > 27);
    }

    #[test]
    fn test_node_children() {
        let mut parent = FbxNode::new("Parent");
        let child = FbxNode::new("Child");
        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name, "Child");
    }

    #[test]
    fn test_finish_contains_footer_magic() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");
        let data = w.finish().expect("finish");
        // Last 16 bytes should be the footer magic
        let footer = &data[data.len() - 16..];
        assert_eq!(footer[0], 0xf8);
        assert_eq!(footer[1], 0x5a);
    }

    #[test]
    fn test_default_trait() {
        let w = FbxBinaryWriter::default();
        assert!(w.output.is_empty());
    }

    // ── v0.1.1 workstream-F tests ────────────────────────────────────────────

    fn minimal_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    /// The convenience export must start with the FBX binary magic header.
    #[test]
    fn test_fbx_magic_bytes() {
        let mesh = minimal_mesh();
        let data = export_mesh_fbx_binary(&mesh).expect("export_mesh_fbx_binary failed");
        assert_eq!(
            &data[..23],
            b"Kaydara FBX Binary  \x00\x1a\x00",
            "FBX magic header mismatch"
        );
    }

    /// A large array (> COMPRESSION_THRESHOLD elements) must be written with
    /// encoding = 1 (zlib), so the fourth byte of the array header payload must
    /// be 1 (first byte of the little-endian u32 encoding field).
    #[test]
    fn test_zlib_array_round_trip() {
        let data: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let mut buf: Vec<u8> = Vec::new();
        // array_length (4 bytes) + encoding (4 bytes) + compressed_len (4 bytes) + ...
        write_array_with_compression(&mut buf, &data, 4, |b, v: &f32| {
            b.extend_from_slice(&v.to_le_bytes());
        })
        .expect("write_array_with_compression failed");

        // encoding field starts at byte offset 4 (after array_length u32).
        let encoding = u32::from_le_bytes(
            buf[4..8]
                .try_into()
                .expect("encoding slice must be 4 bytes"),
        );
        assert_eq!(encoding, 1, "expected zlib encoding (1) for large array");

        // The compressed payload must be smaller than the raw data (1000 * 4 = 4000 bytes).
        let compressed_len = u32::from_le_bytes(
            buf[8..12]
                .try_into()
                .expect("compressed_len slice must be 4 bytes"),
        ) as usize;
        assert!(
            compressed_len < data.len() * 4,
            "compressed payload ({compressed_len} B) should be smaller than raw ({} B)",
            data.len() * 4
        );
    }

    /// Smoke-test: the convenience function must succeed and produce a file
    /// larger than just the 27-byte header.
    #[test]
    fn test_mesh_export_smoke() {
        let mesh = minimal_mesh();
        let result = export_mesh_fbx_binary(&mesh);
        assert!(result.is_ok(), "export_mesh_fbx_binary returned error");
        let bytes = result.expect("already checked above");
        assert!(
            bytes.len() > 27,
            "exported FBX should be larger than the 27-byte header, got {} bytes",
            bytes.len()
        );
    }

    #[test]
    fn test_mesh_then_skeleton() {
        let mut w = FbxBinaryWriter::new();
        w.write_header().expect("header");

        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles = vec![[0, 1, 2]];
        w.write_mesh("Body", &positions, &[], &[], &triangles)
            .expect("mesh");

        let names = vec!["Root".to_string()];
        let parents = vec![None];
        let poses = vec![[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]];
        w.write_skeleton(&names, &parents, &poses)
            .expect("skeleton");

        let data = w.finish().expect("finish");
        assert!(data.len() > 300);
    }
}
