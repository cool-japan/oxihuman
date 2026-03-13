// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Named vertex group management with per-vertex weights.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupEntry {
    pub vertex_index: u32,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupV2 {
    pub name: String,
    pub entries: Vec<VertexGroupEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupSetV2 {
    pub groups: Vec<VertexGroupV2>,
}

#[allow(dead_code)]
pub fn new_vertex_group_set() -> VertexGroupSetV2 {
    VertexGroupSetV2 { groups: Vec::new() }
}

#[allow(dead_code)]
pub fn add_group(set: &mut VertexGroupSetV2, name: &str) -> usize {
    let idx = set.groups.len();
    set.groups.push(VertexGroupV2 {
        name: name.to_string(),
        entries: Vec::new(),
    });
    idx
}

#[allow(dead_code)]
pub fn group_count(set: &VertexGroupSetV2) -> usize {
    set.groups.len()
}

#[allow(dead_code)]
pub fn add_to_group(set: &mut VertexGroupSetV2, group: usize, vertex: u32, weight: f32) {
    if group < set.groups.len() {
        set.groups[group].entries.push(VertexGroupEntry {
            vertex_index: vertex,
            weight,
        });
    }
}

#[allow(dead_code)]
pub fn find_group_by_name<'a>(set: &'a VertexGroupSetV2, name: &str) -> Option<&'a VertexGroupV2> {
    set.groups.iter().find(|g| g.name == name)
}

#[allow(dead_code)]
pub fn group_vertex_count(group: &VertexGroupV2) -> usize {
    group.entries.len()
}

#[allow(dead_code)]
pub fn group_average_weight(group: &VertexGroupV2) -> f32 {
    if group.entries.is_empty() {
        return 0.0;
    }
    let sum: f32 = group.entries.iter().map(|e| e.weight).sum();
    sum / group.entries.len() as f32
}

#[allow(dead_code)]
pub fn weight_in_group(group: &VertexGroupV2, vertex: u32) -> Option<f32> {
    group
        .entries
        .iter()
        .find(|e| e.vertex_index == vertex)
        .map(|e| e.weight)
}

#[allow(dead_code)]
pub fn remove_from_group(set: &mut VertexGroupSetV2, group: usize, vertex: u32) {
    if group < set.groups.len() {
        set.groups[group]
            .entries
            .retain(|e| e.vertex_index != vertex);
    }
}

#[allow(dead_code)]
pub fn group_set_to_json(set: &VertexGroupSetV2) -> String {
    format!("{{\"group_count\":{}}}", group_count(set))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_set() {
        let set = new_vertex_group_set();
        assert_eq!(group_count(&set), 0);
    }

    #[test]
    fn test_add_group() {
        let mut set = new_vertex_group_set();
        add_group(&mut set, "arm");
        assert_eq!(group_count(&set), 1);
    }

    #[test]
    fn test_add_vertex_to_group() {
        let mut set = new_vertex_group_set();
        let g = add_group(&mut set, "leg");
        add_to_group(&mut set, g, 5, 0.8);
        assert_eq!(group_vertex_count(&set.groups[g]), 1);
    }

    #[test]
    fn test_find_by_name() {
        let mut set = new_vertex_group_set();
        add_group(&mut set, "spine");
        assert!(find_group_by_name(&set, "spine").is_some());
    }

    #[test]
    fn test_find_missing() {
        let set = new_vertex_group_set();
        assert!(find_group_by_name(&set, "missing").is_none());
    }

    #[test]
    fn test_average_weight() {
        let mut set = new_vertex_group_set();
        let g = add_group(&mut set, "skin");
        add_to_group(&mut set, g, 0, 1.0);
        add_to_group(&mut set, g, 1, 0.0);
        let avg = group_average_weight(&set.groups[g]);
        assert!((avg - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_weight_in_group() {
        let mut set = new_vertex_group_set();
        let g = add_group(&mut set, "head");
        add_to_group(&mut set, g, 3, 0.6);
        let w = weight_in_group(&set.groups[g], 3);
        assert!(w.is_some_and(|v| (v - 0.6).abs() < 1e-5));
    }

    #[test]
    fn test_remove_from_group() {
        let mut set = new_vertex_group_set();
        let g = add_group(&mut set, "foot");
        add_to_group(&mut set, g, 7, 1.0);
        remove_from_group(&mut set, g, 7);
        assert_eq!(group_vertex_count(&set.groups[g]), 0);
    }

    #[test]
    fn test_json_output() {
        let set = new_vertex_group_set();
        let j = group_set_to_json(&set);
        assert!(j.contains("group_count"));
    }

    #[test]
    fn test_multiple_groups() {
        let mut set = new_vertex_group_set();
        add_group(&mut set, "a");
        add_group(&mut set, "b");
        add_group(&mut set, "c");
        assert_eq!(group_count(&set), 3);
    }

    #[test]
    fn test_empty_group_average() {
        let group = VertexGroupV2 {
            name: "empty".to_string(),
            entries: Vec::new(),
        };
        assert!((group_average_weight(&group)).abs() < 1e-6);
    }
}
