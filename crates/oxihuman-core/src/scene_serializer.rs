// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Scene serializer — converts a scene graph snapshot to/from a flat
//! JSON-like string representation.
//!
//! This module intentionally avoids a full `serde` dependency and instead
//! generates simple, human-readable text that can be round-tripped via the
//! stub deserializer provided here.

#[allow(dead_code)]
/// A lightweight scene node holding an id, name and child ids.
pub struct SerializerSceneNode {
    pub id: u64,
    pub name: String,
    pub children: Vec<u64>,
}

#[allow(dead_code)]
/// Configuration for the scene serializer.
pub struct SceneSerializerConfig {
    /// Whether to include child ids in the serialized output.
    pub include_children: bool,
    /// Indent level used in human-readable output (0 = compact).
    pub indent: usize,
}

#[allow(dead_code)]
/// The result of serialising a collection of scene nodes.
pub struct SerializedScene {
    pub node_count: usize,
    pub payload: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

#[allow(dead_code)]
/// Returns a sensible default [`SceneSerializerConfig`].
pub fn default_scene_serializer_config() -> SceneSerializerConfig {
    SceneSerializerConfig {
        include_children: true,
        indent: 0,
    }
}

#[allow(dead_code)]
/// Creates a new [`SerializerSceneNode`] with the given `id` and `name`.
pub fn new_scene_node(id: u64, name: &str) -> SerializerSceneNode {
    SerializerSceneNode {
        id,
        name: name.to_string(),
        children: Vec::new(),
    }
}

#[allow(dead_code)]
/// Appends `child_id` to the node's children list.
pub fn scene_node_add_child(node: &mut SerializerSceneNode, child_id: u64) {
    if !node.children.contains(&child_id) {
        node.children.push(child_id);
    }
}

#[allow(dead_code)]
/// Serialises `nodes` into a [`SerializedScene`] using `cfg`.
pub fn serialize_scene(
    nodes: &[SerializerSceneNode],
    cfg: &SceneSerializerConfig,
) -> SerializedScene {
    let mut parts: Vec<String> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let children_str = if cfg.include_children && !node.children.is_empty() {
            let ids: Vec<String> = node.children.iter().map(|c| c.to_string()).collect();
            format!(",\"children\":[{}]", ids.join(","))
        } else {
            String::new()
        };
        parts.push(format!(
            "{{\"id\":{},\"name\":\"{}\"{}}}",
            node.id, node.name, children_str
        ));
    }
    let payload = format!("[{}]", parts.join(","));
    SerializedScene {
        node_count: nodes.len(),
        payload,
    }
}

#[allow(dead_code)]
/// Returns the serialised scene as a `&str`-compatible `String`.
pub fn serialized_scene_to_string(scene: &SerializedScene) -> String {
    scene.payload.clone()
}

#[allow(dead_code)]
/// Returns the number of nodes stored in `scene`.
pub fn scene_node_count(scene: &SerializedScene) -> usize {
    scene.node_count
}

#[allow(dead_code)]
/// Stub deserialiser — parses a minimal serialised form and returns nodes.
/// Only `id` and `name` fields are recovered; children are not parsed.
pub fn deserialize_scene_stub(data: &str) -> Vec<SerializerSceneNode> {
    let mut nodes = Vec::new();
    // Very minimal parser: find every {"id":<n>,"name":"<s>" … } block.
    let trimmed = data.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return nodes;
    }
    for block in trimmed.split("},{") {
        let block = block.trim_start_matches('{').trim_end_matches('}');
        let mut id: Option<u64> = None;
        let mut name = String::new();
        for pair in block.split(',') {
            let pair = pair.trim();
            if let Some(rest) = pair.strip_prefix("\"id\":") {
                if let Ok(v) = rest.trim().parse::<u64>() {
                    id = Some(v);
                }
            } else if let Some(rest) = pair.strip_prefix("\"name\":") {
                let raw = rest.trim();
                name = raw.trim_matches('"').to_string();
            }
        }
        if let Some(node_id) = id {
            nodes.push(new_scene_node(node_id, &name));
        }
    }
    nodes
}

#[allow(dead_code)]
/// Returns the depth of node `id` in the scene tree (0 = root, usize::MAX if not found).
pub fn scene_node_depth(nodes: &[SerializerSceneNode], id: u64) -> usize {
    // Build a parent map
    let mut parent: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    for node in nodes {
        for &child in &node.children {
            parent.insert(child, node.id);
        }
    }
    let mut depth = 0usize;
    let mut current = id;
    let mut visited = std::collections::HashSet::new();
    while let Some(&p) = parent.get(&current) {
        if !visited.insert(current) {
            break; // cycle guard
        }
        current = p;
        depth += 1;
    }
    // Verify the original id actually exists
    if nodes.iter().any(|n| n.id == id) {
        depth
    } else {
        usize::MAX
    }
}

#[allow(dead_code)]
/// Finds and returns a reference to the node with the given `id`, if any.
pub fn find_scene_node(nodes: &[SerializerSceneNode], id: u64) -> Option<&SerializerSceneNode> {
    nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
/// Returns the ids of all root nodes (nodes that are not a child of any other node).
pub fn scene_root_nodes(nodes: &[SerializerSceneNode]) -> Vec<u64> {
    let all_children: std::collections::HashSet<u64> =
        nodes.iter().flat_map(|n| n.children.iter().copied()).collect();
    nodes
        .iter()
        .filter(|n| !all_children.contains(&n.id))
        .map(|n| n.id)
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene() -> Vec<SerializerSceneNode> {
        let mut root = new_scene_node(1, "root");
        let mut child = new_scene_node(2, "child_a");
        scene_node_add_child(&mut root, 2);
        scene_node_add_child(&mut child, 3);
        let leaf = new_scene_node(3, "leaf");
        vec![root, child, leaf]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_scene_serializer_config();
        assert!(cfg.include_children);
        assert_eq!(cfg.indent, 0);
    }

    #[test]
    fn test_new_scene_node() {
        let n = new_scene_node(42, "camera");
        assert_eq!(n.id, 42);
        assert_eq!(n.name, "camera");
        assert!(n.children.is_empty());
    }

    #[test]
    fn test_add_child_no_duplicates() {
        let mut n = new_scene_node(1, "root");
        scene_node_add_child(&mut n, 2);
        scene_node_add_child(&mut n, 2);
        assert_eq!(n.children.len(), 1);
    }

    #[test]
    fn test_serialize_node_count() {
        let nodes = make_scene();
        let cfg = default_scene_serializer_config();
        let scene = serialize_scene(&nodes, &cfg);
        assert_eq!(scene_node_count(&scene), 3);
    }

    #[test]
    fn test_serialized_to_string_contains_ids() {
        let nodes = make_scene();
        let cfg = default_scene_serializer_config();
        let scene = serialize_scene(&nodes, &cfg);
        let s = serialized_scene_to_string(&scene);
        assert!(s.contains("\"id\":1"));
        assert!(s.contains("\"id\":2"));
        assert!(s.contains("root"));
    }

    #[test]
    fn test_scene_root_nodes() {
        let nodes = make_scene();
        let roots = scene_root_nodes(&nodes);
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&1));
    }

    #[test]
    fn test_find_scene_node() {
        let nodes = make_scene();
        assert!(find_scene_node(&nodes, 2).is_some());
        assert!(find_scene_node(&nodes, 99).is_none());
    }

    #[test]
    fn test_scene_node_depth() {
        let nodes = make_scene();
        assert_eq!(scene_node_depth(&nodes, 1), 0); // root
        assert_eq!(scene_node_depth(&nodes, 2), 1); // child
        assert_eq!(scene_node_depth(&nodes, 3), 2); // leaf
    }

    #[test]
    fn test_deserialize_stub_roundtrip() {
        let nodes = make_scene();
        let cfg = default_scene_serializer_config();
        let scene = serialize_scene(&nodes, &cfg);
        let s = serialized_scene_to_string(&scene);
        let recovered = deserialize_scene_stub(&s);
        // At least all ids should be present
        assert_eq!(recovered.len(), nodes.len());
        assert!(recovered.iter().any(|n| n.id == 1));
        assert!(recovered.iter().any(|n| n.id == 2));
    }

    #[test]
    fn test_deserialize_empty() {
        let nodes = deserialize_scene_stub("[]");
        assert!(nodes.is_empty());
    }
}
