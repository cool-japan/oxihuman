//! USDZ format export (ZIP archive containing USD ASCII files).

#[allow(dead_code)]
pub struct UsdMesh {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub material_name: Option<String>,
}

#[allow(dead_code)]
pub struct UsdMaterial {
    pub name: String,
    pub diffuse_color: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub texture_path: Option<String>,
}

#[allow(dead_code)]
pub struct UsdScene {
    pub name: String,
    pub meshes: Vec<UsdMesh>,
    pub materials: Vec<UsdMaterial>,
    pub up_axis: String,
    pub meters_per_unit: f32,
}

#[allow(dead_code)]
pub struct UsdzPackage {
    pub usda_content: String,
    pub name: String,
    pub file_count: usize,
}

#[allow(dead_code)]
pub fn new_usd_scene(name: &str) -> UsdScene {
    UsdScene {
        name: name.to_string(),
        meshes: Vec::new(),
        materials: Vec::new(),
        up_axis: "Y".to_string(),
        meters_per_unit: 1.0,
    }
}

#[allow(dead_code)]
pub fn add_usd_mesh(scene: &mut UsdScene, mesh: UsdMesh) {
    scene.meshes.push(mesh);
}

#[allow(dead_code)]
pub fn add_usd_material(scene: &mut UsdScene, mat: UsdMaterial) {
    scene.materials.push(mat);
}

#[allow(dead_code)]
pub fn default_usd_material(name: &str) -> UsdMaterial {
    UsdMaterial {
        name: name.to_string(),
        diffuse_color: [0.8, 0.8, 0.8],
        roughness: 0.5,
        metallic: 0.0,
        texture_path: None,
    }
}

#[allow(dead_code)]
pub fn mesh_to_usda(mesh: &UsdMesh) -> String {
    let mut s = String::new();
    s.push_str(&format!("def Mesh \"{}\" {{\n", mesh.name));

    // positions
    let pts: Vec<String> = mesh
        .positions
        .iter()
        .map(|p| format!("({}, {}, {})", p[0], p[1], p[2]))
        .collect();
    s.push_str(&format!("    point3f[] points = [{}]\n", pts.join(", ")));

    // normals
    if !mesh.normals.is_empty() {
        let nrms: Vec<String> = mesh
            .normals
            .iter()
            .map(|n| format!("({}, {}, {})", n[0], n[1], n[2]))
            .collect();
        s.push_str(&format!("    normal3f[] normals = [{}]\n", nrms.join(", ")));
    }

    // uvs
    if !mesh.uvs.is_empty() {
        let uvs: Vec<String> = mesh
            .uvs
            .iter()
            .map(|uv| format!("({}, {})", uv[0], uv[1]))
            .collect();
        s.push_str(&format!(
            "    texCoord2f[] primvars:st = [{}]\n",
            uvs.join(", ")
        ));
    }

    // indices
    let idx_str: Vec<String> = mesh.indices.iter().map(|i| i.to_string()).collect();
    s.push_str(&format!(
        "    int[] faceVertexIndices = [{}]\n",
        idx_str.join(", ")
    ));

    // face vertex counts (assume triangles)
    let face_count = mesh.indices.len() / 3;
    let counts: Vec<String> = (0..face_count).map(|_| "3".to_string()).collect();
    s.push_str(&format!(
        "    int[] faceVertexCounts = [{}]\n",
        counts.join(", ")
    ));

    if let Some(ref mat) = mesh.material_name {
        s.push_str(&format!(
            "    rel material:binding = </Materials/{}>\n",
            mat
        ));
    }

    s.push_str("}\n");
    s
}

#[allow(dead_code)]
pub fn material_to_usda(mat: &UsdMaterial) -> String {
    let mut s = String::new();
    s.push_str(&format!("def Material \"{}\" {{\n", mat.name));
    s.push_str("    def Shader \"PBRShader\" {\n");
    s.push_str("        uniform token info:id = \"UsdPreviewSurface\"\n");
    s.push_str(&format!(
        "        color3f inputs:diffuseColor = ({}, {}, {})\n",
        mat.diffuse_color[0], mat.diffuse_color[1], mat.diffuse_color[2]
    ));
    s.push_str(&format!(
        "        float inputs:roughness = {}\n",
        mat.roughness
    ));
    s.push_str(&format!(
        "        float inputs:metallic = {}\n",
        mat.metallic
    ));
    if let Some(ref tex) = mat.texture_path {
        s.push_str(&format!("        asset inputs:file = @{}@\n", tex));
    }
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

#[allow(dead_code)]
pub fn scene_to_usda(scene: &UsdScene) -> String {
    let mut s = String::new();
    s.push_str("#usda 1.0\n");
    s.push_str("(\n");
    s.push_str(&format!("    defaultPrim = \"{}\"\n", scene.name));
    s.push_str(&format!("    upAxis = \"{}\"\n", scene.up_axis));
    s.push_str(&format!("    metersPerUnit = {}\n", scene.meters_per_unit));
    s.push_str(")\n\n");

    s.push_str(&format!("def Xform \"{}\" {{\n", scene.name));
    for mesh in &scene.meshes {
        for line in mesh_to_usda(mesh).lines() {
            s.push_str("    ");
            s.push_str(line);
            s.push('\n');
        }
    }
    s.push_str("}\n\n");

    if !scene.materials.is_empty() {
        s.push_str("def Scope \"Materials\" {\n");
        for mat in &scene.materials {
            for line in material_to_usda(mat).lines() {
                s.push_str("    ");
                s.push_str(line);
                s.push('\n');
            }
        }
        s.push_str("}\n");
    }

    s
}

#[allow(dead_code)]
pub fn package_usdz(scene: &UsdScene) -> UsdzPackage {
    let usda = scene_to_usda(scene);
    let file_count = 1 + scene
        .materials
        .iter()
        .filter(|m| m.texture_path.is_some())
        .count();
    UsdzPackage {
        usda_content: usda,
        name: format!("{}.usdz", scene.name),
        file_count,
    }
}

#[allow(dead_code)]
pub fn usdz_file_size_estimate(pkg: &UsdzPackage) -> usize {
    // Local file header overhead per file: ~30 bytes + filename, plus content
    let header_overhead = 30 + pkg.name.len();
    pkg.usda_content.len() + header_overhead * pkg.file_count + 22 // end-of-central-dir record
}

#[allow(dead_code)]
pub fn scene_mesh_count(scene: &UsdScene) -> usize {
    scene.meshes.len()
}

#[allow(dead_code)]
pub fn validate_usd_scene(scene: &UsdScene) -> Vec<String> {
    let mut errors = Vec::new();
    if scene.name.is_empty() {
        errors.push("Scene name is empty".to_string());
    }
    if scene.up_axis != "Y" && scene.up_axis != "Z" {
        errors.push(format!(
            "Invalid up_axis: '{}', must be 'Y' or 'Z'",
            scene.up_axis
        ));
    }
    if scene.meters_per_unit <= 0.0 {
        errors.push("meters_per_unit must be positive".to_string());
    }
    for (i, mesh) in scene.meshes.iter().enumerate() {
        if mesh.name.is_empty() {
            errors.push(format!("Mesh {} has empty name", i));
        }
        if mesh.positions.is_empty() {
            errors.push(format!("Mesh '{}' has no positions", mesh.name));
        }
        if mesh.indices.len() % 3 != 0 {
            errors.push(format!(
                "Mesh '{}' index count {} is not divisible by 3",
                mesh.name,
                mesh.indices.len()
            ));
        }
    }
    for (i, mat) in scene.materials.iter().enumerate() {
        if mat.name.is_empty() {
            errors.push(format!("Material {} has empty name", i));
        }
        if mat.roughness < 0.0 || mat.roughness > 1.0 {
            errors.push(format!(
                "Material '{}' roughness {} out of [0,1]",
                mat.name, mat.roughness
            ));
        }
        if mat.metallic < 0.0 || mat.metallic > 1.0 {
            errors.push(format!(
                "Material '{}' metallic {} out of [0,1]",
                mat.name, mat.metallic
            ));
        }
    }
    errors
}

#[allow(dead_code)]
pub fn usdz_magic_bytes() -> Vec<u8> {
    vec![0x50, 0x4B, 0x03, 0x04]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> UsdMesh {
        UsdMesh {
            name: "TestMesh".to_string(),
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            material_name: Some("Mat0".to_string()),
        }
    }

    #[test]
    fn test_new_usd_scene() {
        let scene = new_usd_scene("MyScene");
        assert_eq!(scene.name, "MyScene");
        assert!(scene.meshes.is_empty());
        assert!(scene.materials.is_empty());
        assert_eq!(scene.up_axis, "Y");
        assert!((scene.meters_per_unit - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_usd_mesh() {
        let mut scene = new_usd_scene("S");
        add_usd_mesh(&mut scene, sample_mesh());
        assert_eq!(scene_mesh_count(&scene), 1);
    }

    #[test]
    fn test_add_usd_material() {
        let mut scene = new_usd_scene("S");
        let mat = default_usd_material("Mat0");
        add_usd_material(&mut scene, mat);
        assert_eq!(scene.materials.len(), 1);
        assert_eq!(scene.materials[0].name, "Mat0");
    }

    #[test]
    fn test_default_usd_material() {
        let mat = default_usd_material("Default");
        assert_eq!(mat.name, "Default");
        assert!((mat.roughness - 0.5).abs() < 1e-6);
        assert!((mat.metallic).abs() < 1e-6);
        assert!(mat.texture_path.is_none());
    }

    #[test]
    fn test_mesh_to_usda_contains_name() {
        let mesh = sample_mesh();
        let usda = mesh_to_usda(&mesh);
        assert!(usda.contains("TestMesh"));
        assert!(usda.contains("points"));
    }

    #[test]
    fn test_mesh_to_usda_contains_indices() {
        let mesh = sample_mesh();
        let usda = mesh_to_usda(&mesh);
        assert!(usda.contains("faceVertexIndices"));
    }

    #[test]
    fn test_material_to_usda_contains_name() {
        let mat = default_usd_material("Mat0");
        let usda = material_to_usda(&mat);
        assert!(usda.contains("Mat0"));
        assert!(usda.contains("UsdPreviewSurface"));
    }

    #[test]
    fn test_scene_to_usda_header() {
        let mut scene = new_usd_scene("Scene1");
        add_usd_mesh(&mut scene, sample_mesh());
        let usda = scene_to_usda(&scene);
        assert!(usda.contains("#usda 1.0"));
        assert!(usda.contains("Scene1"));
    }

    #[test]
    fn test_package_usdz_non_empty() {
        let mut scene = new_usd_scene("Pkg");
        add_usd_mesh(&mut scene, sample_mesh());
        let pkg = package_usdz(&scene);
        assert!(!pkg.usda_content.is_empty());
        assert!(pkg.file_count >= 1);
    }

    #[test]
    fn test_usdz_magic_bytes_len() {
        let magic = usdz_magic_bytes();
        assert_eq!(magic.len(), 4);
        assert_eq!(magic[0], 0x50);
        assert_eq!(magic[1], 0x4B);
    }

    #[test]
    fn test_validate_valid_scene() {
        let mut scene = new_usd_scene("Valid");
        add_usd_mesh(&mut scene, sample_mesh());
        let errors = validate_usd_scene(&scene);
        assert!(errors.is_empty(), "Errors: {:?}", errors);
    }

    #[test]
    fn test_validate_empty_scene_name() {
        let scene = new_usd_scene("");
        let errors = validate_usd_scene(&scene);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_validate_bad_up_axis() {
        let mut scene = new_usd_scene("S");
        scene.up_axis = "X".to_string();
        let errors = validate_usd_scene(&scene);
        assert!(errors.iter().any(|e| e.contains("up_axis")));
    }

    #[test]
    fn test_usdz_file_size_estimate_positive() {
        let scene = new_usd_scene("Test");
        let pkg = package_usdz(&scene);
        let size = usdz_file_size_estimate(&pkg);
        assert!(size > 0);
    }

    #[test]
    fn test_scene_with_materials_usda() {
        let mut scene = new_usd_scene("MS");
        let mat = default_usd_material("PBR");
        add_usd_material(&mut scene, mat);
        let usda = scene_to_usda(&scene);
        assert!(usda.contains("Materials"));
        assert!(usda.contains("PBR"));
    }
}
