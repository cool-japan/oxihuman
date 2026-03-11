// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::io::Write;
use std::path::Path;

use anyhow::Result;
use bytemuck::cast_slice;
use oxihuman_mesh::MeshBuffers;
use serde_json::json;

// GLB magic constants (same as glb.rs / scene.rs)
const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F534A; // "JSON"
const CHUNK_BIN: u32 = 0x004E4942; // "BIN\0"

// ── Transform ────────────────────────────────────────────────────────────────

/// A 4×4 column-major transform matrix (identity by default).
#[derive(Debug, Clone)]
pub struct Transform {
    pub matrix: [f32; 16],
}

impl Transform {
    /// Return the 4×4 identity matrix.
    pub fn identity() -> Self {
        #[rustfmt::skip]
        let m = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        Self { matrix: m }
    }

    /// Translation transform: moves by (x, y, z).
    /// Column-major layout means the translation lives in column 3 (indices 12..14).
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        #[rustfmt::skip]
        let m = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
              x,   y,   z, 1.0,
        ];
        Self { matrix: m }
    }

    /// Uniform-scale transform.
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        #[rustfmt::skip]
        let m = [
             sx, 0.0, 0.0, 0.0,
            0.0,  sy, 0.0, 0.0,
            0.0, 0.0,  sz, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        Self { matrix: m }
    }

    /// Matrix multiply: `self * other` (column-major 4×4).
    pub fn compose(&self, other: &Transform) -> Transform {
        let a = &self.matrix;
        let b = &other.matrix;
        let mut c = [0.0f32; 16];
        // c[col*4 + row] = sum_k a[k*4 + row] * b[col*4 + k]
        for col in 0..4usize {
            for row in 0..4usize {
                let mut s = 0.0f32;
                for k in 0..4usize {
                    s += a[k * 4 + row] * b[col * 4 + k];
                }
                c[col * 4 + row] = s;
            }
        }
        Transform { matrix: c }
    }

    /// Returns true iff the matrix is (approximately) the identity.
    pub fn is_identity(&self) -> bool {
        let id = Self::identity();
        self.matrix
            .iter()
            .zip(id.matrix.iter())
            .all(|(a, b)| (a - b).abs() < 1e-6)
    }
}

// ── SceneNode ─────────────────────────────────────────────────────────────────

/// A node in the scene graph.
pub struct SceneNode {
    pub name: String,
    pub transform: Transform,
    pub mesh: Option<MeshBuffers>,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    /// Create a new node with the given name, identity transform, no mesh, no children.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::identity(),
            mesh: None,
            children: Vec::new(),
        }
    }

    /// Builder-style: set the transform.
    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    /// Builder-style: attach a mesh.
    pub fn with_mesh(mut self, mesh: MeshBuffers) -> Self {
        self.mesh = Some(mesh);
        self
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: SceneNode) {
        self.children.push(child);
    }

    /// Count total nodes: self plus all descendants (recursive).
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Count nodes that carry a mesh: self (if mesh is Some) plus all descendants.
    pub fn mesh_count(&self) -> usize {
        let self_has = if self.mesh.is_some() { 1 } else { 0 };
        self_has + self.children.iter().map(|c| c.mesh_count()).sum::<usize>()
    }

    /// Collect all node names in depth-first (pre-order) traversal.
    pub fn all_names(&self) -> Vec<String> {
        let mut names = vec![self.name.clone()];
        for child in &self.children {
            names.extend(child.all_names());
        }
        names
    }
}

// ── SceneGraph ────────────────────────────────────────────────────────────────

/// A scene graph with a single root node.
pub struct SceneGraph {
    pub root: SceneNode,
}

impl SceneGraph {
    /// Create a new scene graph whose root has the given name.
    pub fn new(root_name: impl Into<String>) -> Self {
        Self {
            root: SceneNode::new(root_name),
        }
    }

    /// Total node count (root + all descendants).
    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }

    /// Total mesh count across all nodes.
    pub fn mesh_count(&self) -> usize {
        self.root.mesh_count()
    }
}

// ── GLB export ────────────────────────────────────────────────────────────────

/// Internal: one mesh node collected during the depth-first walk.
#[allow(dead_code)]
struct MeshEntry {
    /// Flat index in the `gltf_nodes` array.
    gltf_node_idx: usize,
    name: String,
    transform: Transform,
    mesh: MeshBuffers,
}

/// Internal: a GLTF node record (may or may not reference a mesh).
struct GltfNodeRecord {
    name: String,
    transform: Transform,
    /// Index into the GLTF meshes array (only for nodes with a mesh).
    mesh_gltf_idx: Option<usize>,
    /// Indices into the gltf_nodes array for children.
    children: Vec<usize>,
}

/// Walk the scene graph depth-first, assigning each SceneNode a GLTF node index,
/// filling `gltf_records` and `mesh_entries`.
fn walk(
    node: &SceneNode,
    gltf_records: &mut Vec<GltfNodeRecord>,
    mesh_entries: &mut Vec<MeshEntry>,
) -> usize {
    let my_idx = gltf_records.len();
    // Reserve slot; we'll fill children after recursion.
    gltf_records.push(GltfNodeRecord {
        name: node.name.clone(),
        transform: node.transform.clone(),
        mesh_gltf_idx: None,
        children: Vec::new(),
    });

    // Recurse into children first (DFS pre-order: index self, then children).
    let mut child_indices = Vec::new();
    for child in &node.children {
        let child_idx = walk(child, gltf_records, mesh_entries);
        child_indices.push(child_idx);
    }
    gltf_records[my_idx].children = child_indices;

    // If this node has a mesh, we'll resolve mesh_gltf_idx after the full walk
    // when we know how many mesh entries there are.  Store the entry index for now.
    if let Some(mesh) = node.mesh.clone() {
        let entry_idx = mesh_entries.len(); // this will be the GLTF mesh index
        mesh_entries.push(MeshEntry {
            gltf_node_idx: my_idx,
            name: node.name.clone(),
            transform: node.transform.clone(),
            mesh,
        });
        gltf_records[my_idx].mesh_gltf_idx = Some(entry_idx);
    }

    my_idx
}

/// Per-mesh BIN layout info.
struct MeshBinLayout {
    pos_offset: usize,
    norm_offset: usize,
    uv_offset: usize,
    idx_offset: usize,
    n_verts: usize,
    n_idx: usize,
    pos_bytes_len: usize,
    norm_bytes_len: usize,
    uv_bytes_len: usize,
    idx_bytes_len: usize,
}

/// Export a scene graph to a GLB 2.0 file.
///
/// Each mesh node becomes a GLTF node with its transform applied.
/// Child nodes become child nodes in the GLTF node hierarchy.
/// Only nodes with meshes produce GLTF mesh entries.
pub fn export_scene_graph_glb(graph: &SceneGraph, path: &Path) -> Result<()> {
    // ── 1. Walk the scene graph ───────────────────────────────────────────────
    let mut gltf_records: Vec<GltfNodeRecord> = Vec::new();
    let mut mesh_entries: Vec<MeshEntry> = Vec::new();

    let root_idx = walk(&graph.root, &mut gltf_records, &mut mesh_entries);

    // ── 2. Build BIN chunk from mesh entries ─────────────────────────────────
    let mut bin_data: Vec<u8> = Vec::new();
    let mut bin_layouts: Vec<MeshBinLayout> = Vec::new();

    for entry in &mesh_entries {
        let mesh = &entry.mesh;
        let pos_bytes: &[u8] = cast_slice(&mesh.positions);
        let norm_bytes: &[u8] = cast_slice(&mesh.normals);
        let uv_bytes: &[u8] = cast_slice(&mesh.uvs);
        let idx_bytes: &[u8] = cast_slice(&mesh.indices);

        let pos_offset = bin_data.len();
        bin_data.extend_from_slice(pos_bytes);
        let norm_offset = bin_data.len();
        bin_data.extend_from_slice(norm_bytes);
        let uv_offset = bin_data.len();
        bin_data.extend_from_slice(uv_bytes);
        let idx_offset = bin_data.len();
        bin_data.extend_from_slice(idx_bytes);

        bin_layouts.push(MeshBinLayout {
            pos_offset,
            norm_offset,
            uv_offset,
            idx_offset,
            n_verts: mesh.positions.len(),
            n_idx: mesh.indices.len(),
            pos_bytes_len: pos_bytes.len(),
            norm_bytes_len: norm_bytes.len(),
            uv_bytes_len: uv_bytes.len(),
            idx_bytes_len: idx_bytes.len(),
        });
    }

    // Pad BIN to 4-byte boundary
    while !bin_data.len().is_multiple_of(4) {
        bin_data.push(0x00);
    }

    // ── 3. Build accessors, bufferViews, meshes JSON ─────────────────────────
    let mut accessors: Vec<serde_json::Value> = Vec::new();
    let mut buffer_views: Vec<serde_json::Value> = Vec::new();
    let mut meshes_json: Vec<serde_json::Value> = Vec::new();

    for (mesh_idx, (entry, layout)) in mesh_entries.iter().zip(bin_layouts.iter()).enumerate() {
        let pos_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.pos_offset,
            "byteLength": layout.pos_bytes_len
        }));

        let norm_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.norm_offset,
            "byteLength": layout.norm_bytes_len
        }));

        let uv_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.uv_offset,
            "byteLength": layout.uv_bytes_len
        }));

        let idx_bv_idx = buffer_views.len();
        buffer_views.push(json!({
            "buffer": 0,
            "byteOffset": layout.idx_offset,
            "byteLength": layout.idx_bytes_len
        }));

        let pos_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": pos_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC3"
        }));

        let norm_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": norm_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC3"
        }));

        let uv_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": uv_bv_idx,
            "componentType": 5126,
            "count": layout.n_verts,
            "type": "VEC2"
        }));

        let idx_acc_idx = accessors.len();
        accessors.push(json!({
            "bufferView": idx_bv_idx,
            "componentType": 5125,
            "count": layout.n_idx,
            "type": "SCALAR"
        }));

        let _ = mesh_idx; // used implicitly via entry reference below

        meshes_json.push(json!({
            "name": entry.name,
            "primitives": [{
                "attributes": {
                    "POSITION":   pos_acc_idx,
                    "NORMAL":     norm_acc_idx,
                    "TEXCOORD_0": uv_acc_idx
                },
                "indices": idx_acc_idx
            }]
        }));
    }

    // ── 4. Build GLTF nodes JSON ──────────────────────────────────────────────
    let mut nodes_json: Vec<serde_json::Value> = Vec::new();

    for record in &gltf_records {
        let m = &record.transform.matrix;
        // GLTF matrix is column-major, same as our storage: 16 floats.
        let matrix_val: Vec<f64> = m.iter().map(|&v| v as f64).collect();

        let node_val = if let Some(mesh_idx) = record.mesh_gltf_idx {
            if record.children.is_empty() {
                json!({
                    "name": record.name,
                    "matrix": matrix_val,
                    "mesh": mesh_idx
                })
            } else {
                json!({
                    "name": record.name,
                    "matrix": matrix_val,
                    "mesh": mesh_idx,
                    "children": record.children
                })
            }
        } else if record.children.is_empty() {
            json!({
                "name": record.name,
                "matrix": matrix_val
            })
        } else {
            json!({
                "name": record.name,
                "matrix": matrix_val,
                "children": record.children
            })
        };

        nodes_json.push(node_val);
    }

    // ── 5. Build top-level GLTF JSON ─────────────────────────────────────────
    let total_bin = bin_data.len() as u32;

    let gltf = json!({
        "asset": { "version": "2.0", "generator": "oxihuman-export/scene_graph" },
        "scene": 0,
        "scenes": [{ "name": graph.root.name, "nodes": [root_idx] }],
        "nodes": nodes_json,
        "meshes": meshes_json,
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{ "byteLength": total_bin }]
    });

    let mut json_bytes = serde_json::to_vec(&gltf)?;
    // Pad JSON to 4-byte boundary with spaces
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }

    // ── 6. Write GLB ─────────────────────────────────────────────────────────
    let json_chunk_len = json_bytes.len() as u32;
    let bin_chunk_len = bin_data.len() as u32;
    let total_len = 12 + 8 + json_chunk_len + 8 + bin_chunk_len;

    let mut file = std::fs::File::create(path)?;

    // GLB header (12 bytes)
    file.write_all(&GLB_MAGIC.to_le_bytes())?;
    file.write_all(&GLB_VERSION.to_le_bytes())?;
    file.write_all(&total_len.to_le_bytes())?;

    // JSON chunk
    file.write_all(&json_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_JSON.to_le_bytes())?;
    file.write_all(&json_bytes)?;

    // BIN chunk
    file.write_all(&bin_chunk_len.to_le_bytes())?;
    file.write_all(&CHUNK_BIN.to_le_bytes())?;
    file.write_all(&bin_data)?;

    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a minimal triangle mesh (3 verts, 1 tri). `has_suit` is set to true
    /// so the export functions do not reject it.
    fn tri_mesh(y_offset: f32) -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, y_offset, 0.0],
                [1.0, y_offset, 0.0],
                [0.0, y_offset + 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            has_suit: true,
        })
    }

    // ── Transform tests ───────────────────────────────────────────────────────

    #[test]
    fn transform_identity_is_identity() {
        let t = Transform::identity();
        assert!(t.is_identity(), "identity matrix must report is_identity()");
    }

    #[test]
    fn transform_translation_correct_matrix() {
        let t = Transform::translation(3.0, 5.0, 7.0);
        // In column-major layout the translation is at indices 12, 13, 14.
        assert_eq!(t.matrix[12], 3.0);
        assert_eq!(t.matrix[13], 5.0);
        assert_eq!(t.matrix[14], 7.0);
        // Diagonal should be [1,1,1,1]
        assert_eq!(t.matrix[0], 1.0);
        assert_eq!(t.matrix[5], 1.0);
        assert_eq!(t.matrix[10], 1.0);
        assert_eq!(t.matrix[15], 1.0);
    }

    #[test]
    fn transform_compose_identity_unchanged() {
        let t = Transform::translation(1.0, 2.0, 3.0);
        let id = Transform::identity();
        let composed = t.compose(&id);
        // composing with identity should give the same matrix
        for (a, b) in composed.matrix.iter().zip(t.matrix.iter()) {
            assert!(
                (a - b).abs() < 1e-6,
                "compose with identity changed the matrix"
            );
        }
    }

    #[test]
    fn transform_scale_compose_translation() {
        // scale(2,2,2) * translation(1,0,0) should scale the translation too.
        let s = Transform::scale(2.0, 2.0, 2.0);
        let tr = Transform::translation(1.0, 0.0, 0.0);
        let composed = s.compose(&tr);
        // column 3 (indices 12-14) should be [2, 0, 0] because scale * [1,0,0,1]
        assert!((composed.matrix[12] - 2.0).abs() < 1e-6);
        assert!((composed.matrix[13]).abs() < 1e-6);
        assert!((composed.matrix[14]).abs() < 1e-6);
    }

    // ── SceneNode tests ───────────────────────────────────────────────────────

    #[test]
    fn scene_node_no_children_count_one() {
        let node = SceneNode::new("root");
        assert_eq!(node.node_count(), 1);
    }

    #[test]
    fn scene_node_with_children_count_correct() {
        let mut root = SceneNode::new("root");
        root.add_child(SceneNode::new("child_a"));
        let mut child_b = SceneNode::new("child_b");
        child_b.add_child(SceneNode::new("grandchild"));
        root.add_child(child_b);
        // root + child_a + child_b + grandchild = 4
        assert_eq!(root.node_count(), 4);
    }

    #[test]
    fn scene_graph_mesh_count_correct() {
        let mut graph = SceneGraph::new("scene");
        graph
            .root
            .add_child(SceneNode::new("body").with_mesh(tri_mesh(0.0)));
        graph
            .root
            .add_child(SceneNode::new("clothing").with_mesh(tri_mesh(1.0)));
        graph.root.add_child(SceneNode::new("empty_node"));
        assert_eq!(graph.mesh_count(), 2);
    }

    #[test]
    fn all_names_depth_first_order() {
        let mut root = SceneNode::new("root");
        let mut child_a = SceneNode::new("child_a");
        child_a.add_child(SceneNode::new("grandchild_a1"));
        child_a.add_child(SceneNode::new("grandchild_a2"));
        root.add_child(child_a);
        root.add_child(SceneNode::new("child_b"));

        let names = root.all_names();
        assert_eq!(
            names,
            vec![
                "root",
                "child_a",
                "grandchild_a1",
                "grandchild_a2",
                "child_b"
            ]
        );
    }

    // ── Export tests ──────────────────────────────────────────────────────────

    #[test]
    fn export_scene_graph_creates_file() {
        let path = std::path::Path::new("/tmp/test_scene_graph_creates.glb");
        let graph = SceneGraph::new("test");
        export_scene_graph_glb(&graph, path).expect("export must succeed");
        assert!(path.exists(), "GLB file must be created");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn export_scene_graph_valid_glb_header() {
        let path = std::path::Path::new("/tmp/test_scene_graph_header.glb");
        let mut graph = SceneGraph::new("header_test");
        graph
            .root
            .add_child(SceneNode::new("body").with_mesh(tri_mesh(0.0)));
        export_scene_graph_glb(&graph, path).expect("export must succeed");

        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() >= 12, "GLB must have at least 12 bytes");
        // Magic "glTF" in LE = [0x67, 0x6C, 0x54, 0x46]
        assert_eq!(
            &bytes[0..4],
            &[0x67u8, 0x6Cu8, 0x54u8, 0x46u8],
            "GLB magic must be glTF"
        );
        // Version = 2 (little-endian u32)
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2, "GLB version must be 2");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn export_empty_mesh_nodes_still_creates_file() {
        let path = std::path::Path::new("/tmp/test_scene_graph_empty_mesh.glb");
        // Graph with nodes but no meshes
        let mut graph = SceneGraph::new("empty_mesh_test");
        graph.root.add_child(SceneNode::new("no_mesh_child"));
        export_scene_graph_glb(&graph, path).expect("export must succeed even without meshes");
        assert!(path.exists(), "GLB file must be created");
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() >= 12);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn export_two_mesh_nodes() {
        let path = std::path::Path::new("/tmp/test_scene_graph_two_meshes.glb");
        let mut graph = SceneGraph::new("two_mesh_scene");
        graph.root.add_child(
            SceneNode::new("body")
                .with_mesh(tri_mesh(0.0))
                .with_transform(Transform::translation(0.0, 0.0, 0.0)),
        );
        graph.root.add_child(
            SceneNode::new("hat")
                .with_mesh(tri_mesh(2.0))
                .with_transform(Transform::translation(0.0, 1.8, 0.0)),
        );
        export_scene_graph_glb(&graph, path).expect("export must succeed");
        assert!(path.exists());
        let bytes = std::fs::read(path).unwrap();
        // Should be bigger than a single-mesh export since there are two meshes.
        assert!(bytes.len() > 12);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn export_nested_hierarchy() {
        let path = std::path::Path::new("/tmp/test_scene_graph_nested.glb");
        let mut graph = SceneGraph::new("nested");
        let mut torso = SceneNode::new("torso").with_mesh(tri_mesh(0.0));
        let head = SceneNode::new("head")
            .with_mesh(tri_mesh(1.5))
            .with_transform(Transform::translation(0.0, 1.5, 0.0));
        torso.add_child(head);
        graph.root.add_child(torso);
        assert_eq!(graph.mesh_count(), 2);
        export_scene_graph_glb(&graph, path).expect("nested export must succeed");
        assert!(path.exists());
        std::fs::remove_file(path).ok();
    }
}
