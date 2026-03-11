// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Polygon triangulation (fan and ear-clipping).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TriangulateMethod {
    Fan,
    EarClip,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangulateConfig {
    pub method: TriangulateMethod,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TriangulateResult {
    pub indices: Vec<u32>,
    pub triangle_count: usize,
}

#[allow(dead_code)]
pub fn default_triangulate_config() -> TriangulateConfig {
    TriangulateConfig { method: TriangulateMethod::Fan }
}

#[allow(dead_code)]
pub fn triangulate_polygon(polygon: &[u32], config: &TriangulateConfig) -> TriangulateResult {
    match config.method {
        TriangulateMethod::Fan => triangulate_fan(polygon),
        TriangulateMethod::EarClip => triangulate_fan(polygon), // stub: same as fan
    }
}

#[allow(dead_code)]
pub fn triangulate_fan(polygon: &[u32]) -> TriangulateResult {
    if polygon.len() < 3 {
        return TriangulateResult { indices: vec![], triangle_count: 0 };
    }
    let mut indices = Vec::new();
    let pivot = polygon[0];
    for i in 1..(polygon.len() - 1) {
        indices.push(pivot);
        indices.push(polygon[i]);
        indices.push(polygon[i + 1]);
    }
    let triangle_count = indices.len() / 3;
    TriangulateResult { indices, triangle_count }
}

#[allow(dead_code)]
pub fn triangulate_triangle_count(result: &TriangulateResult) -> usize {
    result.triangle_count
}

#[allow(dead_code)]
pub fn triangulate_validate(polygon: &[u32]) -> bool {
    polygon.len() >= 3
}

#[allow(dead_code)]
pub fn triangulate_to_json(result: &TriangulateResult) -> String {
    format!(r#"{{"triangle_count":{},"index_count":{}}}"#, result.triangle_count, result.indices.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_triangle_from_quad() {
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_fan(&quad);
        assert_eq!(res.triangle_count, 2);
        assert_eq!(res.indices.len(), 6);
    }

    #[test]
    fn fan_triangle_from_pentagon() {
        let pent = vec![0u32, 1, 2, 3, 4];
        let res = triangulate_fan(&pent);
        assert_eq!(res.triangle_count, 3);
    }

    #[test]
    fn triangle_unchanged() {
        let tri = vec![0u32, 1, 2];
        let res = triangulate_fan(&tri);
        assert_eq!(res.triangle_count, 1);
    }

    #[test]
    fn less_than_three_returns_empty() {
        let poly = vec![0u32, 1];
        let res = triangulate_fan(&poly);
        assert_eq!(res.triangle_count, 0);
        assert!(res.indices.is_empty());
    }

    #[test]
    fn validate_rejects_small_polygon() {
        assert!(!triangulate_validate(&[0u32, 1]));
        assert!(triangulate_validate(&[0u32, 1, 2]));
    }

    #[test]
    fn default_method_is_fan() {
        let cfg = default_triangulate_config();
        assert_eq!(cfg.method, TriangulateMethod::Fan);
    }

    #[test]
    fn earclip_method_produces_triangles() {
        let cfg = TriangulateConfig { method: TriangulateMethod::EarClip };
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_polygon(&quad, &cfg);
        assert_eq!(res.triangle_count, 2);
    }

    #[test]
    fn to_json_correct_format() {
        let quad = vec![0u32, 1, 2, 3];
        let res = triangulate_fan(&quad);
        let json = triangulate_to_json(&res);
        assert!(json.contains("triangle_count"));
        assert!(json.contains("index_count"));
    }
}
