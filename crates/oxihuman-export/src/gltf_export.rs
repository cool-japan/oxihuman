// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! glTF 2.0 JSON export — meshes, materials, and scene nodes.
//!
//! Produces a minimal, spec-compliant glTF 2.0 JSON string without external
//! library dependencies.

#![allow(dead_code)]

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for glTF 2.0 export.
#[derive(Debug, Clone)]
pub struct GltfExportConfig {
    /// Generator string placed in the asset section. Default: `"OxiHuman"`.
    pub generator: String,
    /// glTF spec version. Default: `"2.0"`.
    pub version: String,
    /// Whether to pretty-print the JSON output. Default: `false`.
    pub pretty: bool,
}

impl Default for GltfExportConfig {
    fn default() -> Self {
        Self {
            generator: "OxiHuman".to_string(),
            version: "2.0".to_string(),
            pretty: false,
        }
    }
}

/// A glTF scene node with an optional mesh index and transform.
#[derive(Debug, Clone)]
pub struct GltfNode {
    /// Node name.
    pub name: String,
    /// Index into the mesh array, if any.
    pub mesh: Option<usize>,
    /// Translation `[x, y, z]`. Default: `[0, 0, 0]`.
    pub translation: [f32; 3],
    /// Rotation quaternion `[x, y, z, w]`. Default: `[0, 0, 0, 1]`.
    pub rotation: [f32; 4],
    /// Scale `[x, y, z]`. Default: `[1, 1, 1]`.
    pub scale: [f32; 3],
}

/// A glTF mesh with a name and a list of primitive attribute names.
#[derive(Debug, Clone)]
pub struct GltfMesh {
    /// Mesh name.
    pub name: String,
    /// Primitive attribute names (e.g., `"POSITION"`, `"NORMAL"`).
    pub attributes: Vec<String>,
}

/// A glTF PBR metallic-roughness material.
#[derive(Debug, Clone)]
pub struct GltfMaterial {
    /// Material name.
    pub name: String,
    /// Base color factor `[r, g, b, a]` in linear space.
    pub base_color_factor: [f32; 4],
    /// Metallic factor `[0..=1]`.
    pub metallic_factor: f32,
    /// Roughness factor `[0..=1]`.
    pub roughness_factor: f32,
    /// Whether the material is double-sided.
    pub double_sided: bool,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Result type for glTF validation.
pub type GltfValidationResult = Result<(), String>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`GltfExportConfig`].
#[allow(dead_code)]
pub fn default_gltf_config() -> GltfExportConfig {
    GltfExportConfig::default()
}

/// Construct a new [`GltfNode`] with the given name.
#[allow(dead_code)]
pub fn new_gltf_node(name: &str) -> GltfNode {
    GltfNode {
        name: name.to_string(),
        mesh: None,
        translation: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}

/// Append `node` to `nodes` and return the new node count.
#[allow(dead_code)]
pub fn add_gltf_node(nodes: &mut Vec<GltfNode>, node: GltfNode) -> usize {
    nodes.push(node);
    nodes.len()
}

/// Append `mesh` to `meshes` and return the new mesh count.
#[allow(dead_code)]
pub fn add_gltf_mesh(meshes: &mut Vec<GltfMesh>, mesh: GltfMesh) -> usize {
    meshes.push(mesh);
    meshes.len()
}

/// Append `material` to `materials` and return the new material count.
#[allow(dead_code)]
pub fn add_gltf_material(materials: &mut Vec<GltfMaterial>, material: GltfMaterial) -> usize {
    materials.push(material);
    materials.len()
}

/// Return the number of nodes.
#[allow(dead_code)]
pub fn node_count(nodes: &[GltfNode]) -> usize {
    nodes.len()
}

/// Return the number of meshes.
#[allow(dead_code)]
pub fn mesh_count(meshes: &[GltfMesh]) -> usize {
    meshes.len()
}

/// Return the number of materials.
#[allow(dead_code)]
pub fn material_count_gltf(materials: &[GltfMaterial]) -> usize {
    materials.len()
}

/// Return a default white PBR [`GltfMaterial`].
#[allow(dead_code)]
pub fn default_gltf_material(name: &str) -> GltfMaterial {
    GltfMaterial {
        name: name.to_string(),
        base_color_factor: [1.0, 1.0, 1.0, 1.0],
        metallic_factor: 0.0,
        roughness_factor: 0.5,
        double_sided: false,
    }
}

/// Validate nodes, meshes, and materials.
///
/// Returns `Err` if any node references a mesh index that is out of range.
#[allow(dead_code)]
pub fn validate_gltf(
    nodes: &[GltfNode],
    meshes: &[GltfMesh],
    _materials: &[GltfMaterial],
) -> GltfValidationResult {
    for node in nodes {
        if let Some(mi) = node.mesh {
            if mi >= meshes.len() {
                return Err(format!(
                    "node '{}' references mesh index {} but only {} meshes exist",
                    node.name,
                    mi,
                    meshes.len()
                ));
            }
        }
    }
    Ok(())
}

/// Estimate the JSON output size in bytes (rough heuristic).
#[allow(dead_code)]
pub fn gltf_file_size_estimate(
    nodes: &[GltfNode],
    meshes: &[GltfMesh],
    materials: &[GltfMaterial],
) -> usize {
    // Base overhead for asset section, scenes, etc.
    let base = 200usize;
    let node_bytes: usize = nodes.iter().map(|n| 80 + n.name.len()).sum();
    let mesh_bytes: usize = meshes.iter().map(|m| 60 + m.name.len() + m.attributes.len() * 15).sum();
    let mat_bytes: usize = materials.iter().map(|m| 100 + m.name.len()).sum();
    base + node_bytes + mesh_bytes + mat_bytes
}

/// Serialize everything into a minimal glTF 2.0 JSON string.
#[allow(dead_code)]
pub fn gltf_to_json(
    nodes: &[GltfNode],
    meshes: &[GltfMesh],
    materials: &[GltfMaterial],
    cfg: &GltfExportConfig,
) -> String {
    let sep = if cfg.pretty { "\n  " } else { "" };
    let nl = if cfg.pretty { "\n" } else { "" };

    // asset
    let asset = format!(
        r#"{{"generator":"{}","version":"{}"}}"#,
        cfg.generator, cfg.version
    );

    // nodes
    let nodes_json: Vec<String> = nodes
        .iter()
        .map(|n| {
            let mesh_part = match n.mesh {
                Some(i) => format!(r#","mesh":{}"#, i),
                None => String::new(),
            };
            format!(
                r#"{{"name":"{}","translation":[{},{},{}],"rotation":[{},{},{},{}],"scale":[{},{},{}]{}}}"#,
                n.name,
                n.translation[0], n.translation[1], n.translation[2],
                n.rotation[0], n.rotation[1], n.rotation[2], n.rotation[3],
                n.scale[0], n.scale[1], n.scale[2],
                mesh_part
            )
        })
        .collect();

    // meshes
    let meshes_json: Vec<String> = meshes
        .iter()
        .map(|m| {
            let attrs: Vec<String> = m
                .attributes
                .iter()
                .enumerate()
                .map(|(i, a)| format!(r#""{}":{}"#, a, i))
                .collect();
            format!(
                r#"{{"name":"{}","primitives":[{{"attributes":{{{}}}}}]}}"#,
                m.name,
                attrs.join(",")
            )
        })
        .collect();

    // materials
    let mats_json: Vec<String> = materials
        .iter()
        .map(|mat| {
            let cf = mat.base_color_factor;
            format!(
                r#"{{"name":"{}","pbrMetallicRoughness":{{"baseColorFactor":[{},{},{},{}],"metallicFactor":{},"roughnessFactor":{}}},"doubleSided":{}}}"#,
                mat.name,
                cf[0], cf[1], cf[2], cf[3],
                mat.metallic_factor,
                mat.roughness_factor,
                mat.double_sided
            )
        })
        .collect();

    // scene node indices
    let node_indices: Vec<String> = (0..nodes.len()).map(|i| i.to_string()).collect();

    format!(
        r#"{{{sep}"asset":{asset},{sep}"scene":0,{sep}"scenes":[{{"nodes":[{nodes_idx}]}}],{sep}"nodes":[{nodes_arr}],{sep}"meshes":[{meshes_arr}],{sep}"materials":[{mats_arr}]{nl}}}"#,
        sep = sep,
        nl = nl,
        asset = asset,
        nodes_idx = node_indices.join(","),
        nodes_arr = nodes_json.join(","),
        meshes_arr = meshes_json.join(","),
        mats_arr = mats_json.join(","),
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node() -> GltfNode {
        let mut n = new_gltf_node("RootNode");
        n.mesh = Some(0);
        n
    }

    fn sample_mesh() -> GltfMesh {
        GltfMesh {
            name: "Body".to_string(),
            attributes: vec!["POSITION".to_string(), "NORMAL".to_string()],
        }
    }

    fn sample_material() -> GltfMaterial {
        default_gltf_material("Skin")
    }

    #[test]
    fn test_default_gltf_config() {
        let cfg = default_gltf_config();
        assert_eq!(cfg.version, "2.0");
        assert_eq!(cfg.generator, "OxiHuman");
        assert!(!cfg.pretty);
    }

    #[test]
    fn test_new_gltf_node_defaults() {
        let n = new_gltf_node("Pelvis");
        assert_eq!(n.name, "Pelvis");
        assert_eq!(n.mesh, None);
        assert_eq!(n.scale, [1.0, 1.0, 1.0]);
        assert_eq!(n.rotation[3], 1.0);
    }

    #[test]
    fn test_add_gltf_node_count() {
        let mut nodes: Vec<GltfNode> = Vec::new();
        assert_eq!(add_gltf_node(&mut nodes, sample_node()), 1);
        assert_eq!(add_gltf_node(&mut nodes, new_gltf_node("Head")), 2);
    }

    #[test]
    fn test_add_gltf_mesh_count() {
        let mut meshes: Vec<GltfMesh> = Vec::new();
        assert_eq!(add_gltf_mesh(&mut meshes, sample_mesh()), 1);
    }

    #[test]
    fn test_add_gltf_material_count() {
        let mut mats: Vec<GltfMaterial> = Vec::new();
        assert_eq!(add_gltf_material(&mut mats, sample_material()), 1);
    }

    #[test]
    fn test_node_count() {
        let nodes = vec![sample_node(), new_gltf_node("B")];
        assert_eq!(node_count(&nodes), 2);
    }

    #[test]
    fn test_mesh_count() {
        let meshes = vec![sample_mesh()];
        assert_eq!(mesh_count(&meshes), 1);
    }

    #[test]
    fn test_material_count_gltf() {
        let mats = vec![sample_material(), default_gltf_material("Hair")];
        assert_eq!(material_count_gltf(&mats), 2);
    }

    #[test]
    fn test_default_gltf_material_fields() {
        let mat = default_gltf_material("Eyes");
        assert_eq!(mat.name, "Eyes");
        assert_eq!(mat.base_color_factor, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic_factor, 0.0);
        assert!(!mat.double_sided);
    }

    #[test]
    fn test_validate_gltf_ok() {
        let nodes = vec![sample_node()];
        let meshes = vec![sample_mesh()];
        let mats = vec![sample_material()];
        assert!(validate_gltf(&nodes, &meshes, &mats).is_ok());
    }

    #[test]
    fn test_validate_gltf_bad_mesh_ref() {
        let mut node = new_gltf_node("Bad");
        node.mesh = Some(99);
        let meshes: Vec<GltfMesh> = Vec::new();
        let mats: Vec<GltfMaterial> = Vec::new();
        assert!(validate_gltf(&[node], &meshes, &mats).is_err());
    }

    #[test]
    fn test_gltf_file_size_estimate_positive() {
        let nodes = vec![sample_node()];
        let meshes = vec![sample_mesh()];
        let mats = vec![sample_material()];
        assert!(gltf_file_size_estimate(&nodes, &meshes, &mats) > 0);
    }

    #[test]
    fn test_gltf_to_json_contains_asset() {
        let cfg = default_gltf_config();
        let json = gltf_to_json(&[sample_node()], &[sample_mesh()], &[sample_material()], &cfg);
        assert!(json.contains("\"asset\""));
        assert!(json.contains("\"2.0\""));
        assert!(json.contains("OxiHuman"));
    }

    #[test]
    fn test_gltf_to_json_contains_node_name() {
        let cfg = default_gltf_config();
        let json = gltf_to_json(&[sample_node()], &[sample_mesh()], &[sample_material()], &cfg);
        assert!(json.contains("RootNode"));
    }

    #[test]
    fn test_gltf_to_json_contains_mesh_name() {
        let cfg = default_gltf_config();
        let json = gltf_to_json(&[sample_node()], &[sample_mesh()], &[sample_material()], &cfg);
        assert!(json.contains("Body"));
    }

    #[test]
    fn test_gltf_to_json_contains_material_name() {
        let cfg = default_gltf_config();
        let json = gltf_to_json(&[sample_node()], &[sample_mesh()], &[sample_material()], &cfg);
        assert!(json.contains("Skin"));
    }

    #[test]
    fn test_gltf_to_json_empty() {
        let cfg = default_gltf_config();
        let json = gltf_to_json(&[], &[], &[], &cfg);
        assert!(json.contains("\"asset\""));
        assert!(json.contains("\"scenes\""));
    }
}
