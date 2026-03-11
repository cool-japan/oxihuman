#![allow(dead_code)]

//! Vertex group assignment and weight management.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupAssign {
    pub name: String,
    pub members: HashMap<usize, f32>,
}

#[allow(dead_code)]
pub fn assign_to_group(group: &mut VertexGroupAssign, vertex: usize, weight: f32) {
    group.members.insert(vertex, weight.clamp(0.0, 1.0));
}

#[allow(dead_code)]
pub fn remove_from_group(group: &mut VertexGroupAssign, vertex: usize) {
    group.members.remove(&vertex);
}

#[allow(dead_code)]
pub fn group_members(group: &VertexGroupAssign) -> Vec<usize> {
    let mut keys: Vec<usize> = group.members.keys().copied().collect();
    keys.sort_unstable();
    keys
}

#[allow(dead_code)]
pub fn group_weight_at(group: &VertexGroupAssign, vertex: usize) -> f32 {
    group.members.get(&vertex).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn group_count_vga(group: &VertexGroupAssign) -> usize {
    group.members.len()
}

#[allow(dead_code)]
pub fn group_name_vga(group: &VertexGroupAssign) -> &str {
    &group.name
}

#[allow(dead_code)]
pub fn normalize_group_weights(group: &mut VertexGroupAssign) {
    let max_w = group.members.values().copied().fold(0.0f32, f32::max);
    if max_w > 1e-12 {
        for w in group.members.values_mut() {
            *w /= max_w;
        }
    }
}

#[allow(dead_code)]
pub fn group_to_json(group: &VertexGroupAssign) -> String {
    let members: Vec<String> = group.members.iter()
        .map(|(k, v)| format!("{{\"vertex\":{},\"weight\":{:.4}}}", k, v))
        .collect();
    format!("{{\"name\":\"{}\",\"count\":{},\"members\":[{}]}}", group.name, group.members.len(), members.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grp() -> VertexGroupAssign {
        VertexGroupAssign { name: "arm".to_string(), members: HashMap::new() }
    }

    #[test]
    fn test_assign() { let mut g = grp(); assign_to_group(&mut g, 0, 0.5); assert_eq!(group_count_vga(&g), 1); }
    #[test]
    fn test_remove() { let mut g = grp(); assign_to_group(&mut g, 0, 0.5); remove_from_group(&mut g, 0); assert_eq!(group_count_vga(&g), 0); }
    #[test]
    fn test_members() { let mut g = grp(); assign_to_group(&mut g, 2, 1.0); assign_to_group(&mut g, 0, 1.0); assert_eq!(group_members(&g), vec![0, 2]); }
    #[test]
    fn test_weight_at() { let mut g = grp(); assign_to_group(&mut g, 0, 0.7); assert!((group_weight_at(&g, 0) - 0.7).abs() < 1e-6); }
    #[test]
    fn test_weight_missing() { let g = grp(); assert!((group_weight_at(&g, 0)).abs() < 1e-6); }
    #[test]
    fn test_name() { let g = grp(); assert_eq!(group_name_vga(&g), "arm"); }
    #[test]
    fn test_normalize() { let mut g = grp(); assign_to_group(&mut g, 0, 0.5); normalize_group_weights(&mut g); assert!((group_weight_at(&g, 0) - 1.0).abs() < 1e-6); }
    #[test]
    fn test_to_json() { let g = grp(); assert!(group_to_json(&g).contains("\"name\":\"arm\"")); }
    #[test]
    fn test_clamp() { let mut g = grp(); assign_to_group(&mut g, 0, 2.0); assert!((group_weight_at(&g, 0) - 1.0).abs() < 1e-6); }
    #[test]
    fn test_reassign() { let mut g = grp(); assign_to_group(&mut g, 0, 0.5); assign_to_group(&mut g, 0, 0.9); assert!((group_weight_at(&g, 0) - 0.9).abs() < 1e-6); }
}
