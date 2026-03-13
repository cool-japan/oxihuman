// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export per-node transform data (position, rotation, scale) as JSON or CSV.

#![allow(dead_code)]

/// Configuration for transform export.
#[derive(Debug, Clone)]
pub struct XformExportConfig {
    /// Whether to include rotation data.
    pub include_rotation: bool,
    /// Whether to include scale data.
    pub include_scale: bool,
    /// Number of decimal places for floating-point values.
    pub precision: usize,
}

/// Per-node transform snapshot.
#[derive(Debug, Clone)]
pub struct NodeXform {
    /// Node name.
    pub name: String,
    /// World-space position [x, y, z].
    pub position: [f64; 3],
    /// Quaternion rotation [x, y, z, w].
    pub rotation: [f64; 4],
    /// Scale [x, y, z].
    pub scale: [f64; 3],
}

/// Result of a transform export operation.
#[derive(Debug, Clone)]
pub struct XformExportResult {
    /// Nodes stored in the exporter.
    pub nodes: Vec<NodeXform>,
    /// Total bytes of the last export payload.
    pub total_bytes: usize,
}

/// Returns the default [`XformExportConfig`].
#[allow(dead_code)]
pub fn default_xform_export_config() -> XformExportConfig {
    XformExportConfig {
        include_rotation: true,
        include_scale: true,
        precision: 6,
    }
}

/// Creates a new, empty [`XformExportResult`].
#[allow(dead_code)]
pub fn new_xform_export() -> XformExportResult {
    XformExportResult {
        nodes: Vec::new(),
        total_bytes: 0,
    }
}

/// Adds a [`NodeXform`] to the result.
#[allow(dead_code)]
pub fn xform_add_node(result: &mut XformExportResult, node: NodeXform) {
    result.nodes.push(node);
}

/// Serialises all nodes as a JSON string.
#[allow(dead_code)]
pub fn xform_export_to_json(result: &XformExportResult, cfg: &XformExportConfig) -> String {
    let prec = cfg.precision;
    let mut out = String::from("[\n");
    for (i, n) in result.nodes.iter().enumerate() {
        let comma = if i + 1 < result.nodes.len() { "," } else { "" };
        let pos = format!(
            "[{:.prec$},{:.prec$},{:.prec$}]",
            n.position[0], n.position[1], n.position[2]
        );
        let mut fields = format!("  {{\"name\":\"{}\",\"position\":{}", n.name, pos);
        if cfg.include_rotation {
            let rot = format!(
                "[{:.prec$},{:.prec$},{:.prec$},{:.prec$}]",
                n.rotation[0], n.rotation[1], n.rotation[2], n.rotation[3]
            );
            fields.push_str(&format!(",\"rotation\":{}", rot));
        }
        if cfg.include_scale {
            let scl = format!(
                "[{:.prec$},{:.prec$},{:.prec$}]",
                n.scale[0], n.scale[1], n.scale[2]
            );
            fields.push_str(&format!(",\"scale\":{}", scl));
        }
        fields.push('}');
        out.push_str(&fields);
        out.push_str(comma);
        out.push('\n');
    }
    out.push(']');
    out
}

/// Serialises all nodes as CSV.
#[allow(dead_code)]
pub fn xform_export_to_csv(result: &XformExportResult, cfg: &XformExportConfig) -> String {
    let prec = cfg.precision;
    let mut out = String::from("name,px,py,pz");
    if cfg.include_rotation {
        out.push_str(",rx,ry,rz,rw");
    }
    if cfg.include_scale {
        out.push_str(",sx,sy,sz");
    }
    out.push('\n');
    for n in &result.nodes {
        let row = format!(
            "{},{:.prec$},{:.prec$},{:.prec$}",
            n.name, n.position[0], n.position[1], n.position[2]
        );
        out.push_str(&row);
        if cfg.include_rotation {
            out.push_str(&format!(
                ",{:.prec$},{:.prec$},{:.prec$},{:.prec$}",
                n.rotation[0], n.rotation[1], n.rotation[2], n.rotation[3]
            ));
        }
        if cfg.include_scale {
            out.push_str(&format!(
                ",{:.prec$},{:.prec$},{:.prec$}",
                n.scale[0], n.scale[1], n.scale[2]
            ));
        }
        out.push('\n');
    }
    out
}

/// Returns the number of nodes stored.
#[allow(dead_code)]
pub fn xform_node_count(result: &XformExportResult) -> usize {
    result.nodes.len()
}

/// Finds a node by name; returns `None` if not found.
#[allow(dead_code)]
pub fn xform_node_by_name<'a>(result: &'a XformExportResult, name: &str) -> Option<&'a NodeXform> {
    result.nodes.iter().find(|n| n.name == name)
}

/// Writes the JSON export to a file (stub – returns byte count).
#[allow(dead_code)]
pub fn xform_export_write_to_file(
    result: &mut XformExportResult,
    cfg: &XformExportConfig,
    _path: &str,
) -> usize {
    let json = xform_export_to_json(result, cfg);
    result.total_bytes = json.len();
    result.total_bytes
}

/// Clears all nodes.
#[allow(dead_code)]
pub fn xform_export_clear(result: &mut XformExportResult) {
    result.nodes.clear();
    result.total_bytes = 0;
}

/// Returns the total byte count of the last export.
#[allow(dead_code)]
pub fn xform_total_bytes(result: &XformExportResult) -> usize {
    result.total_bytes
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn sample_node(name: &str) -> NodeXform {
    NodeXform {
        name: name.to_string(),
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [1.0, 1.0, 1.0],
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_rotation_and_scale() {
        let cfg = default_xform_export_config();
        assert!(cfg.include_rotation);
        assert!(cfg.include_scale);
        assert_eq!(cfg.precision, 6);
    }

    #[test]
    fn new_xform_export_is_empty() {
        let r = new_xform_export();
        assert_eq!(xform_node_count(&r), 0);
    }

    #[test]
    fn add_node_increases_count() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("root"));
        assert_eq!(xform_node_count(&r), 1);
    }

    #[test]
    fn xform_node_by_name_found() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("spine"));
        let found = xform_node_by_name(&r, "spine");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").name, "spine");
    }

    #[test]
    fn xform_node_by_name_not_found() {
        let r = new_xform_export();
        assert!(xform_node_by_name(&r, "missing").is_none());
    }

    #[test]
    fn to_json_contains_name_and_position() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("hips"));
        let cfg = default_xform_export_config();
        let json = xform_export_to_json(&r, &cfg);
        assert!(json.contains("\"hips\""));
        assert!(json.contains("\"position\""));
        assert!(json.contains("\"rotation\""));
        assert!(json.contains("\"scale\""));
    }

    #[test]
    fn to_csv_header_present() {
        let r = new_xform_export();
        let cfg = default_xform_export_config();
        let csv = xform_export_to_csv(&r, &cfg);
        assert!(csv.starts_with("name,px,py,pz"));
    }

    #[test]
    fn write_to_file_updates_total_bytes() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("foot"));
        let cfg = default_xform_export_config();
        let bytes = xform_export_write_to_file(&mut r, &cfg, "/tmp/test_xform.json");
        assert!(bytes > 0);
        assert_eq!(xform_total_bytes(&r), bytes);
    }

    #[test]
    fn clear_resets_state() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("hand"));
        xform_export_clear(&mut r);
        assert_eq!(xform_node_count(&r), 0);
        assert_eq!(xform_total_bytes(&r), 0);
    }

    #[test]
    fn multiple_nodes_in_json() {
        let mut r = new_xform_export();
        xform_add_node(&mut r, sample_node("head"));
        xform_add_node(&mut r, sample_node("neck"));
        xform_add_node(&mut r, sample_node("chest"));
        let cfg = default_xform_export_config();
        let json = xform_export_to_json(&r, &cfg);
        assert!(json.contains("\"head\""));
        assert!(json.contains("\"neck\""));
        assert!(json.contains("\"chest\""));
        assert_eq!(xform_node_count(&r), 3);
    }
}
