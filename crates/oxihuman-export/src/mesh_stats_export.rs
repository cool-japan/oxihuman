#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A statistics report about a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MeshStatsReport {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
    pub center_of_mass: [f32; 3],
}

/// Build a stats report from positions and triangle indices.
#[allow(dead_code)]
pub fn export_mesh_stats(positions: &[f32], indices: &[u32]) -> MeshStatsReport {
    let vc = positions.len() / 3;
    let fc = indices.len() / 3;
    // Euler: E = V + F - 2 (approx for closed manifold)
    let ec = if vc > 0 && fc > 0 { vc + fc } else { 0 };

    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];
    let mut cx = 0.0_f32;
    let mut cy = 0.0_f32;
    let mut cz = 0.0_f32;

    for i in 0..vc {
        let x = positions[i * 3];
        let y = positions[i * 3 + 1];
        let z = positions[i * 3 + 2];
        if x < bmin[0] { bmin[0] = x; }
        if y < bmin[1] { bmin[1] = y; }
        if z < bmin[2] { bmin[2] = z; }
        if x > bmax[0] { bmax[0] = x; }
        if y > bmax[1] { bmax[1] = y; }
        if z > bmax[2] { bmax[2] = z; }
        cx += x;
        cy += y;
        cz += z;
    }

    let com = if vc > 0 {
        let n = vc as f32;
        [cx / n, cy / n, cz / n]
    } else {
        [0.0; 3]
    };

    if vc == 0 {
        bmin = [0.0; 3];
        bmax = [0.0; 3];
    }

    MeshStatsReport {
        vertex_count: vc,
        face_count: fc,
        edge_count: ec,
        bbox_min: bmin,
        bbox_max: bmax,
        center_of_mass: com,
    }
}

/// Return vertex count from report.
#[allow(dead_code)]
pub fn stats_vertex_count(r: &MeshStatsReport) -> usize {
    r.vertex_count
}

/// Return face count from report.
#[allow(dead_code)]
pub fn stats_face_count(r: &MeshStatsReport) -> usize {
    r.face_count
}

/// Return edge count from report.
#[allow(dead_code)]
pub fn stats_edge_count(r: &MeshStatsReport) -> usize {
    r.edge_count
}

/// Return bounding box as (min, max).
#[allow(dead_code)]
pub fn stats_bounding_box(r: &MeshStatsReport) -> ([f32; 3], [f32; 3]) {
    (r.bbox_min, r.bbox_max)
}

/// Return center of mass.
#[allow(dead_code)]
pub fn stats_center_of_mass(r: &MeshStatsReport) -> [f32; 3] {
    r.center_of_mass
}

/// Serialize stats to JSON string.
#[allow(dead_code)]
pub fn stats_to_json(r: &MeshStatsReport) -> String {
    format!(
        "{{\"vertices\":{},\"faces\":{},\"edges\":{},\"center\":[{},{},{}]}}",
        r.vertex_count, r.face_count, r.edge_count,
        r.center_of_mass[0], r.center_of_mass[1], r.center_of_mass[2],
    )
}

/// Serialize stats to CSV row.
#[allow(dead_code)]
pub fn stats_to_csv(r: &MeshStatsReport) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{}",
        r.vertex_count, r.face_count, r.edge_count,
        r.bbox_min[0], r.bbox_min[1], r.bbox_min[2],
        r.bbox_max[0], r.bbox_max[1], r.bbox_max[2],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> MeshStatsReport {
        let pos = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let idx = vec![0, 1, 2];
        export_mesh_stats(&pos, &idx)
    }

    #[test]
    fn test_vertex_count() {
        assert_eq!(stats_vertex_count(&sample_report()), 3);
    }

    #[test]
    fn test_face_count() {
        assert_eq!(stats_face_count(&sample_report()), 1);
    }

    #[test]
    fn test_edge_count() {
        assert!(stats_edge_count(&sample_report()) > 0);
    }

    #[test]
    fn test_bounding_box() {
        let (bmin, bmax) = stats_bounding_box(&sample_report());
        assert!((bmin[0]).abs() < 1e-5);
        assert!((bmax[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_center_of_mass() {
        let com = stats_center_of_mass(&sample_report());
        assert!((com[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = stats_to_json(&sample_report());
        assert!(j.contains("\"vertices\":3"));
        assert!(j.contains("\"faces\":1"));
    }

    #[test]
    fn test_to_csv() {
        let c = stats_to_csv(&sample_report());
        assert!(c.contains("3,1"));
    }

    #[test]
    fn test_empty_mesh() {
        let r = export_mesh_stats(&[], &[]);
        assert_eq!(stats_vertex_count(&r), 0);
        assert_eq!(stats_face_count(&r), 0);
    }

    #[test]
    fn test_bbox_empty() {
        let r = export_mesh_stats(&[], &[]);
        let (bmin, bmax) = stats_bounding_box(&r);
        assert!((bmin[0]).abs() < 1e-5);
        assert!((bmax[0]).abs() < 1e-5);
    }

    #[test]
    fn test_csv_format() {
        let c = stats_to_csv(&sample_report());
        let parts: Vec<&str> = c.split(',').collect();
        assert_eq!(parts.len(), 9);
    }
}
