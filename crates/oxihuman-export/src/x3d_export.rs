// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! X3D / VRML export stub for web-3D interchange.
//!
//! Provides a scene-graph builder that emits X3D XML without requiring
//! a mesh buffer — suitable for constructing X3D documents programmatically.

#![allow(dead_code)]

// ── X3dExportConfig ───────────────────────────────────────────────────────────

/// Configuration for the X3D export stub.
#[derive(Debug, Clone)]
pub struct X3dExportConfig {
    /// X3D specification version string.  Default: `"3.3"`.
    pub version: String,
    /// X3D profile.  Default: `"Interchange"`.
    pub profile: String,
    /// Number of spaces per indentation level.  Default: `2`.
    pub indent: usize,
    /// Background RGB colour (linear, 0.0–1.0).  Default: `[0.0, 0.0, 0.0]`.
    pub background_color: [f32; 3],
}

impl Default for X3dExportConfig {
    fn default() -> Self {
        X3dExportConfig {
            version: "3.3".to_string(),
            profile: "Interchange".to_string(),
            indent: 2,
            background_color: [0.0, 0.0, 0.0],
        }
    }
}

// ── X3dNode ───────────────────────────────────────────────────────────────────

/// A generic node in an X3D scene graph.
#[derive(Debug, Clone)]
pub struct X3dNode {
    /// X3D element type (e.g. `"Shape"`, `"Transform"`, `"Group"`).
    pub node_type: String,
    /// Optional `DEF` name for this node.
    pub def_name: Option<String>,
    /// Flat list of attribute key-value pairs for this element.
    pub attributes: Vec<(String, String)>,
    /// Child node indices within the owning `X3dScene`.
    pub children: Vec<usize>,
}

// ── X3dScene ──────────────────────────────────────────────────────────────────

/// An X3D scene containing a flat node registry.
#[derive(Debug, Clone)]
pub struct X3dScene {
    /// Human-readable scene name (used in a `<meta>` element).
    pub name: String,
    /// All nodes in the scene, referenced by index.
    pub nodes: Vec<X3dNode>,
    /// Export configuration.
    pub config: X3dExportConfig,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Validation result type.
pub type X3dValidationResult = Result<(), String>;

// ── Constructor functions ─────────────────────────────────────────────────────

/// Return a default `X3dExportConfig`.
#[allow(dead_code)]
pub fn default_x3d_config() -> X3dExportConfig {
    X3dExportConfig::default()
}

/// Create an empty `X3dScene` with the given name and configuration.
#[allow(dead_code)]
pub fn new_x3d_scene(name: &str, config: X3dExportConfig) -> X3dScene {
    X3dScene {
        name: name.to_string(),
        nodes: Vec::new(),
        config,
    }
}

// ── Node operations ───────────────────────────────────────────────────────────

/// Append `node` to the scene and return its index.
#[allow(dead_code)]
pub fn add_x3d_node(scene: &mut X3dScene, node: X3dNode) -> usize {
    let idx = scene.nodes.len();
    scene.nodes.push(node);
    idx
}

/// Return the total number of nodes in the scene.
#[allow(dead_code)]
pub fn x3d_node_count(scene: &X3dScene) -> usize {
    scene.nodes.len()
}

/// Return the node type string for the node at `idx`, or `None` if out of range.
#[allow(dead_code)]
pub fn x3d_node_type(scene: &X3dScene, idx: usize) -> Option<&str> {
    scene.nodes.get(idx).map(|n| n.node_type.as_str())
}

// ── Scene metadata ────────────────────────────────────────────────────────────

/// Return the scene name.
#[allow(dead_code)]
pub fn x3d_scene_name(scene: &X3dScene) -> &str {
    &scene.name
}

/// Set the scene name.
#[allow(dead_code)]
pub fn set_scene_name(scene: &mut X3dScene, name: &str) {
    scene.name = name.to_string();
}

/// Return the X3D version string from the scene's config.
#[allow(dead_code)]
pub fn x3d_version(scene: &X3dScene) -> &str {
    &scene.config.version
}

/// Return the background colour from the scene's config.
#[allow(dead_code)]
pub fn x3d_background_color(scene: &X3dScene) -> [f32; 3] {
    scene.config.background_color
}

// ── Serialisation ─────────────────────────────────────────────────────────────

/// XML-escape a string for use in attribute values.
fn xml_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Return `n` spaces.
fn spaces(n: usize) -> String {
    " ".repeat(n)
}

/// Serialise one node (and its children recursively) into XML.
fn node_to_xml(scene: &X3dScene, idx: usize, depth: usize) -> String {
    let node = &scene.nodes[idx];
    let pad = spaces(depth * scene.config.indent);
    let mut out = String::new();

    // Opening tag
    out.push_str(&format!("{}<{}", pad, node.node_type));
    if let Some(ref def) = node.def_name {
        out.push_str(&format!(" DEF=\"{}\"", xml_escape_attr(def)));
    }
    for (k, v) in &node.attributes {
        out.push_str(&format!(" {}=\"{}\"", k, xml_escape_attr(v)));
    }

    if node.children.is_empty() {
        out.push_str("/>\n");
    } else {
        out.push_str(">\n");
        for &child in &node.children {
            if child < scene.nodes.len() {
                out.push_str(&node_to_xml(scene, child, depth + 1));
            }
        }
        out.push_str(&format!("{}</{}>\n", pad, node.node_type));
    }

    out
}

/// Serialise the entire scene to an X3D XML string (stub).
///
/// Top-level nodes are those not referenced as children of any other node.
#[allow(dead_code)]
pub fn x3d_to_xml_stub(scene: &X3dScene) -> String {
    let cfg = &scene.config;
    let sp1 = spaces(cfg.indent);
    let sp2 = spaces(cfg.indent * 2);

    let mut out = String::new();

    // XML declaration
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    // Root element
    out.push_str(&format!(
        "<X3D profile=\"{}\" version=\"{}\"\n",
        xml_escape_attr(&cfg.profile),
        xml_escape_attr(&cfg.version)
    ));
    out.push_str(
        "     xmlns:xsd=\"http://www.w3.org/2001/XMLSchema-instance\"\
         \n     xsd:noNamespaceSchemaLocation=\"http://www.web3d.org/specifications/x3d-3.3.xsd\">\n",
    );

    // Head
    out.push_str(&format!("{}<head>\n", sp1));
    if !scene.name.is_empty() {
        out.push_str(&format!(
            "{}<meta name=\"title\" content=\"{}\"/>\n",
            sp2,
            xml_escape_attr(&scene.name)
        ));
    }
    out.push_str(&format!(
        "{}<meta name=\"generator\" content=\"OxiHuman x3d_export\"/>\n",
        sp2
    ));
    out.push_str(&format!("{}</head>\n", sp1));

    // Scene
    out.push_str(&format!("{}<Scene>\n", sp1));

    // Background node
    let bg = cfg.background_color;
    out.push_str(&format!(
        "{}<Background skyColor=\"{:.4} {:.4} {:.4}\"/>\n",
        sp2, bg[0], bg[1], bg[2]
    ));

    // Collect top-level nodes (not referenced as children)
    let mut referenced: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for node in &scene.nodes {
        for &c in &node.children {
            referenced.insert(c);
        }
    }

    for idx in 0..scene.nodes.len() {
        if !referenced.contains(&idx) {
            out.push_str(&node_to_xml(scene, idx, 2));
        }
    }

    out.push_str(&format!("{}</Scene>\n", sp1));
    out.push_str("</X3D>\n");

    out
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate a serialised X3D XML string for basic structural requirements.
#[allow(dead_code)]
pub fn validate_x3d_export(content: &str) -> X3dValidationResult {
    if !content.starts_with("<?xml") {
        return Err("Missing XML declaration".to_string());
    }
    if !content.contains("<X3D") {
        return Err("Missing <X3D> root element".to_string());
    }
    if !content.contains("profile=") {
        return Err("Missing 'profile' attribute".to_string());
    }
    if !content.contains("<Scene") {
        return Err("Missing <Scene> element".to_string());
    }
    if !content.contains("</X3D>") {
        return Err("Missing </X3D> closing tag".to_string());
    }
    Ok(())
}

// ── File size estimate ────────────────────────────────────────────────────────

/// Estimate output file size in bytes.
///
/// Uses ~80 bytes per node plus a fixed overhead for the XML boilerplate.
#[allow(dead_code)]
pub fn x3d_file_size_estimate(scene: &X3dScene) -> usize {
    512 + scene.nodes.len() * 80
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_shape_node(name: &str) -> X3dNode {
        X3dNode {
            node_type: "Shape".to_string(),
            def_name: Some(name.to_string()),
            attributes: vec![],
            children: vec![],
        }
    }

    fn make_scene() -> X3dScene {
        new_x3d_scene("TestScene", default_x3d_config())
    }

    // 1 – default_x3d_config
    #[test]
    fn test_default_x3d_config_version() {
        let cfg = default_x3d_config();
        assert_eq!(cfg.version, "3.3");
    }

    // 2 – default_x3d_config profile
    #[test]
    fn test_default_x3d_config_profile() {
        let cfg = default_x3d_config();
        assert_eq!(cfg.profile, "Interchange");
    }

    // 3 – new_x3d_scene
    #[test]
    fn test_new_x3d_scene_name() {
        let scene = new_x3d_scene("MyScene", default_x3d_config());
        assert_eq!(scene.name, "MyScene");
        assert!(scene.nodes.is_empty());
    }

    // 4 – add_x3d_node / x3d_node_count
    #[test]
    fn test_add_x3d_node_count() {
        let mut scene = make_scene();
        add_x3d_node(&mut scene, make_shape_node("A"));
        add_x3d_node(&mut scene, make_shape_node("B"));
        assert_eq!(x3d_node_count(&scene), 2);
    }

    // 5 – add_x3d_node returns index
    #[test]
    fn test_add_x3d_node_returns_index() {
        let mut scene = make_scene();
        let idx0 = add_x3d_node(&mut scene, make_shape_node("First"));
        let idx1 = add_x3d_node(&mut scene, make_shape_node("Second"));
        assert_eq!(idx0, 0);
        assert_eq!(idx1, 1);
    }

    // 6 – x3d_node_type
    #[test]
    fn test_x3d_node_type() {
        let mut scene = make_scene();
        add_x3d_node(&mut scene, make_shape_node("X"));
        assert_eq!(x3d_node_type(&scene, 0), Some("Shape"));
    }

    // 7 – x3d_node_type out of range
    #[test]
    fn test_x3d_node_type_out_of_range() {
        let scene = make_scene();
        assert!(x3d_node_type(&scene, 99).is_none());
    }

    // 8 – x3d_scene_name / set_scene_name
    #[test]
    fn test_scene_name_set_get() {
        let mut scene = make_scene();
        set_scene_name(&mut scene, "Renamed");
        assert_eq!(x3d_scene_name(&scene), "Renamed");
    }

    // 9 – x3d_version
    #[test]
    fn test_x3d_version() {
        let scene = make_scene();
        assert_eq!(x3d_version(&scene), "3.3");
    }

    // 10 – x3d_background_color default
    #[test]
    fn test_x3d_background_color_default() {
        let scene = make_scene();
        let bg = x3d_background_color(&scene);
        assert_eq!(bg, [0.0, 0.0, 0.0]);
    }

    // 11 – x3d_background_color custom
    #[test]
    fn test_x3d_background_color_custom() {
        let mut cfg = default_x3d_config();
        cfg.background_color = [0.2, 0.4, 0.6];
        let scene = new_x3d_scene("S", cfg);
        let bg = x3d_background_color(&scene);
        assert!((bg[0] - 0.2).abs() < 1e-6);
    }

    // 12 – x3d_to_xml_stub contains XML declaration
    #[test]
    fn test_x3d_to_xml_stub_declaration() {
        let scene = make_scene();
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.starts_with("<?xml"), "missing XML declaration");
    }

    // 13 – x3d_to_xml_stub contains required elements
    #[test]
    fn test_x3d_to_xml_stub_structure() {
        let scene = make_scene();
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.contains("<X3D"), "missing <X3D>");
        assert!(xml.contains("</X3D>"), "missing </X3D>");
        assert!(xml.contains("<Scene"), "missing <Scene>");
        assert!(xml.contains("</Scene>"), "missing </Scene>");
    }

    // 14 – x3d_to_xml_stub contains scene name
    #[test]
    fn test_x3d_to_xml_stub_scene_name() {
        let scene = new_x3d_scene("BodyScene", default_x3d_config());
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.contains("BodyScene"), "scene name not in XML");
    }

    // 15 – x3d_to_xml_stub with node
    #[test]
    fn test_x3d_to_xml_stub_with_node() {
        let mut scene = make_scene();
        add_x3d_node(&mut scene, make_shape_node("Mesh1"));
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.contains("DEF=\"Mesh1\""), "node DEF not found in XML");
    }

    // 16 – validate_x3d_export passes for correct XML
    #[test]
    fn test_validate_x3d_export_ok() {
        let scene = make_scene();
        let xml = x3d_to_xml_stub(&scene);
        assert!(validate_x3d_export(&xml).is_ok());
    }

    // 17 – validate_x3d_export fails missing declaration
    #[test]
    fn test_validate_x3d_export_missing_decl() {
        let bad = "<X3D profile=\"Interchange\"><Scene></Scene></X3D>";
        assert!(validate_x3d_export(bad).is_err());
    }

    // 18 – validate_x3d_export fails missing scene
    #[test]
    fn test_validate_x3d_export_missing_scene() {
        let bad = "<?xml version=\"1.0\"?><X3D profile=\"X\" version=\"3.3\"></X3D>";
        assert!(validate_x3d_export(bad).is_err());
    }

    // 19 – x3d_file_size_estimate is positive
    #[test]
    fn test_x3d_file_size_estimate_empty() {
        let scene = make_scene();
        assert!(x3d_file_size_estimate(&scene) > 0);
    }

    // 20 – x3d_file_size_estimate grows with nodes
    #[test]
    fn test_x3d_file_size_estimate_grows() {
        let mut scene = make_scene();
        let base = x3d_file_size_estimate(&scene);
        add_x3d_node(&mut scene, make_shape_node("N1"));
        add_x3d_node(&mut scene, make_shape_node("N2"));
        let after = x3d_file_size_estimate(&scene);
        assert!(after > base, "size estimate should grow with more nodes");
    }

    // 21 – node with attributes serialised
    #[test]
    fn test_node_attributes_in_xml() {
        let mut scene = make_scene();
        let node = X3dNode {
            node_type: "Transform".to_string(),
            def_name: None,
            attributes: vec![
                ("translation".to_string(), "1 2 3".to_string()),
            ],
            children: vec![],
        };
        add_x3d_node(&mut scene, node);
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.contains("translation=\"1 2 3\""), "attribute not found in XML");
    }

    // 22 – background color in XML
    #[test]
    fn test_background_in_xml() {
        let mut cfg = default_x3d_config();
        cfg.background_color = [1.0, 0.0, 0.0];
        let scene = new_x3d_scene("S", cfg);
        let xml = x3d_to_xml_stub(&scene);
        assert!(xml.contains("Background"), "Background node missing");
        assert!(xml.contains("skyColor"), "skyColor attribute missing");
    }

    // 23 – x3d_node_count empty
    #[test]
    fn test_x3d_node_count_empty() {
        let scene = make_scene();
        assert_eq!(x3d_node_count(&scene), 0);
    }
}
