// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A single input socket for a geometry nodes modifier.
#[allow(dead_code)]
pub struct GeoNodeInput {
    pub name: String,
    pub input_type: String,
    pub value: f32,
}

/// A geometry nodes modifier instance.
#[allow(dead_code)]
pub struct GeoNodeModifierExport {
    pub name: String,
    pub node_group: String,
    pub inputs: Vec<GeoNodeInput>,
}

/// Create a new geometry node modifier.
#[allow(dead_code)]
pub fn new_geo_node_modifier_export(name: &str, group: &str) -> GeoNodeModifierExport {
    GeoNodeModifierExport {
        name: name.to_string(),
        node_group: group.to_string(),
        inputs: Vec::new(),
    }
}

/// Add an input to the modifier.
#[allow(dead_code)]
pub fn add_input_export(m: &mut GeoNodeModifierExport, name: &str, type_: &str, val: f32) {
    m.inputs.push(GeoNodeInput {
        name: name.to_string(),
        input_type: type_.to_string(),
        value: val,
    });
}

/// Export the modifier to a JSON string.
#[allow(dead_code)]
pub fn export_geo_nodes_to_json_export(m: &GeoNodeModifierExport) -> String {
    let inputs_json: String = m
        .inputs
        .iter()
        .map(|i| {
            format!(
                r#"{{"name":"{}","type":"{}","value":{}}}"#,
                i.name, i.input_type, i.value
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"name":"{}","node_group":"{}","inputs":[{}]}}"#,
        m.name, m.node_group, inputs_json
    )
}

/// Count inputs.
#[allow(dead_code)]
pub fn geo_node_input_count(m: &GeoNodeModifierExport) -> usize {
    m.inputs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_geo_node_modifier_export() {
        let m = new_geo_node_modifier_export("GeometryNodes", "MyGroup");
        assert_eq!(m.name, "GeometryNodes");
        assert_eq!(m.node_group, "MyGroup");
    }

    #[test]
    fn test_add_input_export_count() {
        let mut m = new_geo_node_modifier_export("Mod", "Grp");
        add_input_export(&mut m, "Density", "FLOAT", 0.5);
        add_input_export(&mut m, "Radius", "FLOAT", 1.0);
        assert_eq!(m.inputs.len(), 2);
    }

    #[test]
    fn test_add_input_export_value() {
        let mut m = new_geo_node_modifier_export("Mod", "Grp");
        add_input_export(&mut m, "Scale", "FLOAT", 2.5);
        assert!((m.inputs[0].value - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_export_geo_nodes_to_json_export_contains_name() {
        let m = new_geo_node_modifier_export("ScatterMod", "GeoGroup");
        let json = export_geo_nodes_to_json_export(&m);
        assert!(json.contains("ScatterMod"));
    }

    #[test]
    fn test_export_geo_nodes_to_json_export_contains_group() {
        let m = new_geo_node_modifier_export("Mod", "NodeTree01");
        let json = export_geo_nodes_to_json_export(&m);
        assert!(json.contains("NodeTree01"));
    }

    #[test]
    fn test_export_geo_nodes_to_json_export_with_inputs() {
        let mut m = new_geo_node_modifier_export("Mod", "Grp");
        add_input_export(&mut m, "Seed", "INT", 42.0);
        let json = export_geo_nodes_to_json_export(&m);
        assert!(json.contains("Seed"));
    }

    #[test]
    fn test_geo_node_input_count_empty() {
        let m = new_geo_node_modifier_export("M", "G");
        assert_eq!(geo_node_input_count(&m), 0);
    }

    #[test]
    fn test_geo_node_input_count_after_add() {
        let mut m = new_geo_node_modifier_export("M", "G");
        add_input_export(&mut m, "A", "FLOAT", 1.0);
        add_input_export(&mut m, "B", "FLOAT", 2.0);
        add_input_export(&mut m, "C", "FLOAT", 3.0);
        assert_eq!(geo_node_input_count(&m), 3);
    }

    #[test]
    fn test_export_json_is_valid_structure() {
        let mut m = new_geo_node_modifier_export("Test", "TestGrp");
        add_input_export(&mut m, "Value", "FLOAT", 0.1);
        let json = export_geo_nodes_to_json_export(&m);
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }
}
