// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Mesh statistics report export (HTML/JSON).

use std::collections::HashMap;

#[allow(dead_code)]
pub struct MeshStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub non_manifold_edges: usize,
    pub boundary_edges: usize,
    pub isolated_vertices: usize,
    pub degenerate_faces: usize,
    pub bounding_box: ([f32; 3], [f32; 3]),
    pub surface_area: f32,
    pub volume: f32,
    pub has_uvs: bool,
    pub has_normals: bool,
}

#[allow(dead_code)]
pub struct MeshReport {
    pub title: String,
    pub stats: MeshStats,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

// Type alias for edge map used in boundary/manifold counting
type EdgeMap = HashMap<(u32, u32), u32>;

#[allow(dead_code)]
pub fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    len * 0.5
}

#[allow(dead_code)]
pub fn mesh_surface_area(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let mut total = 0.0f32;
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            break;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a < positions.len() && b < positions.len() && c < positions.len() {
            total += triangle_area(positions[a], positions[b], positions[c]);
        }
    }
    total
}

#[allow(dead_code)]
pub fn mesh_volume_signed(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    // Signed volume via divergence theorem: V = (1/6) * sum over faces of (v0 . (v1 x v2))
    let mut total = 0.0f32;
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            break;
        }
        let (ai, bi, ci) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if ai < positions.len() && bi < positions.len() && ci < positions.len() {
            let a = positions[ai];
            let b = positions[bi];
            let c = positions[ci];
            // (b x c) . a
            let bxc = [
                b[1] * c[2] - b[2] * c[1],
                b[2] * c[0] - b[0] * c[2],
                b[0] * c[1] - b[1] * c[0],
            ];
            total += a[0] * bxc[0] + a[1] * bxc[1] + a[2] * bxc[2];
        }
    }
    total / 6.0
}

#[allow(dead_code)]
pub fn find_degenerate_faces(positions: &[[f32; 3]], indices: &[u32]) -> Vec<usize> {
    const EPSILON: f32 = 1e-10;
    let mut result = Vec::new();
    for (face_idx, tri) in indices.chunks(3).enumerate() {
        if tri.len() < 3 {
            break;
        }
        let (ai, bi, ci) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if ai < positions.len() && bi < positions.len() && ci < positions.len() {
            let area = triangle_area(positions[ai], positions[bi], positions[ci]);
            if area < EPSILON {
                result.push(face_idx);
            }
        }
    }
    result
}

fn build_edge_map(indices: &[u32]) -> EdgeMap {
    let mut map: EdgeMap = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            break;
        }
        let verts = [tri[0], tri[1], tri[2]];
        for e in 0..3 {
            let a = verts[e];
            let b = verts[(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *map.entry(key).or_insert(0) += 1;
        }
    }
    map
}

#[allow(dead_code)]
pub fn count_boundary_mesh_edges(indices: &[u32]) -> usize {
    let map = build_edge_map(indices);
    map.values().filter(|&&c| c == 1).count()
}

fn count_non_manifold_edges(indices: &[u32]) -> usize {
    let map = build_edge_map(indices);
    map.values().filter(|&&c| c > 2).count()
}

fn count_isolated_vertices(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    let mut used = vec![false; positions.len()];
    for &idx in indices {
        let i = idx as usize;
        if i < used.len() {
            used[i] = true;
        }
    }
    used.iter().filter(|&&u| !u).count()
}

fn compute_bounding_box(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for p in positions.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

#[allow(dead_code)]
pub fn compute_mesh_stats(
    positions: &[[f32; 3]],
    indices: &[u32],
    has_uvs: bool,
    has_normals: bool,
) -> MeshStats {
    let face_count = indices.len() / 3;
    let edge_count = build_edge_map(indices).len();
    let degenerate_faces = find_degenerate_faces(positions, indices).len();
    let boundary_edges = count_boundary_mesh_edges(indices);
    let non_manifold_edges = count_non_manifold_edges(indices);
    let isolated_vertices = count_isolated_vertices(positions, indices);
    let bounding_box = compute_bounding_box(positions);
    let surface_area = mesh_surface_area(positions, indices);
    let volume = mesh_volume_signed(positions, indices).abs();

    MeshStats {
        vertex_count: positions.len(),
        face_count,
        edge_count,
        non_manifold_edges,
        boundary_edges,
        isolated_vertices,
        degenerate_faces,
        bounding_box,
        surface_area,
        volume,
        has_uvs,
        has_normals,
    }
}

#[allow(dead_code)]
pub fn generate_mesh_report(
    title: &str,
    positions: &[[f32; 3]],
    indices: &[u32],
    has_uvs: bool,
    has_normals: bool,
) -> MeshReport {
    let stats = compute_mesh_stats(positions, indices, has_uvs, has_normals);
    let mut warnings = Vec::new();
    let mut info = Vec::new();

    if stats.boundary_edges > 0 {
        warnings.push(format!(
            "Mesh is not closed: {} boundary edges",
            stats.boundary_edges
        ));
    }
    if stats.non_manifold_edges > 0 {
        warnings.push(format!(
            "Non-manifold edges detected: {}",
            stats.non_manifold_edges
        ));
    }
    if stats.degenerate_faces > 0 {
        warnings.push(format!(
            "Degenerate faces detected: {}",
            stats.degenerate_faces
        ));
    }
    if stats.isolated_vertices > 0 {
        warnings.push(format!(
            "Isolated (unused) vertices: {}",
            stats.isolated_vertices
        ));
    }

    info.push(format!("Vertices: {}", stats.vertex_count));
    info.push(format!("Faces: {}", stats.face_count));
    info.push(format!("Edges: {}", stats.edge_count));
    info.push(format!("Surface area: {:.4}", stats.surface_area));
    info.push(format!("Volume: {:.4}", stats.volume));
    if !has_uvs {
        info.push("No UV coordinates".to_string());
    }
    if !has_normals {
        info.push("No normals".to_string());
    }

    MeshReport {
        title: title.to_string(),
        stats,
        warnings,
        info,
    }
}

#[allow(dead_code)]
pub fn report_to_json(report: &MeshReport) -> String {
    let bb_min = report.stats.bounding_box.0;
    let bb_max = report.stats.bounding_box.1;
    format!(
        r#"{{"title":"{title}","stats":{{"vertex_count":{vc},"face_count":{fc},"edge_count":{ec},"non_manifold_edges":{nme},"boundary_edges":{be},"isolated_vertices":{iv},"degenerate_faces":{df},"bounding_box_min":[{bx0},{bx1},{bx2}],"bounding_box_max":[{bxm0},{bxm1},{bxm2}],"surface_area":{sa},"volume":{vol},"has_uvs":{uv},"has_normals":{nm}}},"warnings":[{warns}],"info":[{infos}]}}"#,
        title = report.title,
        vc = report.stats.vertex_count,
        fc = report.stats.face_count,
        ec = report.stats.edge_count,
        nme = report.stats.non_manifold_edges,
        be = report.stats.boundary_edges,
        iv = report.stats.isolated_vertices,
        df = report.stats.degenerate_faces,
        bx0 = bb_min[0],
        bx1 = bb_min[1],
        bx2 = bb_min[2],
        bxm0 = bb_max[0],
        bxm1 = bb_max[1],
        bxm2 = bb_max[2],
        sa = report.stats.surface_area,
        vol = report.stats.volume,
        uv = report.stats.has_uvs,
        nm = report.stats.has_normals,
        warns = report
            .warnings
            .iter()
            .map(|w| format!("\"{}\"", w))
            .collect::<Vec<_>>()
            .join(","),
        infos = report
            .info
            .iter()
            .map(|i| format!("\"{}\"", i))
            .collect::<Vec<_>>()
            .join(","),
    )
}

#[allow(dead_code)]
pub fn report_to_html(report: &MeshReport) -> String {
    let bb_min = report.stats.bounding_box.0;
    let bb_max = report.stats.bounding_box.1;
    let warnings_html = if report.warnings.is_empty() {
        "<p>No warnings.</p>".to_string()
    } else {
        let items: String = report
            .warnings
            .iter()
            .map(|w| format!("<li>{}</li>", w))
            .collect();
        format!("<ul>{}</ul>", items)
    };
    format!(
        r#"<html><head><title>{title}</title></head><body>
<h1>{title}</h1>
<table border="1">
<tr><th>Property</th><th>Value</th></tr>
<tr><td>Vertices</td><td>{vc}</td></tr>
<tr><td>Faces</td><td>{fc}</td></tr>
<tr><td>Edges</td><td>{ec}</td></tr>
<tr><td>Non-manifold edges</td><td>{nme}</td></tr>
<tr><td>Boundary edges</td><td>{be}</td></tr>
<tr><td>Isolated vertices</td><td>{iv}</td></tr>
<tr><td>Degenerate faces</td><td>{df}</td></tr>
<tr><td>Bounding box min</td><td>[{bx0:.4}, {bx1:.4}, {bx2:.4}]</td></tr>
<tr><td>Bounding box max</td><td>[{bxm0:.4}, {bxm1:.4}, {bxm2:.4}]</td></tr>
<tr><td>Surface area</td><td>{sa:.4}</td></tr>
<tr><td>Volume</td><td>{vol:.4}</td></tr>
<tr><td>Has UVs</td><td>{uv}</td></tr>
<tr><td>Has Normals</td><td>{nm}</td></tr>
</table>
<h2>Warnings</h2>
{warns}
</body></html>"#,
        title = report.title,
        vc = report.stats.vertex_count,
        fc = report.stats.face_count,
        ec = report.stats.edge_count,
        nme = report.stats.non_manifold_edges,
        be = report.stats.boundary_edges,
        iv = report.stats.isolated_vertices,
        df = report.stats.degenerate_faces,
        bx0 = bb_min[0],
        bx1 = bb_min[1],
        bx2 = bb_min[2],
        bxm0 = bb_max[0],
        bxm1 = bb_max[1],
        bxm2 = bb_max[2],
        sa = report.stats.surface_area,
        vol = report.stats.volume,
        uv = report.stats.has_uvs,
        nm = report.stats.has_normals,
        warns = warnings_html,
    )
}

#[allow(dead_code)]
pub fn report_warnings(report: &MeshReport) -> &[String] {
    &report.warnings
}

#[allow(dead_code)]
pub fn mesh_health_score(stats: &MeshStats) -> f32 {
    let mut score = 1.0f32;
    if stats.boundary_edges > 0 {
        score -= 0.2 * (stats.boundary_edges as f32 / (stats.edge_count.max(1) as f32)).min(1.0);
    }
    if stats.non_manifold_edges > 0 {
        score -=
            0.3 * (stats.non_manifold_edges as f32 / (stats.edge_count.max(1) as f32)).min(1.0);
    }
    if stats.degenerate_faces > 0 {
        score -= 0.2 * (stats.degenerate_faces as f32 / (stats.face_count.max(1) as f32)).min(1.0);
    }
    if stats.isolated_vertices > 0 {
        score -=
            0.1 * (stats.isolated_vertices as f32 / (stats.vertex_count.max(1) as f32)).min(1.0);
    }
    score.max(0.0)
}

#[allow(dead_code)]
pub fn is_watertight(stats: &MeshStats) -> bool {
    stats.boundary_edges == 0 && stats.non_manifold_edges == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn unit_triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    // Simple tetrahedron (4 vertices, 4 faces, closed)
    fn tetra_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
    }

    fn tetra_indices() -> Vec<u32> {
        vec![
            0, 1, 2, // base
            0, 1, 3, // side 1
            1, 2, 3, // side 2
            0, 2, 3, // side 3
        ]
    }

    #[test]
    fn test_triangle_area_unit() {
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(
            (area - 0.5).abs() < 1e-5,
            "unit right triangle should have area 0.5"
        );
    }

    #[test]
    fn test_triangle_area_degenerate() {
        // Collinear points => zero area
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!(area < 1e-10, "degenerate triangle should have area ~0");
    }

    #[test]
    fn test_triangle_area_equilateral() {
        let s = 2.0f32;
        let h = (3.0f32).sqrt();
        let area = triangle_area([0.0, 0.0, 0.0], [s, 0.0, 0.0], [s / 2.0, h, 0.0]);
        let expected = s * h / 2.0;
        assert!((area - expected).abs() < 1e-4, "equilateral area mismatch");
    }

    #[test]
    fn test_mesh_surface_area_single_triangle() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let area = mesh_surface_area(&pos, &idx);
        assert!((area - 0.5).abs() < 1e-5, "single unit triangle area = 0.5");
    }

    #[test]
    fn test_mesh_surface_area_two_triangles() {
        // Two right triangles making a unit square, area = 1.0
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        let area = mesh_surface_area(&pos, &idx);
        assert!((area - 1.0).abs() < 1e-5, "quad area should be 1.0");
    }

    #[test]
    fn test_mesh_volume_signed_tetra() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let vol = mesh_volume_signed(&pos, &idx).abs();
        // Tetrahedron with those vertices: V = 1/6
        assert!(vol > 0.0, "tetrahedron should have nonzero volume");
    }

    #[test]
    fn test_mesh_volume_signed_flat_mesh() {
        // A flat triangle has zero volume
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let vol = mesh_volume_signed(&pos, &idx).abs();
        assert!(vol < 1e-5, "flat mesh volume ~ 0");
    }

    #[test]
    fn test_find_degenerate_faces_none() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let degen = find_degenerate_faces(&pos, &idx);
        assert!(degen.is_empty(), "no degenerate faces in unit triangle");
    }

    #[test]
    fn test_find_degenerate_faces_collinear() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let idx = vec![0, 1, 2];
        let degen = find_degenerate_faces(&pos, &idx);
        assert_eq!(degen.len(), 1, "collinear triangle is degenerate");
    }

    #[test]
    fn test_count_boundary_edges_open_mesh() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let boundary = count_boundary_mesh_edges(&idx);
        // A single triangle has 3 boundary edges
        assert_eq!(boundary, 3);
        let _ = pos;
    }

    #[test]
    fn test_count_boundary_edges_closed_tetra() {
        let idx = tetra_indices();
        let boundary = count_boundary_mesh_edges(&idx);
        assert_eq!(boundary, 0, "closed tetrahedron has no boundary edges");
    }

    #[test]
    fn test_generate_mesh_report() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let report = generate_mesh_report("Test", &pos, &idx, false, false);
        assert_eq!(report.title, "Test");
        assert_eq!(report.stats.vertex_count, 3);
        assert_eq!(report.stats.face_count, 1);
        assert!(
            !report.warnings.is_empty(),
            "should warn about boundary edges"
        );
    }

    #[test]
    fn test_report_to_json_non_empty() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let report = generate_mesh_report("JsonTest", &pos, &idx, true, true);
        let json = report_to_json(&report);
        assert!(!json.is_empty(), "JSON should not be empty");
        assert!(json.contains("JsonTest"), "JSON should contain title");
        assert!(
            json.contains("vertex_count"),
            "JSON should have vertex_count"
        );
    }

    #[test]
    fn test_report_to_html_contains_html_tag() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let report = generate_mesh_report("HtmlTest", &pos, &idx, false, true);
        let html = report_to_html(&report);
        assert!(html.contains("<html"), "HTML output should contain <html");
        assert!(html.contains("HtmlTest"), "HTML should contain the title");
        assert!(html.contains("<table"), "HTML should contain a table");
    }

    #[test]
    fn test_is_watertight_open() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let stats = compute_mesh_stats(&pos, &idx, false, false);
        assert!(!is_watertight(&stats), "open mesh is not watertight");
    }

    #[test]
    fn test_is_watertight_closed() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let stats = compute_mesh_stats(&pos, &idx, false, false);
        assert!(
            is_watertight(&stats),
            "closed tetrahedron should be watertight"
        );
    }

    #[test]
    fn test_mesh_health_score_perfect() {
        let pos = tetra_positions();
        let idx = tetra_indices();
        let stats = compute_mesh_stats(&pos, &idx, true, true);
        let score = mesh_health_score(&stats);
        assert!(
            (score - 1.0).abs() < 1e-5,
            "perfect mesh should have score 1.0"
        );
    }

    #[test]
    fn test_mesh_health_score_degraded() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let stats = compute_mesh_stats(&pos, &idx, false, false);
        let score = mesh_health_score(&stats);
        assert!(
            score < 1.0,
            "mesh with boundary edges should have score < 1.0"
        );
        assert!(score >= 0.0, "health score should be >= 0");
    }

    #[test]
    fn test_report_warnings_slice() {
        let pos = unit_triangle_positions();
        let idx = unit_triangle_indices();
        let report = generate_mesh_report("W", &pos, &idx, false, false);
        let warns = report_warnings(&report);
        assert!(!warns.is_empty(), "should have warnings for open mesh");
    }
}
