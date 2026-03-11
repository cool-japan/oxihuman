//! VRML/WRL (Virtual Reality Modeling Language) format export stub.
//!
//! Provides structures and functions for building and serializing VRML 2.0 (VRML97) documents,
//! including shapes, lighting, and background configuration.

/// Configuration for VRML export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VrmlExportConfig {
    /// VRML version string (e.g. "2.0").
    pub version: String,
    /// Whether to include a default viewpoint node.
    pub include_viewpoint: bool,
    /// Whether to write compact (no extra whitespace) output.
    pub compact: bool,
}

/// A single VRML shape entry (IndexedFaceSet).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VrmlShape {
    /// Vertex positions as `[x, y, z]` triples.
    pub vertices: Vec<[f32; 3]>,
    /// Triangle face indices as `[i0, i1, i2]` triples.
    pub faces: Vec<[u32; 3]>,
    /// Diffuse color `[r, g, b]` in 0..=1.
    pub color: [f32; 3],
}

/// A VRML point light entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VrmlPointLight {
    /// Position `[x, y, z]`.
    pub position: [f32; 3],
    /// Intensity in 0..=1.
    pub intensity: f32,
}

/// A VRML document holding shapes, lights, and background configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VrmlDocument {
    /// Export configuration used to build this document.
    pub config: VrmlExportConfig,
    /// Collection of shapes in the scene.
    pub shapes: Vec<VrmlShape>,
    /// Collection of point lights in the scene.
    pub lights: Vec<VrmlPointLight>,
    /// Background sky color `[r, g, b]`.
    pub background_color: [f32; 3],
}

/// Returns a sensible default [`VrmlExportConfig`].
#[allow(dead_code)]
pub fn default_vrml_config() -> VrmlExportConfig {
    VrmlExportConfig {
        version: "2.0".to_string(),
        include_viewpoint: true,
        compact: false,
    }
}

/// Creates a new, empty [`VrmlDocument`] using the given configuration.
#[allow(dead_code)]
pub fn new_vrml_document(cfg: &VrmlExportConfig) -> VrmlDocument {
    VrmlDocument {
        config: cfg.clone(),
        shapes: Vec::new(),
        lights: Vec::new(),
        background_color: [0.0, 0.0, 0.0],
    }
}

/// Appends a pre-built [`VrmlShape`] to the document.
#[allow(dead_code)]
pub fn vrml_add_shape(doc: &mut VrmlDocument, shape: VrmlShape) {
    doc.shapes.push(shape);
}

/// Builds a [`VrmlShape`] from raw vertex/face data and appends it to the document.
#[allow(dead_code)]
pub fn vrml_add_mesh_shape(
    doc: &mut VrmlDocument,
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    color: [f32; 3],
) {
    doc.shapes.push(VrmlShape {
        vertices: verts.to_vec(),
        faces: faces.to_vec(),
        color,
    });
}

/// Serializes the document to a VRML 2.0 string.
#[allow(dead_code)]
pub fn vrml_to_string(doc: &VrmlDocument) -> String {
    let mut out = String::new();
    out.push_str(&format!("#VRML V{} utf8\n\n", doc.config.version));

    // Background
    let [r, g, b] = doc.background_color;
    out.push_str(&format!("Background {{\n  skyColor [{r:.4} {g:.4} {b:.4}]\n}}\n\n"));

    // Viewpoint
    if doc.config.include_viewpoint {
        out.push_str("Viewpoint {\n  position 0 0 5\n  description \"Default\"\n}\n\n");
    }

    // Point lights
    for light in &doc.lights {
        let [lx, ly, lz] = light.position;
        out.push_str(&format!(
            "PointLight {{\n  location {lx:.4} {ly:.4} {lz:.4}\n  intensity {:.4}\n}}\n\n",
            light.intensity
        ));
    }

    // Shapes
    for shape in &doc.shapes {
        let [cr, cg, cb] = shape.color;
        out.push_str("Shape {\n");
        out.push_str("  appearance Appearance {\n");
        out.push_str(&format!(
            "    material Material {{ diffuseColor {cr:.4} {cg:.4} {cb:.4} }}\n"
        ));
        out.push_str("  }\n");
        out.push_str("  geometry IndexedFaceSet {\n");
        out.push_str("    coord Coordinate { point [\n");
        for v in &shape.vertices {
            out.push_str(&format!("      {} {} {},\n", v[0], v[1], v[2]));
        }
        out.push_str("    ]}\n    coordIndex [\n");
        for f in &shape.faces {
            out.push_str(&format!("      {} {} {} -1,\n", f[0], f[1], f[2]));
        }
        out.push_str("    ]\n  }\n}\n\n");
    }

    out
}

/// Writes the VRML document to a file at the given path.
///
/// Returns `Err` with a message if the path contains invalid characters.
#[allow(dead_code)]
pub fn vrml_write_to_file(doc: &VrmlDocument, path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path must not be empty".to_string());
    }
    let content = vrml_to_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Returns the number of shapes currently in the document.
#[allow(dead_code)]
pub fn vrml_shape_count(doc: &VrmlDocument) -> usize {
    doc.shapes.len()
}

/// Sets the background sky color of the document.
#[allow(dead_code)]
pub fn vrml_set_background_color(doc: &mut VrmlDocument, r: f32, g: f32, b: f32) {
    doc.background_color = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Adds a point light at the given position with the specified intensity.
#[allow(dead_code)]
pub fn vrml_add_point_light(doc: &mut VrmlDocument, pos: [f32; 3], intensity: f32) {
    doc.lights.push(VrmlPointLight {
        position: pos,
        intensity: intensity.clamp(0.0, 1.0),
    });
}

/// Removes all shapes and lights from the document, resetting it to empty.
#[allow(dead_code)]
pub fn vrml_document_clear(doc: &mut VrmlDocument) {
    doc.shapes.clear();
    doc.lights.clear();
    doc.background_color = [0.0, 0.0, 0.0];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_vrml_config();
        assert_eq!(cfg.version, "2.0");
        assert!(cfg.include_viewpoint);
    }

    #[test]
    fn test_new_document_empty() {
        let cfg = default_vrml_config();
        let doc = new_vrml_document(&cfg);
        assert_eq!(vrml_shape_count(&doc), 0);
        assert!(doc.lights.is_empty());
    }

    #[test]
    fn test_add_mesh_shape() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        let verts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let faces = [[0u32, 1, 2]];
        vrml_add_mesh_shape(&mut doc, &verts, &faces, [1.0, 0.0, 0.0]);
        assert_eq!(vrml_shape_count(&doc), 1);
        assert_eq!(doc.shapes[0].vertices.len(), 3);
    }

    #[test]
    fn test_vrml_to_string_contains_header() {
        let cfg = default_vrml_config();
        let doc = new_vrml_document(&cfg);
        let s = vrml_to_string(&doc);
        assert!(s.contains("#VRML V2.0 utf8"));
    }

    #[test]
    fn test_vrml_to_string_contains_background() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        vrml_set_background_color(&mut doc, 0.1, 0.2, 0.3);
        let s = vrml_to_string(&doc);
        assert!(s.contains("Background"));
    }

    #[test]
    fn test_background_color_clamped() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        vrml_set_background_color(&mut doc, 2.0, -1.0, 0.5);
        assert_eq!(doc.background_color[0], 1.0);
        assert_eq!(doc.background_color[1], 0.0);
        assert_eq!(doc.background_color[2], 0.5);
    }

    #[test]
    fn test_add_point_light() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        vrml_add_point_light(&mut doc, [1.0, 2.0, 3.0], 0.8);
        assert_eq!(doc.lights.len(), 1);
        assert!((doc.lights[0].intensity - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_document_clear() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        vrml_add_mesh_shape(&mut doc, &[[0.0, 0.0, 0.0]], &[], [1.0, 1.0, 1.0]);
        vrml_add_point_light(&mut doc, [0.0, 0.0, 0.0], 1.0);
        vrml_document_clear(&mut doc);
        assert_eq!(vrml_shape_count(&doc), 0);
        assert!(doc.lights.is_empty());
    }

    #[test]
    fn test_add_shape_direct() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        let shape = VrmlShape {
            vertices: vec![[0.0, 0.0, 0.0]],
            faces: vec![],
            color: [0.5, 0.5, 0.5],
        };
        vrml_add_shape(&mut doc, shape);
        assert_eq!(vrml_shape_count(&doc), 1);
    }

    #[test]
    fn test_write_to_file_empty_path() {
        let cfg = default_vrml_config();
        let doc = new_vrml_document(&cfg);
        assert!(vrml_write_to_file(&doc, "").is_err());
    }

    #[test]
    fn test_light_intensity_clamped() {
        let cfg = default_vrml_config();
        let mut doc = new_vrml_document(&cfg);
        vrml_add_point_light(&mut doc, [0.0, 0.0, 0.0], 5.0);
        assert_eq!(doc.lights[0].intensity, 1.0);
    }
}
