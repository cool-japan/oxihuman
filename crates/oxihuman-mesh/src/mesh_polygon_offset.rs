#![allow(dead_code)]

/// Result of a polygon offset operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PolygonOffset {
    pub vertices: Vec<[f32; 2]>,
    pub distance: f32,
}

/// Offset a 2D polygon by the given distance (positive = outward, negative = inward).
#[allow(dead_code)]
pub fn offset_polygon_2d(polygon: &[[f32; 2]], distance: f32) -> PolygonOffset {
    if polygon.len() < 3 {
        return PolygonOffset {
            vertices: polygon.to_vec(),
            distance,
        };
    }

    let n = polygon.len();
    let mut result = Vec::with_capacity(n);

    for i in 0..n {
        let prev = if i == 0 { n - 1 } else { i - 1 };
        let next = (i + 1) % n;

        // Edge normals
        let e1 = [polygon[i][0] - polygon[prev][0], polygon[i][1] - polygon[prev][1]];
        let e2 = [polygon[next][0] - polygon[i][0], polygon[next][1] - polygon[i][1]];

        let n1 = normalize_2d([-e1[1], e1[0]]);
        let n2 = normalize_2d([-e2[1], e2[0]]);

        let avg_n = normalize_2d([(n1[0] + n2[0]) * 0.5, (n1[1] + n2[1]) * 0.5]);

        // Compute the offset factor based on the angle between edges
        let dot = n1[0] * n2[0] + n1[1] * n2[1];
        let factor = if dot.abs() < 0.001 { 1.0 } else { 1.0 / (1.0 + dot).max(0.1).sqrt() };

        result.push([
            polygon[i][0] + avg_n[0] * distance * factor,
            polygon[i][1] + avg_n[1] * distance * factor,
        ]);
    }

    PolygonOffset {
        vertices: result,
        distance,
    }
}

/// Offset a polygon inward by the given distance.
#[allow(dead_code)]
pub fn offset_polygon_inward(polygon: &[[f32; 2]], distance: f32) -> PolygonOffset {
    offset_polygon_2d(polygon, -distance.abs())
}

/// Offset a polygon outward by the given distance.
#[allow(dead_code)]
pub fn offset_polygon_outward(polygon: &[[f32; 2]], distance: f32) -> PolygonOffset {
    offset_polygon_2d(polygon, distance.abs())
}

/// Get the offset distance.
#[allow(dead_code)]
pub fn offset_distance(offset: &PolygonOffset) -> f32 {
    offset.distance
}

/// Compute the area of the polygon after offset.
#[allow(dead_code)]
pub fn polygon_area_after_offset(offset: &PolygonOffset) -> f32 {
    polygon_area_2d(&offset.vertices)
}

/// Validate that the offset didn't create degenerate geometry.
#[allow(dead_code)]
pub fn validate_offset(offset: &PolygonOffset) -> bool {
    offset.vertices.len() >= 3 && polygon_area_2d(&offset.vertices) > 0.0
}

/// Get the number of vertices in the offset polygon.
#[allow(dead_code)]
pub fn offset_vertex_count(offset: &PolygonOffset) -> usize {
    offset.vertices.len()
}

/// Check if the offset creates a self-intersection.
#[allow(dead_code)]
pub fn offset_creates_self_intersection(offset: &PolygonOffset) -> bool {
    let n = offset.vertices.len();
    if n < 4 {
        return false;
    }
    for i in 0..n {
        let a1 = offset.vertices[i];
        let a2 = offset.vertices[(i + 1) % n];
        for j in (i + 2)..n {
            if j == (i + n - 1) % n {
                continue;
            }
            let b1 = offset.vertices[j];
            let b2 = offset.vertices[(j + 1) % n];
            if segments_intersect(a1, a2, b1, b2) {
                return true;
            }
        }
    }
    false
}

fn normalize_2d(v: [f32; 2]) -> [f32; 2] {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0];
    }
    [v[0] / len, v[1] / len]
}

fn polygon_area_2d(polygon: &[[f32; 2]]) -> f32 {
    let n = polygon.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i][0] * polygon[j][1];
        area -= polygon[j][0] * polygon[i][1];
    }
    (area * 0.5).abs()
}

fn segments_intersect(a1: [f32; 2], a2: [f32; 2], b1: [f32; 2], b2: [f32; 2]) -> bool {
    let d1 = cross_2d(a1, a2, b1);
    let d2 = cross_2d(a1, a2, b2);
    let d3 = cross_2d(b1, b2, a1);
    let d4 = cross_2d(b1, b2, a2);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

fn cross_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<[f32; 2]> {
        vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]
    }

    #[test]
    fn test_offset_outward() {
        let sq = square();
        let off = offset_polygon_outward(&sq, 0.1);
        assert_eq!(offset_vertex_count(&off), 4);
        let area = polygon_area_after_offset(&off);
        assert!(area > 1.0);
    }

    #[test]
    fn test_offset_inward() {
        let sq = square();
        let off = offset_polygon_inward(&sq, 0.1);
        let area = polygon_area_after_offset(&off);
        assert!(area < 1.0);
    }

    #[test]
    fn test_offset_distance() {
        let sq = square();
        let off = offset_polygon_2d(&sq, 0.5);
        assert!((offset_distance(&off) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_validate_offset() {
        let sq = square();
        let off = offset_polygon_outward(&sq, 0.1);
        assert!(validate_offset(&off));
    }

    #[test]
    fn test_vertex_count() {
        let sq = square();
        let off = offset_polygon_2d(&sq, 0.1);
        assert_eq!(offset_vertex_count(&off), 4);
    }

    #[test]
    fn test_no_self_intersection_small_offset() {
        let sq = square();
        let off = offset_polygon_outward(&sq, 0.01);
        assert!(!offset_creates_self_intersection(&off));
    }

    #[test]
    fn test_zero_offset() {
        let sq = square();
        let off = offset_polygon_2d(&sq, 0.0);
        let area = polygon_area_after_offset(&off);
        assert!((area - 1.0).abs() < 1e-2);
    }

    #[test]
    fn test_too_few_vertices() {
        let poly = vec![[0.0, 0.0], [1.0, 0.0]];
        let off = offset_polygon_2d(&poly, 0.1);
        assert_eq!(off.vertices.len(), 2);
    }

    #[test]
    fn test_triangle_offset() {
        let tri = vec![[0.0, 0.0], [2.0, 0.0], [1.0, 2.0]];
        let off = offset_polygon_outward(&tri, 0.1);
        assert_eq!(offset_vertex_count(&off), 3);
    }

    #[test]
    fn test_polygon_area_2d() {
        let sq = square();
        let area = polygon_area_2d(&sq);
        assert!((area - 1.0).abs() < 1e-6);
    }
}
