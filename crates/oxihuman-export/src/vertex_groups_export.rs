#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export vertex groups (skin/weight groups).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupsEntry {
    pub name: String,
    pub vertex_weights: Vec<(u32, f32)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct VertexGroupsExport {
    pub groups: Vec<VertexGroupsEntry>,
}

#[allow(dead_code)]
pub fn new_vertex_groups_export() -> VertexGroupsExport {
    VertexGroupsExport { groups: Vec::new() }
}

#[allow(dead_code)]
pub fn add_group_vg(exp: &mut VertexGroupsExport, name: &str) {
    exp.groups.push(VertexGroupsEntry { name: name.to_string(), vertex_weights: Vec::new() });
}

#[allow(dead_code)]
pub fn add_vertex_weight_vg(group: &mut VertexGroupsEntry, vert: u32, weight: f32) {
    group.vertex_weights.push((vert, weight));
}

#[allow(dead_code)]
pub fn export_vertex_groups_to_json(exp: &VertexGroupsExport) -> String {
    let mut groups_json = String::new();
    for (i, g) in exp.groups.iter().enumerate() {
        if i > 0 {
            groups_json.push(',');
        }
        let weights: Vec<String> = g
            .vertex_weights
            .iter()
            .map(|(v, w)| format!(r#"{{"vert":{},"weight":{}}}"#, v, w))
            .collect();
        groups_json.push_str(&format!(
            r#"{{"name":"{}","weights":[{}]}}"#,
            g.name,
            weights.join(",")
        ));
    }
    format!(r#"{{"groups":[{}]}}"#, groups_json)
}

#[allow(dead_code)]
pub fn group_count_vg(exp: &VertexGroupsExport) -> usize {
    exp.groups.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_vertex_groups_export();
        assert_eq!(group_count_vg(&e), 0);
    }

    #[test]
    fn add_group_increases_count() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "arm_L");
        assert_eq!(group_count_vg(&e), 1);
    }

    #[test]
    fn group_name_stored() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "leg_R");
        assert_eq!(e.groups[0].name, "leg_R");
    }

    #[test]
    fn add_vertex_weight_to_group() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "spine");
        add_vertex_weight_vg(&mut e.groups[0], 5, 0.9);
        assert_eq!(e.groups[0].vertex_weights.len(), 1);
    }

    #[test]
    fn export_json_has_groups() {
        let e = new_vertex_groups_export();
        let j = export_vertex_groups_to_json(&e);
        assert!(j.contains("groups"));
    }

    #[test]
    fn export_json_has_group_name() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "head");
        let j = export_vertex_groups_to_json(&e);
        assert!(j.contains("head"));
    }

    #[test]
    fn export_json_has_weight() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "g");
        add_vertex_weight_vg(&mut e.groups[0], 0, 0.5);
        let j = export_vertex_groups_to_json(&e);
        assert!(j.contains("weight"));
    }

    #[test]
    fn multiple_groups() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "a");
        add_group_vg(&mut e, "b");
        add_group_vg(&mut e, "c");
        assert_eq!(group_count_vg(&e), 3);
    }

    #[test]
    fn vertex_weight_vert_id_stored() {
        let mut e = new_vertex_groups_export();
        add_group_vg(&mut e, "g");
        add_vertex_weight_vg(&mut e.groups[0], 42, 1.0);
        assert_eq!(e.groups[0].vertex_weights[0].0, 42);
    }
}
