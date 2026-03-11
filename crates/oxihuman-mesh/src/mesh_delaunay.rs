// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 2D Delaunay triangulation using Bowyer-Watson algorithm.

// ── Structs ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DelaunayConfig {
    pub epsilon: f32,
    pub super_triangle_scale: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DelaunayPoint {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DelaunayTriangle {
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DelaunayResult {
    pub triangles: Vec<DelaunayTriangle>,
    pub point_count: usize,
}

// ── Functions ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn default_delaunay_config() -> DelaunayConfig {
    DelaunayConfig {
        epsilon: 1e-6,
        super_triangle_scale: 100.0,
    }
}

#[allow(dead_code)]
pub fn point_in_circumcircle(
    p: &DelaunayPoint,
    a: &DelaunayPoint,
    b: &DelaunayPoint,
    c: &DelaunayPoint,
) -> bool {
    let cc = triangle_circumcenter(a, b, c);
    let r2 = (a.x - cc.x).powi(2) + (a.y - cc.y).powi(2);
    let d2 = (p.x - cc.x).powi(2) + (p.y - cc.y).powi(2);
    d2 < r2
}

#[allow(dead_code)]
pub fn triangle_circumcenter(
    a: &DelaunayPoint,
    b: &DelaunayPoint,
    c: &DelaunayPoint,
) -> DelaunayPoint {
    let ax = a.x;
    let ay = a.y;
    let bx = b.x;
    let by = b.y;
    let cx = c.x;
    let cy = c.y;

    let d = 2.0 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
    if d.abs() < 1e-10 {
        return DelaunayPoint {
            x: (ax + bx + cx) / 3.0,
            y: (ay + by + cy) / 3.0,
        };
    }
    let ux = ((ax * ax + ay * ay) * (by - cy)
        + (bx * bx + by * by) * (cy - ay)
        + (cx * cx + cy * cy) * (ay - by))
        / d;
    let uy = ((ax * ax + ay * ay) * (cx - bx)
        + (bx * bx + by * by) * (ax - cx)
        + (cx * cx + cy * cy) * (bx - ax))
        / d;
    DelaunayPoint { x: ux, y: uy }
}

#[allow(dead_code)]
pub fn triangle_area_2d(a: &DelaunayPoint, b: &DelaunayPoint, c: &DelaunayPoint) -> f32 {
    0.5 * ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs()
}

#[allow(dead_code)]
pub fn is_ccw(a: &DelaunayPoint, b: &DelaunayPoint, c: &DelaunayPoint) -> bool {
    (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y) > 0.0
}

#[allow(dead_code)]
pub fn delaunay_triangle_to_json(t: &DelaunayTriangle) -> String {
    format!("{{\"a\":{},\"b\":{},\"c\":{}}}", t.a, t.b, t.c)
}

#[allow(dead_code)]
pub fn delaunay_result_to_json(r: &DelaunayResult) -> String {
    let tris_json: Vec<String> = r.triangles.iter().map(delaunay_triangle_to_json).collect();
    format!(
        "{{\"point_count\":{},\"triangles\":[{}]}}",
        r.point_count,
        tris_json.join(",")
    )
}

#[allow(dead_code)]
pub fn point_count(r: &DelaunayResult) -> usize {
    r.point_count
}

#[allow(dead_code)]
pub fn triangle_count_delaunay(r: &DelaunayResult) -> usize {
    r.triangles.len()
}

/// Bowyer-Watson Delaunay triangulation.
///
/// Super-triangle vertices are prepended; they are stripped from the result.
#[allow(dead_code)]
pub fn triangulate(points: &[DelaunayPoint], cfg: &DelaunayConfig) -> DelaunayResult {
    if points.is_empty() {
        return DelaunayResult {
            triangles: vec![],
            point_count: 0,
        };
    }

    let scale = cfg.super_triangle_scale;

    // Compute bounding box
    let (mut xmin, mut xmax) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut ymin, mut ymax) = (f32::INFINITY, f32::NEG_INFINITY);
    for p in points {
        xmin = xmin.min(p.x);
        xmax = xmax.max(p.x);
        ymin = ymin.min(p.y);
        ymax = ymax.max(p.y);
    }
    let cx = (xmin + xmax) * 0.5;
    let cy = (ymin + ymax) * 0.5;
    let dx = (xmax - xmin + 1.0) * scale;
    let dy = (ymax - ymin + 1.0) * scale;

    // Super-triangle at indices n, n+1, n+2
    let n = points.len();
    let mut all_points: Vec<DelaunayPoint> = points.to_vec();
    all_points.push(DelaunayPoint {
        x: cx - dx,
        y: cy - dy,
    });
    all_points.push(DelaunayPoint {
        x: cx,
        y: cy + dy,
    });
    all_points.push(DelaunayPoint {
        x: cx + dx,
        y: cy - dy,
    });

    // Indices of super-triangle
    let s0 = n as u32;
    let s1 = (n + 1) as u32;
    let s2 = (n + 2) as u32;

    // Start with super-triangle
    let mut triangles: Vec<(u32, u32, u32)> = vec![(s0, s1, s2)];

    for (pi, pt) in points.iter().enumerate() {
        let pi = pi as u32;
        // Find all triangles whose circumcircle contains pt
        let mut bad_triangles: Vec<(u32, u32, u32)> = Vec::new();
        for &(ta, tb, tc) in &triangles {
            let pa = &all_points[ta as usize];
            let pb = &all_points[tb as usize];
            let pc = &all_points[tc as usize];
            if point_in_circumcircle(pt, pa, pb, pc) {
                bad_triangles.push((ta, tb, tc));
            }
        }

        // Find the boundary polygon of bad triangles
        let mut polygon: Vec<(u32, u32)> = Vec::new();
        for &(ta, tb, tc) in &bad_triangles {
            let edges = [(ta, tb), (tb, tc), (tc, ta)];
            for &(ea, eb) in &edges {
                let shared = bad_triangles.iter().any(|&(oa, ob, oc)| {
                    let o_edges = [(oa, ob), (ob, oc), (oc, oa)];
                    o_edges.iter().any(|&(oe1, oe2)| {
                        (oe1 == ea && oe2 == eb) || (oe1 == eb && oe2 == ea)
                    })
                });
                // Edge is on boundary if it is NOT shared with another bad triangle
                // (i.e., appears exactly once across all bad triangles)
                let count = bad_triangles.iter().flat_map(|&(oa, ob, oc)| {
                    let oe = [(oa, ob), (ob, oc), (oc, oa)];
                    oe.into_iter()
                }).filter(|&(oe1, oe2)| {
                    (oe1 == ea && oe2 == eb) || (oe1 == eb && oe2 == ea)
                }).count();
                let _ = shared;
                if count == 1 {
                    polygon.push((ea, eb));
                }
            }
        }

        // Remove bad triangles
        triangles.retain(|t| !bad_triangles.contains(t));

        // Add new triangles from boundary polygon
        for (ea, eb) in polygon {
            triangles.push((ea, eb, pi));
        }
    }

    // Remove any triangle that shares a vertex with the super-triangle
    triangles.retain(|&(ta, tb, tc)| {
        ta < n as u32 && tb < n as u32 && tc < n as u32
    });

    let result_triangles: Vec<DelaunayTriangle> = triangles
        .into_iter()
        .map(|(a, b, c)| DelaunayTriangle { a, b, c })
        .collect();

    DelaunayResult {
        point_count: points.len(),
        triangles: result_triangles,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn square_points() -> Vec<DelaunayPoint> {
        vec![
            DelaunayPoint { x: 0.0, y: 0.0 },
            DelaunayPoint { x: 1.0, y: 0.0 },
            DelaunayPoint { x: 1.0, y: 1.0 },
            DelaunayPoint { x: 0.0, y: 1.0 },
        ]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_delaunay_config();
        assert!(cfg.epsilon > 0.0);
        assert!(cfg.super_triangle_scale > 1.0);
    }

    #[test]
    fn test_triangulate_square() {
        let pts = square_points();
        let cfg = default_delaunay_config();
        let result = triangulate(&pts, &cfg);
        assert_eq!(result.point_count, 4);
        // A square must triangulate to exactly 2 triangles
        assert_eq!(result.triangles.len(), 2);
    }

    #[test]
    fn test_point_in_circumcircle() {
        let a = DelaunayPoint { x: 0.0, y: 0.0 };
        let b = DelaunayPoint { x: 1.0, y: 0.0 };
        let c = DelaunayPoint { x: 0.5, y: 1.0 };
        let inside = DelaunayPoint { x: 0.5, y: 0.4 };
        let outside = DelaunayPoint { x: 5.0, y: 5.0 };
        assert!(point_in_circumcircle(&inside, &a, &b, &c));
        assert!(!point_in_circumcircle(&outside, &a, &b, &c));
    }

    #[test]
    fn test_triangle_area_2d() {
        let a = DelaunayPoint { x: 0.0, y: 0.0 };
        let b = DelaunayPoint { x: 2.0, y: 0.0 };
        let c = DelaunayPoint { x: 0.0, y: 2.0 };
        let area = triangle_area_2d(&a, &b, &c);
        assert!((area - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_ccw() {
        let a = DelaunayPoint { x: 0.0, y: 0.0 };
        let b = DelaunayPoint { x: 1.0, y: 0.0 };
        let c = DelaunayPoint { x: 0.0, y: 1.0 };
        assert!(is_ccw(&a, &b, &c));
        assert!(!is_ccw(&a, &c, &b));
    }

    #[test]
    fn test_delaunay_triangle_to_json() {
        let t = DelaunayTriangle { a: 0, b: 1, c: 2 };
        let json = delaunay_triangle_to_json(&t);
        assert!(json.contains("\"a\":0"));
        assert!(json.contains("\"b\":1"));
        assert!(json.contains("\"c\":2"));
    }

    #[test]
    fn test_delaunay_result_to_json() {
        let pts = square_points();
        let cfg = default_delaunay_config();
        let result = triangulate(&pts, &cfg);
        let json = delaunay_result_to_json(&result);
        assert!(json.contains("\"point_count\":4"));
    }

    #[test]
    fn test_point_count_and_triangle_count() {
        let pts = square_points();
        let cfg = default_delaunay_config();
        let result = triangulate(&pts, &cfg);
        assert_eq!(point_count(&result), 4);
        assert_eq!(triangle_count_delaunay(&result), 2);
    }

    #[test]
    fn test_triangulate_empty() {
        let cfg = default_delaunay_config();
        let result = triangulate(&[], &cfg);
        assert_eq!(result.point_count, 0);
        assert!(result.triangles.is_empty());
    }

    #[test]
    fn test_circumcenter_equilateral() {
        let a = DelaunayPoint { x: 0.0, y: 0.0 };
        let b = DelaunayPoint { x: 1.0, y: 0.0 };
        let c = DelaunayPoint { x: 0.5, y: 0.866_025 };
        let cc = triangle_circumcenter(&a, &b, &c);
        // circumcenter of equilateral triangle should be near centroid
        assert!((cc.x - 0.5).abs() < 0.01);
    }
}
