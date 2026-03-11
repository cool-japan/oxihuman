// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Corner table data structure for efficient mesh traversal.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CornerTable {
    pub corners: Vec<Corner>,
    pub triangle_count: usize,
}

/// A corner references a vertex within a triangle.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Corner {
    pub vertex: u32,
    pub opposite: i32,
}

#[allow(dead_code)]
impl CornerTable {
    /// Build a corner table from triangle indices.
    pub fn build(indices: &[u32]) -> Self {
        let tri_count = indices.len() / 3;
        let mut corners: Vec<Corner> = indices
            .iter()
            .map(|&v| Corner { vertex: v, opposite: -1 })
            .collect();
        for i in 0..tri_count {
            for j in 0..3 {
                let ci = i * 3 + j;
                let v0 = corners[ci].vertex;
                let v1 = corners[i * 3 + (j + 1) % 3].vertex;
                for k in (i + 1)..tri_count {
                    for l in 0..3 {
                        let ck = k * 3 + l;
                        let u0 = corners[ck].vertex;
                        let u1 = corners[k * 3 + (l + 1) % 3].vertex;
                        if v0 == u1 && v1 == u0 {
                            corners[ci].opposite = ck as i32;
                            corners[ck].opposite = ci as i32;
                        }
                    }
                }
            }
        }
        Self { corners, triangle_count: tri_count }
    }

    /// Get the next corner in the same triangle.
    pub fn next(&self, corner: usize) -> usize {
        let tri = corner / 3;
        let local = corner % 3;
        tri * 3 + (local + 1) % 3
    }

    /// Get the previous corner in the same triangle.
    pub fn prev(&self, corner: usize) -> usize {
        let tri = corner / 3;
        let local = corner % 3;
        tri * 3 + (local + 2) % 3
    }

    /// Get the opposite corner (-1 if boundary).
    pub fn opposite(&self, corner: usize) -> i32 {
        self.corners[corner].opposite
    }

    /// Get the vertex at a corner.
    pub fn vertex_at(&self, corner: usize) -> u32 {
        self.corners[corner].vertex
    }

    /// Total corners.
    pub fn corner_count(&self) -> usize {
        self.corners.len()
    }

    /// Check if a corner is on a boundary.
    pub fn is_boundary(&self, corner: usize) -> bool {
        self.corners[corner].opposite < 0
    }

    /// Count boundary corners.
    pub fn boundary_count(&self) -> usize {
        self.corners.iter().filter(|c| c.opposite < 0).count()
    }
}

/// Build corner table (convenience).
#[allow(dead_code)]
pub fn build_corner_table(indices: &[u32]) -> CornerTable {
    CornerTable::build(indices)
}

/// Serialize corner table stats to JSON.
#[allow(dead_code)]
pub fn corner_table_to_json(ct: &CornerTable) -> String {
    format!(
        "{{\"triangles\":{},\"corners\":{},\"boundary_corners\":{}}}",
        ct.triangle_count,
        ct.corner_count(),
        ct.boundary_count()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> Vec<u32> { vec![0, 1, 2] }
    fn two_tris() -> Vec<u32> { vec![0, 1, 2, 2, 1, 3] }

    #[test]
    fn test_build_single() {
        let ct = build_corner_table(&single_tri());
        assert_eq!(ct.triangle_count, 1);
        assert_eq!(ct.corner_count(), 3);
    }

    #[test]
    fn test_next_prev() {
        let ct = build_corner_table(&single_tri());
        assert_eq!(ct.next(0), 1);
        assert_eq!(ct.prev(0), 2);
    }

    #[test]
    fn test_boundary_single() {
        let ct = build_corner_table(&single_tri());
        assert_eq!(ct.boundary_count(), 3);
    }

    #[test]
    fn test_two_tris_opposite() {
        let ct = build_corner_table(&two_tris());
        let mut has_opposite = false;
        for i in 0..ct.corner_count() {
            if ct.opposite(i) >= 0 {
                has_opposite = true;
            }
        }
        assert!(has_opposite);
    }

    #[test]
    fn test_vertex_at() {
        let ct = build_corner_table(&single_tri());
        assert_eq!(ct.vertex_at(0), 0);
        assert_eq!(ct.vertex_at(1), 1);
    }

    #[test]
    fn test_is_boundary() {
        let ct = build_corner_table(&single_tri());
        assert!(ct.is_boundary(0));
    }

    #[test]
    fn test_empty() {
        let ct = build_corner_table(&[]);
        assert_eq!(ct.triangle_count, 0);
    }

    #[test]
    fn test_to_json() {
        let ct = build_corner_table(&single_tri());
        let json = corner_table_to_json(&ct);
        assert!(json.contains("triangles"));
    }

    #[test]
    fn test_two_tris_boundary_count() {
        let ct = build_corner_table(&two_tris());
        assert!(ct.boundary_count() < 6);
    }

    #[test]
    fn test_next_wraps() {
        let ct = build_corner_table(&single_tri());
        assert_eq!(ct.next(2), 0);
    }
}
