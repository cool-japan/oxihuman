//! FBX format export stub (ASCII FBX 7.4 compatible header + geometry).

#[allow(dead_code)]
pub struct FbxNode {
    pub name: String,
    pub id: u64,
    pub parent_id: Option<u64>,
    pub transform: [[f32; 4]; 4],
}

#[allow(dead_code)]
pub struct FbxMesh {
    pub node_id: u64,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
pub struct FbxScene {
    pub name: String,
    pub nodes: Vec<FbxNode>,
    pub meshes: Vec<FbxMesh>,
    pub up_axis: u8,
    pub units: f32,
}

#[allow(dead_code)]
pub struct FbxExport {
    pub content: String,
    pub version: u32,
}

#[allow(dead_code)]
pub fn new_fbx_scene(name: &str) -> FbxScene {
    FbxScene {
        name: name.to_string(),
        nodes: Vec::new(),
        meshes: Vec::new(),
        up_axis: 1,
        units: 1.0,
    }
}

#[allow(dead_code)]
pub fn add_fbx_node(scene: &mut FbxScene, name: &str, parent: Option<u64>) -> u64 {
    let id = scene.nodes.len() as u64 + 1;
    scene.nodes.push(FbxNode {
        name: name.to_string(),
        id,
        parent_id: parent,
        transform: fbx_identity_matrix(),
    });
    id
}

#[allow(dead_code)]
pub fn add_fbx_mesh(scene: &mut FbxScene, mesh: FbxMesh) {
    scene.meshes.push(mesh);
}

#[allow(dead_code)]
pub fn fbx_identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[allow(dead_code)]
pub fn fbx_header(version: u32) -> String {
    let major = version / 1000;
    let minor = (version % 1000) / 100;
    let patch = version % 100;
    format!(
        "; FBX {major}.{minor}.{patch} project file\n\
         ; Copyright (C) 1997-2023 Autodesk Inc. and/or its licensors.\n\
         ; All Rights Reserved.\n\
         \n\
         FBXHeaderExtension: {{\n\
         \tFBXHeaderVersion: 1003\n\
         \tFBXVersion: {version}\n\
         }}\n"
    )
}

#[allow(dead_code)]
pub fn fbx_node_to_string(node: &FbxNode) -> String {
    let parent_str = match node.parent_id {
        Some(pid) => format!("\tParentId: {pid}\n"),
        None => String::new(),
    };
    let t = &node.transform;
    format!(
        "Model: {id}, \"{name}\", \"Mesh\" {{\n\
         {parent_str}\
         \tTransform: {r0:?}, {r1:?}, {r2:?}, {r3:?}\n\
         }}\n",
        id = node.id,
        name = node.name,
        r0 = t[0],
        r1 = t[1],
        r2 = t[2],
        r3 = t[3],
    )
}

#[allow(dead_code)]
pub fn fbx_mesh_to_string(mesh: &FbxMesh) -> String {
    let vert_count = mesh.positions.len();
    let mut s = format!(
        "Geometry: {id}, \"Geometry::\", \"Mesh\" {{\n\
         \tVertices: *{vert_count} {{\n",
        id = mesh.node_id,
    );
    s.push_str("\t\ta: ");
    let coords: Vec<String> = mesh
        .positions
        .iter()
        .map(|p| format!("{},{},{}", p[0], p[1], p[2]))
        .collect();
    s.push_str(&coords.join(","));
    s.push('\n');
    s.push_str("\t}\n");

    let idx_count = mesh.indices.len();
    s.push_str(&format!("\tPolygonVertexIndex: *{idx_count} {{\n\t\ta: "));
    let idx_strs: Vec<String> = mesh
        .indices
        .chunks(3)
        .flat_map(|tri| {
            if tri.len() == 3 {
                vec![
                    tri[0].to_string(),
                    tri[1].to_string(),
                    format!("-{}", tri[2] + 1),
                ]
            } else {
                tri.iter().map(|v| v.to_string()).collect()
            }
        })
        .collect();
    s.push_str(&idx_strs.join(","));
    s.push_str("\n\t}\n}\n");
    s
}

#[allow(dead_code)]
pub fn fbx_connections(scene: &FbxScene) -> String {
    let mut s = "Connections: {\n".to_string();
    for node in &scene.nodes {
        let parent = node.parent_id.unwrap_or(0);
        s.push_str(&format!("\tC: \"OO\", {}, {}\n", node.id, parent));
    }
    for mesh in &scene.meshes {
        s.push_str(&format!(
            "\tC: \"OO\", Geo_{}, {}\n",
            mesh.node_id, mesh.node_id
        ));
    }
    s.push_str("}\n");
    s
}

#[allow(dead_code)]
pub fn export_fbx_ascii(scene: &FbxScene) -> FbxExport {
    let version = 7400u32;
    let mut content = fbx_header(version);

    content.push_str("\nObjects: {\n");
    for node in &scene.nodes {
        content.push_str(&fbx_node_to_string(node));
    }
    for mesh in &scene.meshes {
        content.push_str(&fbx_mesh_to_string(mesh));
    }
    content.push_str("}\n\n");
    content.push_str(&fbx_connections(scene));

    FbxExport { content, version }
}

#[allow(dead_code)]
pub fn node_count_fbx(scene: &FbxScene) -> usize {
    scene.nodes.len()
}

#[allow(dead_code)]
pub fn mesh_count_fbx(scene: &FbxScene) -> usize {
    scene.meshes.len()
}

#[allow(dead_code)]
pub fn validate_fbx_scene(scene: &FbxScene) -> Vec<String> {
    let mut issues = Vec::new();
    if scene.name.is_empty() {
        issues.push("Scene name is empty".to_string());
    }
    let node_ids: Vec<u64> = scene.nodes.iter().map(|n| n.id).collect();
    for node in &scene.nodes {
        if let Some(pid) = node.parent_id {
            if !node_ids.contains(&pid) {
                issues.push(format!(
                    "Node '{}' references non-existent parent id {pid}",
                    node.name
                ));
            }
        }
    }
    for mesh in &scene.meshes {
        if mesh.positions.is_empty() {
            issues.push(format!("Mesh for node {} has no positions", mesh.node_id));
        }
        if mesh.indices.len() % 3 != 0 {
            issues.push(format!(
                "Mesh for node {} has non-triangulated index count {}",
                mesh.node_id,
                mesh.indices.len()
            ));
        }
    }
    if scene.units <= 0.0 {
        issues.push("Scene units must be positive".to_string());
    }
    issues
}

#[allow(dead_code)]
pub fn fbx_export_size_estimate(export: &FbxExport) -> usize {
    export.content.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fbx_scene() {
        let scene = new_fbx_scene("TestScene");
        assert_eq!(scene.name, "TestScene");
        assert!(scene.nodes.is_empty());
        assert!(scene.meshes.is_empty());
        assert_eq!(scene.up_axis, 1);
        assert!((scene.units - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_fbx_node_no_parent() {
        let mut scene = new_fbx_scene("S");
        let id = add_fbx_node(&mut scene, "Root", None);
        assert_eq!(id, 1);
        assert_eq!(scene.nodes.len(), 1);
        assert_eq!(scene.nodes[0].name, "Root");
        assert!(scene.nodes[0].parent_id.is_none());
    }

    #[test]
    fn test_add_fbx_node_with_parent() {
        let mut scene = new_fbx_scene("S");
        let root = add_fbx_node(&mut scene, "Root", None);
        let child = add_fbx_node(&mut scene, "Child", Some(root));
        assert_eq!(child, 2);
        assert_eq!(scene.nodes[1].parent_id, Some(root));
    }

    #[test]
    fn test_add_fbx_mesh() {
        let mut scene = new_fbx_scene("S");
        let mesh = FbxMesh {
            node_id: 1,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
        };
        add_fbx_mesh(&mut scene, mesh);
        assert_eq!(scene.meshes.len(), 1);
    }

    #[test]
    fn test_fbx_header_contains_fbx() {
        let header = fbx_header(7400);
        assert!(header.contains("FBX"));
        assert!(header.contains("7400"));
    }

    #[test]
    fn test_fbx_identity_matrix() {
        let m = fbx_identity_matrix();
        for (i, row) in m.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                let expected = if i == j { 1.0f32 } else { 0.0f32 };
                assert!((val - expected).abs() < 1e-6, "m[{i}][{j}] = {val}");
            }
        }
    }

    #[test]
    fn test_export_fbx_ascii_non_empty() {
        let mut scene = new_fbx_scene("Test");
        add_fbx_node(&mut scene, "Root", None);
        let export = export_fbx_ascii(&scene);
        assert!(!export.content.is_empty());
        assert_eq!(export.version, 7400);
    }

    #[test]
    fn test_node_count_fbx() {
        let mut scene = new_fbx_scene("S");
        assert_eq!(node_count_fbx(&scene), 0);
        add_fbx_node(&mut scene, "A", None);
        add_fbx_node(&mut scene, "B", None);
        assert_eq!(node_count_fbx(&scene), 2);
    }

    #[test]
    fn test_mesh_count_fbx() {
        let mut scene = new_fbx_scene("S");
        assert_eq!(mesh_count_fbx(&scene), 0);
        let mesh = FbxMesh {
            node_id: 1,
            positions: vec![[0.0; 3]],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
        };
        add_fbx_mesh(&mut scene, mesh);
        assert_eq!(mesh_count_fbx(&scene), 1);
    }

    #[test]
    fn test_validate_fbx_scene_passes() {
        let mut scene = new_fbx_scene("Valid");
        add_fbx_node(&mut scene, "Root", None);
        let issues = validate_fbx_scene(&scene);
        assert!(issues.is_empty(), "Expected no issues, got: {issues:?}");
    }

    #[test]
    fn test_validate_fbx_scene_empty_name() {
        let scene = new_fbx_scene("");
        let issues = validate_fbx_scene(&scene);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_fbx_mesh_to_string_contains_vertex_count() {
        let mesh = FbxMesh {
            node_id: 42,
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![],
            uvs: vec![],
            indices: vec![0, 1, 2],
        };
        let s = fbx_mesh_to_string(&mesh);
        assert!(s.contains("*3"), "Expected vertex count 3 in: {s}");
    }

    #[test]
    fn test_fbx_connections_non_empty() {
        let mut scene = new_fbx_scene("S");
        add_fbx_node(&mut scene, "Root", None);
        let conn = fbx_connections(&scene);
        assert!(conn.contains("Connections:"));
        assert!(conn.contains("OO"));
    }

    #[test]
    fn test_fbx_export_size_estimate() {
        let mut scene = new_fbx_scene("S");
        add_fbx_node(&mut scene, "Root", None);
        let export = export_fbx_ascii(&scene);
        let size = fbx_export_size_estimate(&export);
        assert_eq!(size, export.content.len());
        assert!(size > 0);
    }

    #[test]
    fn test_fbx_node_to_string() {
        let node = FbxNode {
            name: "TestNode".to_string(),
            id: 100,
            parent_id: None,
            transform: fbx_identity_matrix(),
        };
        let s = fbx_node_to_string(&node);
        assert!(s.contains("TestNode"));
        assert!(s.contains("100"));
    }

    #[test]
    fn test_validate_fbx_bad_parent() {
        let mut scene = new_fbx_scene("S");
        scene.nodes.push(FbxNode {
            name: "Orphan".to_string(),
            id: 1,
            parent_id: Some(999),
            transform: fbx_identity_matrix(),
        });
        let issues = validate_fbx_scene(&scene);
        assert!(!issues.is_empty());
    }
}
