// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeData {
    pub crease: f32,
    pub bevel_weight: f32,
    pub is_seam: bool,
    pub is_sharp: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeDataMap {
    pub data: HashMap<(u32, u32), EdgeData>,
}

#[allow(dead_code)]
pub fn default_edge_data() -> EdgeData {
    EdgeData {
        crease: 0.0,
        bevel_weight: 0.0,
        is_seam: false,
        is_sharp: false,
    }
}

#[allow(dead_code)]
pub fn new_edge_data_map() -> EdgeDataMap {
    EdgeDataMap { data: HashMap::new() }
}

fn canonical(a: u32, b: u32) -> (u32, u32) {
    if a <= b { (a, b) } else { (b, a) }
}

#[allow(dead_code)]
pub fn ed_set(map: &mut EdgeDataMap, a: u32, b: u32, data: EdgeData) {
    map.data.insert(canonical(a, b), data);
}

#[allow(dead_code)]
pub fn ed_get(map: &EdgeDataMap, a: u32, b: u32) -> Option<&EdgeData> {
    map.data.get(&canonical(a, b))
}

#[allow(dead_code)]
pub fn ed_set_crease(map: &mut EdgeDataMap, a: u32, b: u32, v: f32) {
    let key = canonical(a, b);
    let e = map.data.entry(key).or_insert_with(default_edge_data);
    e.crease = v;
}

#[allow(dead_code)]
pub fn ed_set_seam(map: &mut EdgeDataMap, a: u32, b: u32, seam: bool) {
    let key = canonical(a, b);
    let e = map.data.entry(key).or_insert_with(default_edge_data);
    e.is_seam = seam;
}

#[allow(dead_code)]
pub fn ed_edge_count(map: &EdgeDataMap) -> usize {
    map.data.len()
}

#[allow(dead_code)]
pub fn ed_seam_count(map: &EdgeDataMap) -> usize {
    map.data.values().filter(|e| e.is_seam).count()
}

#[allow(dead_code)]
pub fn ed_to_json(map: &EdgeDataMap) -> String {
    format!(
        r#"{{"edge_count":{},"seam_count":{}}}"#,
        map.data.len(),
        ed_seam_count(map)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_edge_data() {
        let e = default_edge_data();
        assert!((e.crease).abs() < 1e-6);
        assert!(!e.is_seam);
    }

    #[test]
    fn test_new_map_empty() {
        let m = new_edge_data_map();
        assert_eq!(ed_edge_count(&m), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut m = new_edge_data_map();
        let mut d = default_edge_data();
        d.crease = 0.5;
        ed_set(&mut m, 0, 1, d);
        let g = ed_get(&m, 0, 1).expect("should succeed");
        assert!((g.crease - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_canonical_order() {
        let mut m = new_edge_data_map();
        let mut d = default_edge_data();
        d.is_sharp = true;
        ed_set(&mut m, 5, 2, d);
        let g = ed_get(&m, 2, 5).expect("should succeed");
        assert!(g.is_sharp);
    }

    #[test]
    fn test_set_crease() {
        let mut m = new_edge_data_map();
        ed_set_crease(&mut m, 1, 2, 0.8);
        let g = ed_get(&m, 1, 2).expect("should succeed");
        assert!((g.crease - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_set_seam() {
        let mut m = new_edge_data_map();
        ed_set_seam(&mut m, 3, 4, true);
        assert_eq!(ed_seam_count(&m), 1);
    }

    #[test]
    fn test_edge_count() {
        let mut m = new_edge_data_map();
        ed_set_crease(&mut m, 0, 1, 1.0);
        ed_set_crease(&mut m, 1, 2, 0.5);
        assert_eq!(ed_edge_count(&m), 2);
    }

    #[test]
    fn test_to_json() {
        let mut m = new_edge_data_map();
        ed_set_seam(&mut m, 0, 1, true);
        let j = ed_to_json(&m);
        assert!(j.contains("seam_count"));
        assert!(j.contains("edge_count"));
    }
}
