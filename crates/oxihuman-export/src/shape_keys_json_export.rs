#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export shape key (morph target) data to JSON.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKeyJsonEntry {
    pub name: String,
    pub value: f32,
    pub vertex_offsets: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeKeysJsonExport {
    pub base_name: String,
    pub keys: Vec<ShapeKeyJsonEntry>,
}

#[allow(dead_code)]
pub fn new_shape_keys_json_export(base: &str) -> ShapeKeysJsonExport {
    ShapeKeysJsonExport { base_name: base.to_string(), keys: Vec::new() }
}

#[allow(dead_code)]
pub fn add_shape_key_json(
    exp: &mut ShapeKeysJsonExport,
    name: &str,
    val: f32,
    offsets: Vec<[f32; 3]>,
) {
    exp.keys.push(ShapeKeyJsonEntry { name: name.to_string(), value: val, vertex_offsets: offsets });
}

#[allow(dead_code)]
pub fn export_shape_keys_to_json(exp: &ShapeKeysJsonExport) -> String {
    let mut keys_json = String::new();
    for (i, k) in exp.keys.iter().enumerate() {
        if i > 0 {
            keys_json.push(',');
        }
        let offsets: Vec<String> = k
            .vertex_offsets
            .iter()
            .map(|o| format!("[{},{},{}]", o[0], o[1], o[2]))
            .collect();
        keys_json.push_str(&format!(
            r#"{{"name":"{}","value":{},"offsets":[{}]}}"#,
            k.name,
            k.value,
            offsets.join(",")
        ));
    }
    format!(
        r#"{{"base":"{}","keys":[{}]}}"#,
        exp.base_name, keys_json
    )
}

#[allow(dead_code)]
pub fn key_count_json(exp: &ShapeKeysJsonExport) -> usize {
    exp.keys.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_shape_keys_json_export("base");
        assert_eq!(e.base_name, "base");
        assert_eq!(key_count_json(&e), 0);
    }

    #[test]
    fn add_key_increases_count() {
        let mut e = new_shape_keys_json_export("mesh");
        add_shape_key_json(&mut e, "smile", 0.5, vec![[0.1, 0.0, 0.0]]);
        assert_eq!(key_count_json(&e), 1);
    }

    #[test]
    fn key_fields_stored() {
        let mut e = new_shape_keys_json_export("mesh");
        add_shape_key_json(&mut e, "brow_up", 0.8, vec![[0.0, 0.1, 0.0]]);
        assert_eq!(e.keys[0].name, "brow_up");
        assert!((e.keys[0].value - 0.8).abs() < 1e-6);
    }

    #[test]
    fn export_json_contains_base() {
        let e = new_shape_keys_json_export("my_mesh");
        let j = export_shape_keys_to_json(&e);
        assert!(j.contains("my_mesh"));
    }

    #[test]
    fn export_json_contains_key_name() {
        let mut e = new_shape_keys_json_export("m");
        add_shape_key_json(&mut e, "jaw_open", 1.0, vec![]);
        let j = export_shape_keys_to_json(&e);
        assert!(j.contains("jaw_open"));
    }

    #[test]
    fn export_json_contains_offsets() {
        let mut e = new_shape_keys_json_export("m");
        add_shape_key_json(&mut e, "k", 0.0, vec![[1.0, 2.0, 3.0]]);
        let j = export_shape_keys_to_json(&e);
        assert!(j.contains("offsets"));
    }

    #[test]
    fn multiple_keys() {
        let mut e = new_shape_keys_json_export("m");
        for i in 0..4 {
            add_shape_key_json(&mut e, &format!("k{}", i), 0.0, vec![]);
        }
        assert_eq!(key_count_json(&e), 4);
    }

    #[test]
    fn empty_offsets_ok() {
        let mut e = new_shape_keys_json_export("m");
        add_shape_key_json(&mut e, "k", 0.5, vec![]);
        let j = export_shape_keys_to_json(&e);
        assert!(j.contains("k"));
    }
}
