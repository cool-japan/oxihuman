// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge crease set: manage crease values for subdivision control.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeCreaseSet {
    creases: HashMap<(u32,u32), f32>,
}

#[allow(dead_code)]
pub fn new_edge_crease_set() -> EdgeCreaseSet { EdgeCreaseSet { creases: HashMap::new() } }

#[allow(dead_code)]
pub fn edge_key(a: u32, b: u32) -> (u32, u32) { if a < b { (a, b) } else { (b, a) } }

#[allow(dead_code)]
pub fn set_crease_value(set: &mut EdgeCreaseSet, a: u32, b: u32, value: f32) {
    set.creases.insert(edge_key(a, b), value.clamp(0.0, 1.0));
}

#[allow(dead_code)]
pub fn get_crease_value(set: &EdgeCreaseSet, a: u32, b: u32) -> f32 {
    *set.creases.get(&edge_key(a, b)).unwrap_or(&0.0)
}

#[allow(dead_code)]
pub fn remove_crease_value(set: &mut EdgeCreaseSet, a: u32, b: u32) {
    set.creases.remove(&edge_key(a, b));
}

#[allow(dead_code)]
pub fn crease_count_ecs(set: &EdgeCreaseSet) -> usize { set.creases.len() }

#[allow(dead_code)]
pub fn all_creased_edges_ecs(set: &EdgeCreaseSet) -> Vec<((u32,u32), f32)> {
    set.creases.iter().map(|(&k,&v)| (k,v)).collect()
}

#[allow(dead_code)]
pub fn clear_crease_set(set: &mut EdgeCreaseSet) { set.creases.clear(); }

#[allow(dead_code)]
pub fn max_crease_value(set: &EdgeCreaseSet) -> f32 {
    set.creases.values().copied().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn crease_set_to_json(set: &EdgeCreaseSet) -> String {
    format!("{{\"count\":{},\"max\":{:.4}}}", set.creases.len(), max_crease_value(set))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let s=new_edge_crease_set(); assert_eq!(crease_count_ecs(&s),0); }
    #[test] fn test_set_get() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,0.5); assert!((get_crease_value(&s,0,1)-0.5).abs()<1e-6); }
    #[test] fn test_symmetric_key() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,1,0,0.7); assert!((get_crease_value(&s,0,1)-0.7).abs()<1e-6); }
    #[test] fn test_remove() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,0.5); remove_crease_value(&mut s,0,1); assert_eq!(crease_count_ecs(&s),0); }
    #[test] fn test_clamp() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,2.0); assert!((get_crease_value(&s,0,1)-1.0).abs()<1e-6); }
    #[test] fn test_all_creased() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,0.5); set_crease_value(&mut s,1,2,0.3); let all=all_creased_edges_ecs(&s); assert_eq!(all.len(),2); }
    #[test] fn test_clear() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,0.5); clear_crease_set(&mut s); assert_eq!(crease_count_ecs(&s),0); }
    #[test] fn test_max_crease() { let mut s=new_edge_crease_set(); set_crease_value(&mut s,0,1,0.3); set_crease_value(&mut s,2,3,0.8); assert!((max_crease_value(&s)-0.8).abs()<1e-6); }
    #[test] fn test_to_json() { let s=new_edge_crease_set(); assert!(crease_set_to_json(&s).contains("count")); }
    #[test] fn test_get_missing() { let s=new_edge_crease_set(); assert!((get_crease_value(&s,99,100)).abs()<1e-6); }
}
