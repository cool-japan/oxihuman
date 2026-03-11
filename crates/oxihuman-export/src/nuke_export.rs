// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nuke .nk script export stub.

/// A Nuke node in the script.
#[derive(Debug, Clone)]
pub struct NukeNode {
    pub class: String,
    pub name: String,
    pub knobs: Vec<(String, String)>,
    pub xpos: i32,
    pub ypos: i32,
}

/// A Nuke script export.
#[derive(Debug, Clone)]
pub struct NukeScriptExport {
    pub version: String,
    pub nodes: Vec<NukeNode>,
}

/// Create a new Nuke script export.
pub fn new_nuke_export(version: &str) -> NukeScriptExport {
    NukeScriptExport {
        version: version.to_string(),
        nodes: Vec::new(),
    }
}

/// Add a node to the script.
pub fn nuke_add_node(export: &mut NukeScriptExport, class: &str, name: &str) {
    export.nodes.push(NukeNode {
        class: class.to_string(),
        name: name.to_string(),
        knobs: Vec::new(),
        xpos: 0,
        ypos: 0,
    });
}

/// Set a knob on the last node.
pub fn nuke_set_knob(export: &mut NukeScriptExport, key: &str, value: &str) {
    if let Some(node) = export.nodes.last_mut() {
        node.knobs.push((key.to_string(), value.to_string()));
    }
}

/// Set the node position.
pub fn nuke_set_position(export: &mut NukeScriptExport, x: i32, y: i32) {
    if let Some(node) = export.nodes.last_mut() {
        node.xpos = x;
        node.ypos = y;
    }
}

/// Return the node count.
pub fn nuke_node_count(export: &NukeScriptExport) -> usize {
    export.nodes.len()
}

/// Serialize the script to a .nk string.
pub fn nuke_to_string(export: &NukeScriptExport) -> String {
    let mut out = format!("# Nuke {}\n", export.version);
    for node in &export.nodes {
        out.push_str(&format!("{} {{\n", node.class));
        out.push_str(&format!(" name {}\n", node.name));
        out.push_str(&format!(" xpos {}\n ypos {}\n", node.xpos, node.ypos));
        for (k, v) in &node.knobs {
            out.push_str(&format!(" {} {}\n", k, v));
        }
        out.push_str("}\n");
    }
    out
}

/// Estimate the .nk file size.
pub fn nuke_size_estimate(export: &NukeScriptExport) -> usize {
    nuke_to_string(export).len()
}

/// Find a node by name.
pub fn nuke_find_node<'a>(export: &'a NukeScriptExport, name: &str) -> Option<&'a NukeNode> {
    export.nodes.iter().find(|n| n.name == name)
}

/// Validate the export (at least a version string).
pub fn validate_nuke(export: &NukeScriptExport) -> bool {
    !export.version.is_empty()
}

/// Count nodes of a specific class.
pub fn nuke_count_by_class(export: &NukeScriptExport, class: &str) -> usize {
    export.nodes.iter().filter(|n| n.class == class).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> NukeScriptExport {
        let mut exp = new_nuke_export("15.0");
        nuke_add_node(&mut exp, "Read", "Read1");
        nuke_set_knob(&mut exp, "file", "/path/to/file.exr");
        exp
    }

    #[test]
    fn test_node_count() {
        let exp = sample();
        assert_eq!(nuke_node_count(&exp), 1);
    }

    #[test]
    fn test_to_string_contains_class() {
        let exp = sample();
        assert!(nuke_to_string(&exp).contains("Read"));
    }

    #[test]
    fn test_validate() {
        let exp = sample();
        assert!(validate_nuke(&exp));
    }

    #[test]
    fn test_find_node() {
        let exp = sample();
        assert!(nuke_find_node(&exp, "Read1").is_some());
        assert!(nuke_find_node(&exp, "NoSuch").is_none());
    }

    #[test]
    fn test_count_by_class() {
        let exp = sample();
        assert_eq!(nuke_count_by_class(&exp, "Read"), 1);
        assert_eq!(nuke_count_by_class(&exp, "Write"), 0);
    }

    #[test]
    fn test_set_position() {
        let mut exp = new_nuke_export("15.0");
        nuke_add_node(&mut exp, "Dot", "Dot1");
        nuke_set_position(&mut exp, 100, 200);
        assert_eq!(exp.nodes[0].xpos, 100);
        assert_eq!(exp.nodes[0].ypos, 200);
    }

    #[test]
    fn test_size_estimate_positive() {
        let exp = sample();
        assert!(nuke_size_estimate(&exp) > 0);
    }

    #[test]
    fn test_knob_count() {
        let exp = sample();
        assert_eq!(exp.nodes[0].knobs.len(), 1);
    }

    #[test]
    fn test_empty_export() {
        let exp = new_nuke_export("15.0");
        assert_eq!(nuke_node_count(&exp), 0);
    }
}
