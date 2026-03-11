#![allow(dead_code)]
//! Edge bevel operations.

/// Edge bevel result.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeBevel {
    pub new_vertices: Vec<[f32; 3]>,
    pub new_faces: Vec<[u32; 3]>,
    pub segments: u32,
    pub width: f32,
}

/// Bevel a single edge.
#[allow(dead_code)]
pub fn bevel_edge(
    positions: &[[f32; 3]],
    edge: [u32; 2],
    width: f32,
    segments: u32,
) -> EdgeBevel {
    let a = positions[edge[0] as usize];
    let b = positions[edge[1] as usize];
    let segs = segments.max(1);
    let mut new_verts = Vec::new();
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        new_verts.push([
            a[0] + (b[0] - a[0]) * t + width * 0.5,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]);
        new_verts.push([
            a[0] + (b[0] - a[0]) * t - width * 0.5,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]);
    }
    let base = positions.len() as u32;
    let mut new_faces = Vec::new();
    for i in 0..segs {
        let i0 = base + i * 2;
        let i1 = base + i * 2 + 1;
        let i2 = base + (i + 1) * 2;
        let i3 = base + (i + 1) * 2 + 1;
        new_faces.push([i0, i2, i1]);
        new_faces.push([i1, i2, i3]);
    }
    EdgeBevel {
        new_vertices: new_verts,
        new_faces,
        segments: segs,
        width,
    }
}

/// Bevel all edges.
#[allow(dead_code)]
pub fn bevel_all_edges(
    positions: &[[f32; 3]],
    edges: &[[u32; 2]],
    width: f32,
    segments: u32,
) -> Vec<EdgeBevel> {
    edges.iter().map(|e| bevel_edge(positions, *e, width, segments)).collect()
}

/// Get segments count.
#[allow(dead_code)]
pub fn bevel_segments(eb: &EdgeBevel) -> u32 {
    eb.segments
}

/// Get bevel width.
#[allow(dead_code)]
pub fn bevel_width(eb: &EdgeBevel) -> f32 {
    eb.width
}

/// Count new vertices.
#[allow(dead_code)]
pub fn bevel_vertex_count(eb: &EdgeBevel) -> usize {
    eb.new_vertices.len()
}

/// Count new faces.
#[allow(dead_code)]
pub fn bevel_face_count(eb: &EdgeBevel) -> usize {
    eb.new_faces.len()
}

/// Get bevel profile (linear interpolation weights).
#[allow(dead_code)]
pub fn bevel_profile(segments: u32) -> Vec<f32> {
    let segs = segments.max(1);
    (0..=segs).map(|i| i as f32 / segs as f32).collect()
}

/// Serialize bevel to JSON.
#[allow(dead_code)]
pub fn bevel_to_json(eb: &EdgeBevel) -> String {
    format!(
        "{{\"segments\":{},\"width\":{:.4},\"new_verts\":{},\"new_faces\":{}}}",
        eb.segments,
        eb.width,
        eb.new_vertices.len(),
        eb.new_faces.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bevel_edge() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let eb = bevel_edge(&pos, [0, 1], 0.1, 2);
        assert!(!eb.new_vertices.is_empty());
        assert!(!eb.new_faces.is_empty());
    }

    #[test]
    fn test_bevel_edge_single_segment() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let eb = bevel_edge(&pos, [0, 1], 0.1, 1);
        assert_eq!(eb.segments, 1);
    }

    #[test]
    fn test_bevel_all_edges() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let edges = vec![[0, 1], [1, 2]];
        let results = bevel_all_edges(&pos, &edges, 0.1, 1);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_bevel_segments() {
        let eb = EdgeBevel { new_vertices: vec![], new_faces: vec![], segments: 3, width: 0.1 };
        assert_eq!(bevel_segments(&eb), 3);
    }

    #[test]
    fn test_bevel_width() {
        let eb = EdgeBevel { new_vertices: vec![], new_faces: vec![], segments: 1, width: 0.25 };
        assert!((bevel_width(&eb) - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_bevel_vertex_count() {
        let eb = EdgeBevel { new_vertices: vec![[0.0; 3]; 4], new_faces: vec![], segments: 1, width: 0.1 };
        assert_eq!(bevel_vertex_count(&eb), 4);
    }

    #[test]
    fn test_bevel_face_count() {
        let eb = EdgeBevel { new_vertices: vec![], new_faces: vec![[0, 1, 2]], segments: 1, width: 0.1 };
        assert_eq!(bevel_face_count(&eb), 1);
    }

    #[test]
    fn test_bevel_profile() {
        let p = bevel_profile(4);
        assert_eq!(p.len(), 5);
        assert!((p[0]).abs() < 1e-6);
        assert!((p[4] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bevel_to_json() {
        let eb = EdgeBevel { new_vertices: vec![], new_faces: vec![], segments: 2, width: 0.5 };
        let j = bevel_to_json(&eb);
        assert!(j.contains("segments"));
    }

    #[test]
    fn test_bevel_edge_zero_width() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let eb = bevel_edge(&pos, [0, 1], 0.0, 1);
        assert!(!eb.new_vertices.is_empty());
    }
}
