//! Heat diffusion / harmonic field computation on mesh surface.

#[allow(dead_code)]
pub struct HeatField {
    pub values: Vec<f32>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub struct HeatDiffuseConfig {
    pub iterations: u32,
    pub diffusion_rate: f32,
    pub boundary_fixed: bool,
}

#[allow(dead_code)]
pub struct HeatSource {
    pub vertex_index: usize,
    pub temperature: f32,
}

#[allow(dead_code)]
pub fn default_heat_config() -> HeatDiffuseConfig {
    HeatDiffuseConfig {
        iterations: 100,
        diffusion_rate: 0.25,
        boundary_fixed: true,
    }
}

#[allow(dead_code)]
pub fn new_heat_field(vertex_count: usize, initial: f32) -> HeatField {
    HeatField {
        values: vec![initial; vertex_count],
        vertex_count,
    }
}

#[allow(dead_code)]
pub fn set_heat_source(field: &mut HeatField, src: &HeatSource) {
    if src.vertex_index < field.vertex_count {
        field.values[src.vertex_index] = src.temperature;
    }
}

/// One Jacobi iteration of heat diffusion.
#[allow(dead_code)]
pub fn diffuse_step(field: &mut HeatField, adjacency: &[Vec<usize>], cfg: &HeatDiffuseConfig) {
    let old = field.values.clone();
    for (i, neighbors) in adjacency.iter().enumerate() {
        if !neighbors.is_empty() {
            let avg: f32 = neighbors.iter().map(|&j| old[j]).sum::<f32>() / neighbors.len() as f32;
            field.values[i] = old[i] + cfg.diffusion_rate * (avg - old[i]);
        }
    }
}

/// Full diffusion: apply sources, iterate, re-apply sources if boundary_fixed.
#[allow(dead_code)]
pub fn diffuse_heat(
    field: &mut HeatField,
    adjacency: &[Vec<usize>],
    sources: &[HeatSource],
    cfg: &HeatDiffuseConfig,
) {
    for src in sources {
        set_heat_source(field, src);
    }
    for _ in 0..cfg.iterations {
        diffuse_step(field, adjacency, cfg);
        if cfg.boundary_fixed {
            for src in sources {
                set_heat_source(field, src);
            }
        }
    }
}

/// Build per-vertex adjacency list from triangle index buffer.
#[allow(dead_code)]
pub fn build_adjacency(vertex_count: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    let triangles = indices.len() / 3;
    for t in 0..triangles {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        for &(u, v) in &[(a, b), (b, a), (b, c), (c, b), (a, c), (c, a)] {
            if u < vertex_count && v < vertex_count && !adj[u].contains(&v) {
                adj[u].push(v);
            }
        }
    }
    adj
}

/// Average delta between vertex and its neighbors.
#[allow(dead_code)]
pub fn heat_gradient(field: &HeatField, adjacency: &[Vec<usize>], vertex_idx: usize) -> f32 {
    let neighbors = &adjacency[vertex_idx];
    if neighbors.is_empty() {
        return 0.0;
    }
    let sum: f32 = neighbors
        .iter()
        .map(|&j| (field.values[j] - field.values[vertex_idx]).abs())
        .sum();
    sum / neighbors.len() as f32
}

/// Remap field values to [0, 1].
#[allow(dead_code)]
pub fn normalize_field(field: &mut HeatField) {
    let mn = heat_field_min(field);
    let mx = heat_field_max(field);
    let range = mx - mn;
    if range > 1e-10 {
        for v in field.values.iter_mut() {
            *v = (*v - mn) / range;
        }
    }
}

#[allow(dead_code)]
pub fn heat_field_min(field: &HeatField) -> f32 {
    field.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn heat_field_max(field: &HeatField) -> f32 {
    field
        .values
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Cool-to-warm: blue (0) -> cyan -> green -> yellow -> red (1).
#[allow(dead_code)]
pub fn heat_to_color(value: f32) -> [f32; 3] {
    let t = value.clamp(0.0, 1.0);
    // Simple blue->red linear blend
    [t, 0.0, 1.0 - t]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Approximate geodesic distance from source vertex via heat method.
#[allow(dead_code)]
pub fn geodesic_heat_distance(
    positions: &[[f32; 3]],
    indices: &[u32],
    source_vertex: usize,
    iterations: u32,
) -> Vec<f32> {
    let n = positions.len();
    let adj = build_adjacency(n, indices);
    let cfg = HeatDiffuseConfig {
        iterations,
        diffusion_rate: 0.25,
        boundary_fixed: false,
    };
    let mut field = new_heat_field(n, 0.0);
    let src = HeatSource {
        vertex_index: source_vertex,
        temperature: 1.0,
    };
    diffuse_heat(&mut field, &adj, &[src], &cfg);

    // Convert heat values to approximate distances
    let heat_max = heat_field_max(&field).max(1e-10);
    field
        .values
        .iter()
        .enumerate()
        .map(|(i, &h)| {
            let heat_dist = (1.0 - h / heat_max).max(0.0);
            // Scale by average edge length as approximation
            let edge_len: f32 = if !adj[i].is_empty() {
                adj[i]
                    .iter()
                    .map(|&j| dist3(positions[i], positions[j]))
                    .sum::<f32>()
                    / adj[i].len() as f32
            } else {
                dist3(positions[i], positions[source_vertex])
            };
            heat_dist * edge_len * n as f32
        })
        .collect()
}

/// Return boolean mask: true where field value >= threshold.
#[allow(dead_code)]
pub fn threshold_field(field: &HeatField, threshold: f32) -> Vec<bool> {
    field.values.iter().map(|&v| v >= threshold).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_square_indices() -> Vec<u32> {
        // 4 vertices, 2 triangles: (0,1,2), (0,2,3)
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_new_heat_field_count() {
        let f = new_heat_field(10, 0.5);
        assert_eq!(f.vertex_count, 10);
        assert_eq!(f.values.len(), 10);
    }

    #[test]
    fn test_new_heat_field_initial_value() {
        let f = new_heat_field(5, 2.5);
        assert!(f.values.iter().all(|&v| (v - 2.5).abs() < 1e-6));
    }

    #[test]
    fn test_build_adjacency_basic() {
        let adj = build_adjacency(4, &simple_square_indices());
        assert_eq!(adj.len(), 4);
        // Vertex 0 should be adjacent to 1, 2, 3
        assert!(adj[0].contains(&1));
        assert!(adj[0].contains(&2));
        assert!(adj[0].contains(&3));
    }

    #[test]
    fn test_build_adjacency_symmetry() {
        let adj = build_adjacency(4, &simple_square_indices());
        for (i, neighbors) in adj.iter().enumerate() {
            for &j in neighbors {
                assert!(
                    adj[j].contains(&i),
                    "Adjacency should be symmetric: {i} -> {j} but not {j} -> {i}"
                );
            }
        }
    }

    #[test]
    fn test_set_heat_source() {
        let mut f = new_heat_field(5, 0.0);
        set_heat_source(
            &mut f,
            &HeatSource {
                vertex_index: 2,
                temperature: 1.0,
            },
        );
        assert!((f.values[2] - 1.0).abs() < 1e-6);
        assert!((f.values[0]).abs() < 1e-6);
    }

    #[test]
    fn test_set_heat_source_out_of_bounds() {
        let mut f = new_heat_field(3, 0.0);
        // Should not panic
        set_heat_source(
            &mut f,
            &HeatSource {
                vertex_index: 100,
                temperature: 999.0,
            },
        );
        assert!(f.values.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn test_heat_field_min_max() {
        let mut f = new_heat_field(4, 0.0);
        f.values[0] = -1.0;
        f.values[3] = 5.0;
        assert!((heat_field_min(&f) - (-1.0)).abs() < 1e-6);
        assert!((heat_field_max(&f) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_field() {
        let mut f = new_heat_field(4, 0.0);
        f.values = vec![0.0, 2.0, 4.0, 8.0];
        normalize_field(&mut f);
        assert!((f.values[0] - 0.0).abs() < 1e-6);
        assert!((f.values[3] - 1.0).abs() < 1e-6);
        assert!((f.values[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_field_uniform() {
        let mut f = new_heat_field(3, 3.0);
        normalize_field(&mut f); // Should not panic
        assert_eq!(f.values.len(), 3);
    }

    #[test]
    fn test_heat_to_color_blue_at_zero() {
        let c = heat_to_color(0.0);
        assert!((c[0]).abs() < 1e-6); // R = 0
        assert!((c[2] - 1.0).abs() < 1e-6); // B = 1
    }

    #[test]
    fn test_heat_to_color_red_at_one() {
        let c = heat_to_color(1.0);
        assert!((c[0] - 1.0).abs() < 1e-6); // R = 1
        assert!((c[2]).abs() < 1e-6); // B = 0
    }

    #[test]
    fn test_threshold_field() {
        let mut f = new_heat_field(4, 0.0);
        f.values = vec![0.1, 0.5, 0.9, 0.3];
        let mask = threshold_field(&f, 0.4);
        assert!(!mask[0]);
        assert!(mask[1]);
        assert!(mask[2]);
        assert!(!mask[3]);
    }

    #[test]
    fn test_diffuse_heat_propagates() {
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let adj = build_adjacency(4, &idx);
        let cfg = HeatDiffuseConfig {
            iterations: 50,
            diffusion_rate: 0.25,
            boundary_fixed: true,
        };
        let mut field = new_heat_field(4, 0.0);
        let src = HeatSource {
            vertex_index: 0,
            temperature: 1.0,
        };
        diffuse_heat(&mut field, &adj, &[src], &cfg);
        // Heat should propagate from vertex 0 to others
        assert!(field.values[0] > 0.5, "Source should stay hot");
        assert!(field.values[1] > 0.0, "Neighbor should receive heat");
        assert!(field.values[3] > 0.0, "Far neighbor should receive heat");
    }

    #[test]
    fn test_default_heat_config() {
        let cfg = default_heat_config();
        assert!(cfg.iterations > 0);
        assert!(cfg.diffusion_rate > 0.0);
    }

    #[test]
    fn test_heat_gradient_uniform() {
        let f = new_heat_field(4, 1.0);
        let adj = build_adjacency(4, &simple_square_indices());
        let grad = heat_gradient(&f, &adj, 0);
        assert!((grad).abs() < 1e-6, "Uniform field has zero gradient");
    }

    #[test]
    fn test_geodesic_heat_distance_source_small() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let dists = geodesic_heat_distance(&positions, &indices, 0, 100);
        assert_eq!(dists.len(), 4);
        // Source should have smallest distance
        assert!(dists[0] <= dists[1] + 1e-3);
    }

    #[test]
    fn test_build_adjacency_empty() {
        let adj = build_adjacency(5, &[]);
        assert_eq!(adj.len(), 5);
        assert!(adj.iter().all(|n| n.is_empty()));
    }
}
