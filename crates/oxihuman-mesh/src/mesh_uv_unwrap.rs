//! Simple angle-based UV unwrapping (LSCM-lite: fix two boundary vertices, relax the rest).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvUnwrapConfig {
    pub iterations: usize,
    pub relaxation: f32,
    pub pin_vertex_a: usize,
    pub pin_vertex_b: usize,
    pub pin_uv_a: [f32; 2],
    pub pin_uv_b: [f32; 2],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvUnwrapResult {
    pub uvs: Vec<[f32; 2]>,
    pub vertex_count: usize,
    pub seam_count: usize,
    pub chart_count: usize,
    pub distortion: f32,
    pub coverage: f32,
}

#[allow(dead_code)]
pub fn default_uv_unwrap_config() -> UvUnwrapConfig {
    UvUnwrapConfig {
        iterations: 64,
        relaxation: 0.5,
        pin_vertex_a: 0,
        pin_vertex_b: 1,
        pin_uv_a: [0.0, 0.0],
        pin_uv_b: [1.0, 0.0],
    }
}

/// Perform a simple Laplacian relaxation UV unwrap.
/// `positions` is vertex positions, `triangles` is index triples.
#[allow(dead_code)]
pub fn uv_unwrap_mesh(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &UvUnwrapConfig,
) -> UvUnwrapResult {
    let n = positions.len();
    if n == 0 {
        return UvUnwrapResult {
            uvs: Vec::new(),
            vertex_count: 0,
            seam_count: 0,
            chart_count: 0,
            distortion: 0.0,
            coverage: 0.0,
        };
    }

    // Initialize UVs by projecting onto XZ plane
    let mut uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0], p[2]]).collect();

    // Override pinned vertices
    if config.pin_vertex_a < n {
        uvs[config.pin_vertex_a] = config.pin_uv_a;
    }
    if config.pin_vertex_b < n {
        uvs[config.pin_vertex_b] = config.pin_uv_b;
    }

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for tri in triangles {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            if !adj[a].contains(&b) { adj[a].push(b); }
            if !adj[b].contains(&a) { adj[b].push(a); }
        }
    }

    // Laplacian relaxation
    for _ in 0..config.iterations {
        let prev = uvs.clone();
        for v in 0..n {
            if v == config.pin_vertex_a || v == config.pin_vertex_b {
                continue;
            }
            let neighbors = &adj[v];
            if neighbors.is_empty() {
                continue;
            }
            let sum_u: f32 = neighbors.iter().map(|&nb| prev[nb][0]).sum();
            let sum_v: f32 = neighbors.iter().map(|&nb| prev[nb][1]).sum();
            let cnt = neighbors.len() as f32;
            let avg = [sum_u / cnt, sum_v / cnt];
            let r = config.relaxation;
            uvs[v][0] = prev[v][0] * (1.0 - r) + avg[0] * r;
            uvs[v][1] = prev[v][1] * (1.0 - r) + avg[1] * r;
        }
    }

    let distortion = compute_distortion(positions, triangles, &uvs);
    let coverage = compute_coverage(&uvs);

    UvUnwrapResult {
        vertex_count: n,
        seam_count: 0,
        chart_count: 1,
        distortion,
        coverage,
        uvs,
    }
}

fn compute_distortion(positions: &[[f32; 3]], triangles: &[[usize; 3]], uvs: &[[f32; 2]]) -> f32 {
    if triangles.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    for tri in triangles {
        let [a, b, c] = *tri;
        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }
        let p0 = positions[a]; let p1 = positions[b]; let p2 = positions[c];
        let e1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
        let e2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
        let area3d = (e1[1]*e2[2] - e1[2]*e2[1]).powi(2)
            + (e1[2]*e2[0] - e1[0]*e2[2]).powi(2)
            + (e1[0]*e2[1] - e1[1]*e2[0]).powi(2);
        let area3d = area3d.sqrt() * 0.5;

        let u0 = uvs[a]; let u1 = uvs[b]; let u2 = uvs[c];
        let f1 = [u1[0]-u0[0], u1[1]-u0[1]];
        let f2 = [u2[0]-u0[0], u2[1]-u0[1]];
        let area2d = (f1[0]*f2[1] - f1[1]*f2[0]).abs() * 0.5;

        if area3d > 1e-8 && area2d > 1e-8 {
            total += (area3d / area2d - 1.0).abs();
        }
    }
    total / (triangles.len() as f32).max(1.0)
}

fn compute_coverage(uvs: &[[f32; 2]]) -> f32 {
    if uvs.is_empty() {
        return 0.0;
    }
    let min_u = uvs.iter().map(|u| u[0]).fold(f32::MAX, f32::min);
    let max_u = uvs.iter().map(|u| u[0]).fold(f32::MIN, f32::max);
    let min_v = uvs.iter().map(|u| u[1]).fold(f32::MAX, f32::min);
    let max_v = uvs.iter().map(|u| u[1]).fold(f32::MIN, f32::max);
    let w = (max_u - min_u).max(0.0);
    let h = (max_v - min_v).max(0.0);
    (w * h).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn uv_unwrap_vertex_count(result: &UvUnwrapResult) -> usize {
    result.vertex_count
}

#[allow(dead_code)]
pub fn uv_unwrap_distortion(result: &UvUnwrapResult) -> f32 {
    result.distortion
}

#[allow(dead_code)]
pub fn uv_unwrap_to_json(result: &UvUnwrapResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"seam_count\":{},\"chart_count\":{},\"distortion\":{:.6},\"coverage\":{:.6}}}",
        result.vertex_count, result.seam_count, result.chart_count, result.distortion, result.coverage
    )
}

#[allow(dead_code)]
pub fn uv_unwrap_seam_count(result: &UvUnwrapResult) -> usize {
    result.seam_count
}

#[allow(dead_code)]
pub fn uv_unwrap_chart_count(result: &UvUnwrapResult) -> usize {
    result.chart_count
}

#[allow(dead_code)]
pub fn uv_unwrap_coverage(result: &UvUnwrapResult) -> f32 {
    result.coverage
}

#[allow(dead_code)]
pub fn uv_unwrap_normalize(result: &mut UvUnwrapResult) {
    if result.uvs.is_empty() {
        return;
    }
    let min_u = result.uvs.iter().map(|u| u[0]).fold(f32::MAX, f32::min);
    let max_u = result.uvs.iter().map(|u| u[0]).fold(f32::MIN, f32::max);
    let min_v = result.uvs.iter().map(|u| u[1]).fold(f32::MAX, f32::min);
    let max_v = result.uvs.iter().map(|u| u[1]).fold(f32::MIN, f32::max);
    let rw = if (max_u - min_u).abs() > 1e-8 { 1.0 / (max_u - min_u) } else { 1.0 };
    let rh = if (max_v - min_v).abs() > 1e-8 { 1.0 / (max_v - min_v) } else { 1.0 };
    for uv in &mut result.uvs {
        uv[0] = (uv[0] - min_u) * rw;
        uv[1] = (uv[1] - min_v) * rh;
    }
}

#[allow(dead_code)]
pub fn uv_unwrap_flip_v(result: &mut UvUnwrapResult) {
    for uv in &mut result.uvs {
        uv[1] = 1.0 - uv[1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,0.0,1.0],[0.0,0.0,1.0]]
    }

    fn quad_triangles() -> Vec<[usize; 3]> {
        vec![[0,1,2],[0,2,3]]
    }

    #[test]
    fn test_default_config() {
        let cfg = default_uv_unwrap_config();
        assert_eq!(cfg.iterations, 64);
        assert!((cfg.relaxation - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_unwrap_vertex_count() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        assert_eq!(uv_unwrap_vertex_count(&res), 4);
    }

    #[test]
    fn test_empty_mesh() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&[], &[], &cfg);
        assert_eq!(uv_unwrap_vertex_count(&res), 0);
        assert!(res.uvs.is_empty());
    }

    #[test]
    fn test_pin_vertices_fixed() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        assert!((res.uvs[0][0] - 0.0).abs() < 1e-5);
        assert!((res.uvs[0][1] - 0.0).abs() < 1e-5);
        assert!((res.uvs[1][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_distortion_finite() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        assert!(uv_unwrap_distortion(&res).is_finite());
    }

    #[test]
    fn test_to_json_fields() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        let json = uv_unwrap_to_json(&res);
        assert!(json.contains("vertex_count"));
        assert!(json.contains("chart_count"));
        assert!(json.contains("distortion"));
    }

    #[test]
    fn test_normalize() {
        let cfg = default_uv_unwrap_config();
        let mut res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        uv_unwrap_normalize(&mut res);
        let max_u = res.uvs.iter().map(|u| u[0]).fold(f32::MIN, f32::max);
        let min_u = res.uvs.iter().map(|u| u[0]).fold(f32::MAX, f32::min);
        assert!((max_u - 1.0).abs() < 1e-5);
        assert!((min_u - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_flip_v() {
        let cfg = default_uv_unwrap_config();
        let mut res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        let before = res.uvs[2][1];
        uv_unwrap_flip_v(&mut res);
        assert!((res.uvs[2][1] - (1.0 - before)).abs() < 1e-5);
    }

    #[test]
    fn test_chart_seam_count() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        assert_eq!(uv_unwrap_chart_count(&res), 1);
        assert_eq!(uv_unwrap_seam_count(&res), 0);
    }

    #[test]
    fn test_coverage_nonneg() {
        let cfg = default_uv_unwrap_config();
        let res = uv_unwrap_mesh(&quad_positions(), &quad_triangles(), &cfg);
        assert!(uv_unwrap_coverage(&res) >= 0.0);
    }
}
