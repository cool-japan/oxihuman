//! 3MF (3D Manufacturing Format) export stub for additive manufacturing.

// ── Types ────────────────────────────────────────────────────────────────────

/// Unit system used in the 3MF file.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ThreeMfUnits {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

/// Configuration for 3MF export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThreeMfConfig {
    pub units: ThreeMfUnits,
    pub author: String,
    pub include_materials: bool,
}

/// A single mesh in the 3MF model.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThreeMfMesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

/// A complete 3MF model containing one or more meshes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThreeMfModel {
    pub meshes: Vec<ThreeMfMesh>,
    pub name: String,
}

/// Result of converting a model to 3MF XML.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ThreeMfExportResult {
    pub xml_string: String,
    pub triangle_count: usize,
    pub vertex_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`ThreeMfConfig`] using millimetres.
#[allow(dead_code)]
pub fn default_threemf_config() -> ThreeMfConfig {
    ThreeMfConfig {
        units: ThreeMfUnits::Millimeter,
        author: String::from("OxiHuman"),
        include_materials: false,
    }
}

/// Create a new empty [`ThreeMfModel`] with the given name.
#[allow(dead_code)]
pub fn new_threemf_model(name: &str) -> ThreeMfModel {
    ThreeMfModel {
        meshes: Vec::new(),
        name: name.to_string(),
    }
}

/// Append a mesh to an existing model.
#[allow(dead_code)]
pub fn add_mesh_to_model(model: &mut ThreeMfModel, mesh: ThreeMfMesh) {
    model.meshes.push(mesh);
}

/// Construct a [`ThreeMfMesh`] from vertex and triangle data.
#[allow(dead_code)]
pub fn new_threemf_mesh(verts: Vec<[f32; 3]>, tris: Vec<[u32; 3]>) -> ThreeMfMesh {
    ThreeMfMesh {
        vertices: verts,
        triangles: tris,
    }
}

/// Convert a model to a [`ThreeMfExportResult`] containing XML.
#[allow(dead_code)]
pub fn model_to_3mf(model: &ThreeMfModel, cfg: &ThreeMfConfig) -> ThreeMfExportResult {
    let mut xml = String::new();
    xml.push_str(&threemf_header(cfg));
    for mesh in &model.meshes {
        xml.push_str(&threemf_mesh_xml(mesh));
    }
    xml.push_str(&threemf_footer());
    let tri_count = threemf_triangle_count(model);
    let vtx_count = threemf_vertex_count(model);
    ThreeMfExportResult {
        xml_string: xml,
        triangle_count: tri_count,
        vertex_count: vtx_count,
    }
}

/// Generate the XML header for a 3MF file.
#[allow(dead_code)]
pub fn threemf_header(cfg: &ThreeMfConfig) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><model unit="{}" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><metadata name="Designer">{}</metadata><resources>"#,
        units_name_3mf(cfg),
        cfg.author
    )
}

/// Generate XML for a single mesh object.
#[allow(dead_code)]
pub fn threemf_mesh_xml(mesh: &ThreeMfMesh) -> String {
    let mut s = String::from("<object id=\"1\" type=\"model\"><mesh><vertices>");
    for v in &mesh.vertices {
        s.push_str(&format!(
            r#"<vertex x="{:.6}" y="{:.6}" z="{:.6}"/>"#,
            v[0], v[1], v[2]
        ));
    }
    s.push_str("</vertices><triangles>");
    for t in &mesh.triangles {
        s.push_str(&format!(
            r#"<triangle v1="{}" v2="{}" v3="{}"/>"#,
            t[0], t[1], t[2]
        ));
    }
    s.push_str("</triangles></mesh></object>");
    s
}

/// Generate the closing XML footer for a 3MF file.
#[allow(dead_code)]
pub fn threemf_footer() -> String {
    String::from("</resources><build/></model>")
}

/// Count total vertices across all meshes in the model.
#[allow(dead_code)]
pub fn threemf_vertex_count(model: &ThreeMfModel) -> usize {
    model.meshes.iter().map(|m| m.vertices.len()).sum()
}

/// Count total triangles across all meshes in the model.
#[allow(dead_code)]
pub fn threemf_triangle_count(model: &ThreeMfModel) -> usize {
    model.meshes.iter().map(|m| m.triangles.len()).sum()
}

/// Return the 3MF unit string for the configured unit system.
#[allow(dead_code)]
pub fn units_name_3mf(cfg: &ThreeMfConfig) -> &'static str {
    match cfg.units {
        ThreeMfUnits::Millimeter => "millimeter",
        ThreeMfUnits::Centimeter => "centimeter",
        ThreeMfUnits::Meter => "meter",
        ThreeMfUnits::Inch => "inch",
        ThreeMfUnits::Foot => "foot",
    }
}

/// Return `true` if the export result has valid (non-empty) XML and counts.
#[allow(dead_code)]
pub fn validate_3mf(result: &ThreeMfExportResult) -> bool {
    !result.xml_string.is_empty()
        && result.xml_string.contains("model")
        && result.triangle_count > 0
        && result.vertex_count > 0
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mesh() -> ThreeMfMesh {
        new_threemf_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        )
    }

    #[test]
    fn default_config_is_millimeter() {
        let cfg = default_threemf_config();
        assert_eq!(cfg.units, ThreeMfUnits::Millimeter);
        assert!(!cfg.include_materials);
    }

    #[test]
    fn new_model_is_empty() {
        let model = new_threemf_model("test");
        assert_eq!(model.name, "test");
        assert!(model.meshes.is_empty());
    }

    #[test]
    fn add_mesh_increases_count() {
        let mut model = new_threemf_model("m");
        add_mesh_to_model(&mut model, sample_mesh());
        assert_eq!(model.meshes.len(), 1);
    }

    #[test]
    fn vertex_and_triangle_counts() {
        let mut model = new_threemf_model("m");
        add_mesh_to_model(&mut model, sample_mesh());
        assert_eq!(threemf_vertex_count(&model), 3);
        assert_eq!(threemf_triangle_count(&model), 1);
    }

    #[test]
    fn units_name_all_variants() {
        let variants = [
            (ThreeMfUnits::Millimeter, "millimeter"),
            (ThreeMfUnits::Centimeter, "centimeter"),
            (ThreeMfUnits::Meter, "meter"),
            (ThreeMfUnits::Inch, "inch"),
            (ThreeMfUnits::Foot, "foot"),
        ];
        for (unit, expected) in &variants {
            let cfg = ThreeMfConfig {
                units: unit.clone(),
                author: String::new(),
                include_materials: false,
            };
            assert_eq!(units_name_3mf(&cfg), *expected);
        }
    }

    #[test]
    fn model_to_3mf_produces_valid_xml() {
        let mut model = new_threemf_model("body");
        add_mesh_to_model(&mut model, sample_mesh());
        let cfg = default_threemf_config();
        let result = model_to_3mf(&model, &cfg);
        assert!(result.xml_string.contains("<?xml"));
        assert!(result.xml_string.contains("millimeter"));
        assert!(result.xml_string.contains("vertex"));
        assert!(result.xml_string.contains("triangle"));
    }

    #[test]
    fn validate_3mf_passes_for_valid_result() {
        let mut model = new_threemf_model("v");
        add_mesh_to_model(&mut model, sample_mesh());
        let cfg = default_threemf_config();
        let result = model_to_3mf(&model, &cfg);
        assert!(validate_3mf(&result));
    }

    #[test]
    fn validate_3mf_fails_for_empty_model() {
        let result = ThreeMfExportResult {
            xml_string: String::new(),
            triangle_count: 0,
            vertex_count: 0,
        };
        assert!(!validate_3mf(&result));
    }
}
