//! K-nearest-vertex and closest-point-on-surface queries using a brute-force spatial search.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProximityQueryConfig {
    pub max_k: usize,
    pub max_radius: f32,
    pub include_query_point: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProximityResult {
    pub indices: Vec<usize>,
    pub distances: Vec<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SurfacePoint {
    pub position: [f32; 3],
    pub triangle_index: usize,
    pub barycentric: [f32; 3],
    pub distance: f32,
}

#[allow(dead_code)]
pub struct ProximityQuery {
    config: ProximityQueryConfig,
    points: Vec<[f32; 3]>,
    triangles: Vec<[usize; 3]>,
}

fn dist_sq(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    (a[0]-b[0]).powi(2) + (a[1]-b[1]).powi(2) + (a[2]-b[2]).powi(2)
}

/// Project point `p` onto triangle (`v0`, `v1`, `v2`). Returns (closest point, barycentric).
fn closest_point_on_triangle(p: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let e0 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
    let e1 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
    let d = [v0[0]-p[0], v0[1]-p[1], v0[2]-p[2]];

    let a = e0[0]*e0[0] + e0[1]*e0[1] + e0[2]*e0[2];
    let b = e0[0]*e1[0] + e0[1]*e1[1] + e0[2]*e1[2];
    let c = e1[0]*e1[0] + e1[1]*e1[1] + e1[2]*e1[2];
    let d_dot_e0 = d[0]*e0[0] + d[1]*e0[1] + d[2]*e0[2];
    let d_dot_e1 = d[0]*e1[0] + d[1]*e1[1] + d[2]*e1[2];

    let det = a*c - b*b;
    let mut s = b*d_dot_e1 - c*d_dot_e0;
    let mut t = b*d_dot_e0 - a*d_dot_e1;

    if s + t <= det {
        if s < 0.0 {
            if t < 0.0 {
                s = 0.0; t = 0.0;
            } else {
                s = 0.0; t = (d_dot_e1 / c).clamp(0.0, 1.0);
            }
        } else if t < 0.0 {
            t = 0.0; s = (-d_dot_e0 / a).clamp(0.0, 1.0);
        }
    } else if s < 0.0 {
        s = 0.0; t = 1.0;
    } else if t < 0.0 {
        t = 0.0; s = 1.0;
    } else {
        let inv = if det.abs() > 1e-10 { 1.0 / det } else { 0.0 };
        s *= inv; t *= inv;
        let sum = s + t;
        if sum > 1.0 { s /= sum; t /= sum; }
    }

    let w = 1.0 - s - t;
    let cp = [
        v0[0]*w + v1[0]*s + v2[0]*t,
        v0[1]*w + v1[1]*s + v2[1]*t,
        v0[2]*w + v1[2]*s + v2[2]*t,
    ];
    (cp, [w, s, t])
}

#[allow(dead_code)]
pub fn default_proximity_query_config() -> ProximityQueryConfig {
    ProximityQueryConfig {
        max_k: 8,
        max_radius: f32::MAX,
        include_query_point: false,
    }
}

#[allow(dead_code)]
pub fn new_proximity_query(config: ProximityQueryConfig) -> ProximityQuery {
    ProximityQuery { config, points: Vec::new(), triangles: Vec::new() }
}

#[allow(dead_code)]
pub fn proximity_build(query: &mut ProximityQuery, points: Vec<[f32; 3]>, triangles: Vec<[usize; 3]>) {
    query.points = points;
    query.triangles = triangles;
}

#[allow(dead_code)]
pub fn proximity_k_nearest(query: &ProximityQuery, p: [f32; 3], k: usize) -> ProximityResult {
    let mut pairs: Vec<(f32, usize)> = query.points
        .iter()
        .enumerate()
        .map(|(i, pt)| (dist_sq(&p, pt).sqrt(), i))
        .collect();

    if !query.config.include_query_point {
        pairs.retain(|(d, _)| *d > 1e-8);
    }

    pairs.retain(|(d, _)| *d <= query.config.max_radius);
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    pairs.truncate(k.min(query.config.max_k));

    ProximityResult {
        indices: pairs.iter().map(|(_, i)| *i).collect(),
        distances: pairs.iter().map(|(d, _)| *d).collect(),
    }
}

#[allow(dead_code)]
pub fn proximity_closest_surface_point(query: &ProximityQuery, p: [f32; 3]) -> Option<SurfacePoint> {
    if query.triangles.is_empty() || query.points.is_empty() {
        return None;
    }
    let mut best_dist = f32::MAX;
    let mut best: Option<SurfacePoint> = None;

    for (ti, tri) in query.triangles.iter().enumerate() {
        let [a, b, c] = *tri;
        if a >= query.points.len() || b >= query.points.len() || c >= query.points.len() {
            continue;
        }
        let (cp, bary) = closest_point_on_triangle(p, query.points[a], query.points[b], query.points[c]);
        let d = dist_sq(&p, &cp).sqrt();
        if d < best_dist {
            best_dist = d;
            best = Some(SurfacePoint { position: cp, triangle_index: ti, barycentric: bary, distance: d });
        }
    }
    best
}

#[allow(dead_code)]
pub fn proximity_point_count(query: &ProximityQuery) -> usize {
    query.points.len()
}

#[allow(dead_code)]
pub fn proximity_query_to_json(query: &ProximityQuery) -> String {
    format!(
        "{{\"point_count\":{},\"triangle_count\":{},\"max_k\":{},\"max_radius\":{:.6}}}",
        query.points.len(), query.triangles.len(), query.config.max_k, query.config.max_radius
    )
}

#[allow(dead_code)]
pub fn proximity_clear(query: &mut ProximityQuery) {
    query.points.clear();
    query.triangles.clear();
}

#[allow(dead_code)]
pub fn proximity_radius_query(query: &ProximityQuery, p: [f32; 3], radius: f32) -> ProximityResult {
    let mut pairs: Vec<(f32, usize)> = query.points
        .iter()
        .enumerate()
        .filter_map(|(i, pt)| {
            let d = dist_sq(&p, pt).sqrt();
            if d <= radius { Some((d, i)) } else { None }
        })
        .collect();
    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    ProximityResult {
        indices: pairs.iter().map(|(_, i)| *i).collect(),
        distances: pairs.iter().map(|(d, _)| *d).collect(),
    }
}

#[allow(dead_code)]
pub fn proximity_distance_to_surface(query: &ProximityQuery, p: [f32; 3]) -> f32 {
    proximity_closest_surface_point(query, p).map(|sp| sp.distance).unwrap_or(f32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_query() -> ProximityQuery {
        let mut q = new_proximity_query(default_proximity_query_config());
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let tris = vec![[0, 1, 3], [1, 4, 3], [1, 2, 4]];
        proximity_build(&mut q, pts, tris);
        q
    }

    #[test]
    fn test_default_config() {
        let cfg = default_proximity_query_config();
        assert_eq!(cfg.max_k, 8);
    }

    #[test]
    fn test_point_count() {
        let q = make_query();
        assert_eq!(proximity_point_count(&q), 5);
    }

    #[test]
    fn test_k_nearest_count() {
        let q = make_query();
        let res = proximity_k_nearest(&q, [0.5, 0.5, 0.0], 3);
        assert_eq!(res.indices.len(), 3);
        assert_eq!(res.distances.len(), 3);
    }

    #[test]
    fn test_k_nearest_ordered() {
        let q = make_query();
        let res = proximity_k_nearest(&q, [0.0, 0.0, 0.0], 4);
        for i in 1..res.distances.len() {
            assert!(res.distances[i] >= res.distances[i-1]);
        }
    }

    #[test]
    fn test_radius_query() {
        let q = make_query();
        let res = proximity_radius_query(&q, [0.0, 0.0, 0.0], 1.1);
        // Points at dist 0(self), 1.0 (index 1 and 3) are within 1.1
        assert!(!res.indices.is_empty());
    }

    #[test]
    fn test_closest_surface_point() {
        let q = make_query();
        let sp = proximity_closest_surface_point(&q, [0.5, 0.5, 0.5]);
        assert!(sp.is_some());
        let sp = sp.unwrap();
        assert!(sp.distance >= 0.0);
    }

    #[test]
    fn test_distance_to_surface() {
        let q = make_query();
        let d = proximity_distance_to_surface(&q, [0.5, 0.0, 0.5]);
        assert!(d.is_finite());
        assert!(d >= 0.0);
    }

    #[test]
    fn test_to_json_fields() {
        let q = make_query();
        let json = proximity_query_to_json(&q);
        assert!(json.contains("point_count"));
        assert!(json.contains("max_k"));
        assert!(json.contains("triangle_count"));
    }

    #[test]
    fn test_clear() {
        let mut q = make_query();
        proximity_clear(&mut q);
        assert_eq!(proximity_point_count(&q), 0);
    }

    #[test]
    fn test_no_surface_when_empty() {
        let q = new_proximity_query(default_proximity_query_config());
        assert!(proximity_closest_surface_point(&q, [0.0, 0.0, 0.0]).is_none());
    }
}
