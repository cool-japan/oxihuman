// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Gltf2Asset {
    pub version: String,
    pub generator: String,
}

pub struct Gltf2Node {
    pub name: String,
    pub mesh_index: Option<usize>,
    pub translation: [f32; 3],
}

pub fn new_gltf2_asset() -> Gltf2Asset {
    Gltf2Asset {
        version: "2.0".to_string(),
        generator: "oxihuman".to_string(),
    }
}

pub fn asset_to_json(a: &Gltf2Asset) -> String {
    format!(
        r#"{{"version":"{}","generator":"{}"}}"#,
        a.version, a.generator
    )
}

pub fn new_gltf2_node(name: &str) -> Gltf2Node {
    Gltf2Node {
        name: name.to_string(),
        mesh_index: None,
        translation: [0.0; 3],
    }
}

pub fn node_to_json(n: &Gltf2Node) -> String {
    let mesh = n
        .mesh_index
        .map(|i| format!(",\"mesh\":{i}"))
        .unwrap_or_default();
    let t = n.translation;
    format!(
        r#"{{"name":"{}"{mesh},"translation":[{},{},{}]}}"#,
        n.name, t[0], t[1], t[2]
    )
}

pub fn nodes_to_json(nodes: &[Gltf2Node]) -> String {
    let inner: Vec<_> = nodes.iter().map(node_to_json).collect();
    format!("[{}]", inner.join(","))
}

pub fn gltf2_scene_json(nodes: &[Gltf2Node], asset: &Gltf2Asset) -> String {
    format!(
        r#"{{"asset":{},"nodes":{}}}"#,
        asset_to_json(asset),
        nodes_to_json(nodes)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_asset_version() {
        /* version is 2.0 */
        let a = new_gltf2_asset();
        assert_eq!(a.version, "2.0");
    }

    #[test]
    fn test_asset_to_json_contains_version() {
        /* json contains version */
        let a = new_gltf2_asset();
        let j = asset_to_json(&a);
        assert!(j.contains("2.0"));
    }

    #[test]
    fn test_new_node_name() {
        /* name is set */
        let n = new_gltf2_node("Root");
        assert_eq!(n.name, "Root");
    }

    #[test]
    fn test_node_to_json_contains_name() {
        /* json contains name */
        let n = new_gltf2_node("Arm");
        let j = node_to_json(&n);
        assert!(j.contains("Arm"));
    }

    #[test]
    fn test_gltf2_scene_json_not_empty() {
        /* scene json not empty */
        let nodes = vec![new_gltf2_node("Root")];
        let asset = new_gltf2_asset();
        let j = gltf2_scene_json(&nodes, &asset);
        assert!(!j.is_empty());
    }
}
