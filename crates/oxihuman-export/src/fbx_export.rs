//! FBX-compatible export stub.
//!
//! Generates ASCII FBX skeleton with node, mesh, material, and bone sections.
//! This is a structural stub; binary FBX encoding is not implemented.

use std::fmt::Write as FmtWrite;

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for FBX export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxExportConfig {
    /// FBX format version string (e.g. `"7.4.0"`).
    pub version: String,
    /// Scene up-axis: `"Y"` or `"Z"`.
    pub up_axis: String,
    /// Units per centimetre.
    pub unit_scale: f32,
    /// Whether to embed textures.
    pub embed_textures: bool,
}

// ── Node ──────────────────────────────────────────────────────────────────────

/// Kind of FBX node.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FbxNodeKind {
    /// Mesh geometry node.
    Mesh,
    /// Material node.
    Material,
    /// Skeleton bone node.
    Bone,
    /// Generic/null node.
    Null,
}

/// A node in an FBX document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxNode {
    /// Unique numeric identifier.
    pub id: u32,
    /// Node name.
    pub name: String,
    /// Node kind.
    pub kind: FbxNodeKind,
    /// Optional parent id for bone hierarchy.
    pub parent_id: Option<u32>,
    /// Number of vertices (mesh nodes only).
    pub vertex_count: u32,
    /// Number of faces (mesh nodes only).
    pub face_count: u32,
    /// Diffuse colour for material nodes: `[r, g, b]`.
    pub color: [f32; 3],
}

// ── Document ──────────────────────────────────────────────────────────────────

/// In-memory FBX document.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbxDocument {
    /// Export configuration.
    pub config: FbxExportConfig,
    /// Scene name.
    pub scene_name: String,
    /// All nodes in the document.
    pub nodes: Vec<FbxNode>,
    /// Counter for the next node id.
    next_id: u32,
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Returns a sensible default `FbxExportConfig`.
#[allow(dead_code)]
pub fn default_fbx_export_config() -> FbxExportConfig {
    FbxExportConfig {
        version: "7.4.0".to_string(),
        up_axis: "Y".to_string(),
        unit_scale: 1.0,
        embed_textures: false,
    }
}

/// Creates a new, empty `FbxDocument` with the given config.
#[allow(dead_code)]
pub fn new_fbx_document(cfg: &FbxExportConfig) -> FbxDocument {
    FbxDocument {
        config: cfg.clone(),
        scene_name: "Scene".to_string(),
        nodes: Vec::new(),
        next_id: 1,
    }
}

/// Adds a mesh node and returns its id.
#[allow(dead_code)]
pub fn fbx_add_mesh_node(
    doc: &mut FbxDocument,
    name: &str,
    vertex_count: u32,
    face_count: u32,
) -> u32 {
    let id = doc.next_id;
    doc.next_id += 1;
    doc.nodes.push(FbxNode {
        id,
        name: name.to_string(),
        kind: FbxNodeKind::Mesh,
        parent_id: None,
        vertex_count,
        face_count,
        color: [1.0, 1.0, 1.0],
    });
    id
}

/// Adds a material node and returns its id.
#[allow(dead_code)]
pub fn fbx_add_material_node(
    doc: &mut FbxDocument,
    name: &str,
    color: [f32; 3],
) -> u32 {
    let id = doc.next_id;
    doc.next_id += 1;
    doc.nodes.push(FbxNode {
        id,
        name: name.to_string(),
        kind: FbxNodeKind::Material,
        parent_id: None,
        vertex_count: 0,
        face_count: 0,
        color,
    });
    id
}

/// Adds a bone node and returns its id.
#[allow(dead_code)]
pub fn fbx_add_bone_node(
    doc: &mut FbxDocument,
    name: &str,
    parent_id: Option<u32>,
) -> u32 {
    let id = doc.next_id;
    doc.next_id += 1;
    doc.nodes.push(FbxNode {
        id,
        name: name.to_string(),
        kind: FbxNodeKind::Bone,
        parent_id,
        vertex_count: 0,
        face_count: 0,
        color: [0.8, 0.8, 0.8],
    });
    id
}

/// Serialises the document to an ASCII FBX string.
#[allow(dead_code)]
pub fn fbx_to_string(doc: &FbxDocument) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "; FBX {ver} project file\n; Creator: OxiHuman FBX Exporter\n; Scene: {scene}",
        ver = doc.config.version,
        scene = doc.scene_name,
    );
    out.push_str("\nObjects:  {\n");
    for node in &doc.nodes {
        let kind_str = match node.kind {
            FbxNodeKind::Mesh => "Mesh",
            FbxNodeKind::Material => "Material",
            FbxNodeKind::Bone => "LimbNode",
            FbxNodeKind::Null => "Null",
        };
        let _ = writeln!(
            out,
            "    NodeAttribute: {id}, \"{name}\", \"{kind}\" {{",
            id = node.id,
            name = node.name,
            kind = kind_str,
        );
        if node.kind == FbxNodeKind::Mesh {
            let _ = writeln!(out, "        VertexCount: {}", node.vertex_count);
            let _ = writeln!(out, "        FaceCount: {}", node.face_count);
        }
        if node.kind == FbxNodeKind::Material {
            let _ = writeln!(
                out,
                "        DiffuseColor: {:.3},{:.3},{:.3}",
                node.color[0], node.color[1], node.color[2]
            );
        }
        if let Some(pid) = node.parent_id {
            let _ = writeln!(out, "        Parent: {pid}");
        }
        out.push_str("    }\n");
    }
    out.push_str("}\n");
    out
}

/// Writes the FBX document to a file at `path`.
#[allow(dead_code)]
pub fn fbx_write_to_file(doc: &FbxDocument, path: &str) -> Result<(), String> {
    let content = fbx_to_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Returns the number of nodes in the document.
#[allow(dead_code)]
pub fn fbx_node_count(doc: &FbxDocument) -> usize {
    doc.nodes.len()
}

/// Sets the scene name.
#[allow(dead_code)]
pub fn fbx_set_scene_name(doc: &mut FbxDocument, name: &str) {
    doc.scene_name = name.to_string();
}

/// Removes all nodes from the document and resets the id counter.
#[allow(dead_code)]
pub fn fbx_document_clear(doc: &mut FbxDocument) {
    doc.nodes.clear();
    doc.next_id = 1;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_version() {
        let cfg = default_fbx_export_config();
        assert_eq!(cfg.version, "7.4.0");
        assert_eq!(cfg.up_axis, "Y");
    }

    #[test]
    fn test_new_document_empty() {
        let cfg = default_fbx_export_config();
        let doc = new_fbx_document(&cfg);
        assert_eq!(fbx_node_count(&doc), 0);
        assert_eq!(doc.scene_name, "Scene");
    }

    #[test]
    fn test_add_mesh_node() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        let id = fbx_add_mesh_node(&mut doc, "Body", 1000, 500);
        assert_eq!(id, 1);
        assert_eq!(fbx_node_count(&doc), 1);
        assert_eq!(doc.nodes[0].kind, FbxNodeKind::Mesh);
        assert_eq!(doc.nodes[0].vertex_count, 1000);
    }

    #[test]
    fn test_add_material_node() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        let id = fbx_add_material_node(&mut doc, "Skin", [0.9, 0.7, 0.6]);
        assert_eq!(id, 1);
        assert_eq!(doc.nodes[0].kind, FbxNodeKind::Material);
    }

    #[test]
    fn test_add_bone_node_hierarchy() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        let root = fbx_add_bone_node(&mut doc, "Hips", None);
        let child = fbx_add_bone_node(&mut doc, "Spine", Some(root));
        assert_eq!(root, 1);
        assert_eq!(child, 2);
        assert_eq!(doc.nodes[1].parent_id, Some(1));
    }

    #[test]
    fn test_fbx_to_string_contains_objects() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        fbx_add_mesh_node(&mut doc, "Mesh1", 10, 4);
        let s = fbx_to_string(&doc);
        assert!(s.contains("Objects:"));
        assert!(s.contains("Mesh1"));
    }

    #[test]
    fn test_set_scene_name() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        fbx_set_scene_name(&mut doc, "CharacterScene");
        assert_eq!(doc.scene_name, "CharacterScene");
        let s = fbx_to_string(&doc);
        assert!(s.contains("CharacterScene"));
    }

    #[test]
    fn test_document_clear() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        fbx_add_mesh_node(&mut doc, "A", 0, 0);
        fbx_document_clear(&mut doc);
        assert_eq!(fbx_node_count(&doc), 0);
        // Ids restart from 1.
        let id = fbx_add_bone_node(&mut doc, "Root", None);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_node_ids_are_sequential() {
        let cfg = default_fbx_export_config();
        let mut doc = new_fbx_document(&cfg);
        let id1 = fbx_add_mesh_node(&mut doc, "M1", 0, 0);
        let id2 = fbx_add_material_node(&mut doc, "Mat1", [1.0, 1.0, 1.0]);
        let id3 = fbx_add_bone_node(&mut doc, "Bone1", None);
        assert_eq!((id1, id2, id3), (1, 2, 3));
    }
}
