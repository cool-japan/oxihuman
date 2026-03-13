// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinWeight {
    pub bone_index: u32,
    pub weight: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexSkinData {
    pub weights: Vec<SkinWeight>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinWeightMap {
    pub vertices: Vec<VertexSkinData>,
}

#[allow(dead_code)]
pub fn new_skin_weight_map() -> SkinWeightMap {
    SkinWeightMap { vertices: Vec::new() }
}

#[allow(dead_code)]
pub fn sw_add_vertex(map: &mut SkinWeightMap) {
    map.vertices.push(VertexSkinData { weights: Vec::new() });
}

#[allow(dead_code)]
pub fn sw_set_weight(map: &mut SkinWeightMap, vert: usize, bone: u32, weight: f32) {
    if let Some(v) = map.vertices.get_mut(vert) {
        if let Some(existing) = v.weights.iter_mut().find(|w| w.bone_index == bone) {
            existing.weight = weight;
        } else {
            v.weights.push(SkinWeight { bone_index: bone, weight });
        }
    }
}

#[allow(dead_code)]
pub fn sw_get_weights(map: &SkinWeightMap, vert: usize) -> &[SkinWeight] {
    map.vertices.get(vert).map(|v| v.weights.as_slice()).unwrap_or(&[])
}

#[allow(dead_code)]
pub fn sw_normalize(map: &mut SkinWeightMap, vert: usize) {
    if let Some(v) = map.vertices.get_mut(vert) {
        let total: f32 = v.weights.iter().map(|w| w.weight).sum();
        if total > 1e-12 {
            for w in &mut v.weights {
                w.weight /= total;
            }
        }
    }
}

#[allow(dead_code)]
pub fn sw_normalize_all(map: &mut SkinWeightMap) {
    let len = map.vertices.len();
    for i in 0..len {
        sw_normalize(map, i);
    }
}

#[allow(dead_code)]
pub fn sw_vertex_count(map: &SkinWeightMap) -> usize {
    map.vertices.len()
}

#[allow(dead_code)]
pub fn sw_to_json(map: &SkinWeightMap) -> String {
    format!(r#"{{"vertex_count":{}}}"#, map.vertices.len())
}

#[allow(dead_code)]
pub fn sw_max_bones_per_vertex(map: &SkinWeightMap) -> usize {
    map.vertices.iter().map(|v| v.weights.len()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map_empty() {
        let m = new_skin_weight_map();
        assert_eq!(sw_vertex_count(&m), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_add_vertex(&mut m);
        assert_eq!(sw_vertex_count(&m), 2);
    }

    #[test]
    fn test_set_weight() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_set_weight(&mut m, 0, 0, 0.8);
        sw_set_weight(&mut m, 0, 1, 0.2);
        let ws = sw_get_weights(&m, 0);
        assert_eq!(ws.len(), 2);
    }

    #[test]
    fn test_update_weight() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_set_weight(&mut m, 0, 0, 0.5);
        sw_set_weight(&mut m, 0, 0, 0.9);
        let ws = sw_get_weights(&m, 0);
        assert_eq!(ws.len(), 1);
        assert!((ws[0].weight - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_normalize() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_set_weight(&mut m, 0, 0, 2.0);
        sw_set_weight(&mut m, 0, 1, 2.0);
        sw_normalize(&mut m, 0);
        let ws = sw_get_weights(&m, 0);
        let total: f32 = ws.iter().map(|w| w.weight).sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_all() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_add_vertex(&mut m);
        sw_set_weight(&mut m, 0, 0, 4.0);
        sw_set_weight(&mut m, 1, 0, 3.0);
        sw_normalize_all(&mut m);
        let ws0 = sw_get_weights(&m, 0);
        assert!((ws0[0].weight - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_bones() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        sw_add_vertex(&mut m);
        sw_set_weight(&mut m, 0, 0, 1.0);
        sw_set_weight(&mut m, 1, 0, 0.5);
        sw_set_weight(&mut m, 1, 1, 0.5);
        assert_eq!(sw_max_bones_per_vertex(&m), 2);
    }

    #[test]
    fn test_to_json() {
        let mut m = new_skin_weight_map();
        sw_add_vertex(&mut m);
        let j = sw_to_json(&m);
        assert!(j.contains("vertex_count"));
    }
}
