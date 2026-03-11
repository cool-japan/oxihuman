//! Surface flow / direction field on a mesh.
#![allow(dead_code)]

/// A single flow vector at a vertex.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct FlowVector {
    pub direction: [f32; 3],
    pub magnitude: f32,
}

/// A flow field stored per vertex.
#[allow(dead_code)]
pub struct FlowField {
    pub vectors: Vec<FlowVector>,
}

/// Create a new flow field for `n` vertices, initialized to zero.
#[allow(dead_code)]
pub fn new_flow_field(n: usize) -> FlowField {
    FlowField {
        vectors: vec![
            FlowVector {
                direction: [1.0, 0.0, 0.0],
                magnitude: 0.0
            };
            n
        ],
    }
}

/// Set the flow vector at vertex index `i`.
#[allow(dead_code)]
pub fn set_flow_vector(field: &mut FlowField, i: usize, dir: [f32; 3], magnitude: f32) {
    if i < field.vectors.len() {
        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        let d = if len < 1e-10 {
            [1.0, 0.0, 0.0]
        } else {
            [dir[0] / len, dir[1] / len, dir[2] / len]
        };
        field.vectors[i] = FlowVector {
            direction: d,
            magnitude,
        };
    }
}

/// Get the flow vector at vertex index `i`.
#[allow(dead_code)]
pub fn flow_at_vertex(field: &FlowField, i: usize) -> FlowVector {
    if i < field.vectors.len() {
        field.vectors[i]
    } else {
        FlowVector {
            direction: [1.0, 0.0, 0.0],
            magnitude: 0.0,
        }
    }
}

/// Advect a point on the surface by following the flow field for `dt` time.
/// Simple Euler integration: just moves the point along the nearest vertex's flow direction.
#[allow(dead_code)]
pub fn advect_point_on_surface(
    point: [f32; 3],
    field: &FlowField,
    positions: &[[f32; 3]],
    dt: f32,
) -> [f32; 3] {
    // Find nearest vertex
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, &p) in positions.iter().enumerate() {
        let dx = point[0] - p[0];
        let dy = point[1] - p[1];
        let dz = point[2] - p[2];
        let d = dx * dx + dy * dy + dz * dz;
        if d < best_dist {
            best_dist = d;
            best_idx = i;
        }
    }
    let fv = flow_at_vertex(field, best_idx);
    [
        point[0] + fv.direction[0] * fv.magnitude * dt,
        point[1] + fv.direction[1] * fv.magnitude * dt,
        point[2] + fv.direction[2] * fv.magnitude * dt,
    ]
}

/// Smooth the flow field by averaging neighboring vertex directions.
#[allow(dead_code)]
pub fn smooth_flow_field(field: &FlowField, iterations: usize) -> FlowField {
    let mut result = FlowField {
        vectors: field.vectors.clone(),
    };
    let n = result.vectors.len();
    for _ in 0..iterations {
        let prev = result.vectors.clone();
        for i in 0..n {
            let left = if i == 0 { n - 1 } else { i - 1 };
            let right = (i + 1) % n;
            let dx =
                (prev[left].direction[0] + prev[i].direction[0] + prev[right].direction[0]) / 3.0;
            let dy =
                (prev[left].direction[1] + prev[i].direction[1] + prev[right].direction[1]) / 3.0;
            let dz =
                (prev[left].direction[2] + prev[i].direction[2] + prev[right].direction[2]) / 3.0;
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-10);
            result.vectors[i].direction = [dx / len, dy / len, dz / len];
            result.vectors[i].magnitude =
                (prev[left].magnitude + prev[i].magnitude + prev[right].magnitude) / 3.0;
        }
    }
    result
}

/// Get the flow magnitude at vertex index `i`.
#[allow(dead_code)]
pub fn flow_magnitude_at(field: &FlowField, i: usize) -> f32 {
    flow_at_vertex(field, i).magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow_field_size() {
        let f = new_flow_field(5);
        assert_eq!(f.vectors.len(), 5);
    }

    #[test]
    fn test_set_and_get_flow_vector() {
        let mut f = new_flow_field(3);
        set_flow_vector(&mut f, 1, [1.0, 0.0, 0.0], 2.5);
        let v = flow_at_vertex(&f, 1);
        assert!((v.magnitude - 2.5).abs() < 1e-5);
    }

    #[test]
    fn test_flow_direction_normalized() {
        let mut f = new_flow_field(3);
        set_flow_vector(&mut f, 0, [3.0, 0.0, 0.0], 1.0);
        let v = flow_at_vertex(&f, 0);
        assert!((v.direction[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_flow_magnitude_at() {
        let mut f = new_flow_field(2);
        set_flow_vector(&mut f, 0, [0.0, 1.0, 0.0], 3.0);
        assert!((flow_magnitude_at(&f, 0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_advect_point_moves() {
        let mut f = new_flow_field(1);
        set_flow_vector(&mut f, 0, [1.0, 0.0, 0.0], 1.0);
        let pos = vec![[0.0f32, 0.0, 0.0]];
        let p = advect_point_on_surface([0.0, 0.0, 0.0], &f, &pos, 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_flow_field_count() {
        let f = new_flow_field(4);
        let s = smooth_flow_field(&f, 1);
        assert_eq!(s.vectors.len(), 4);
    }

    #[test]
    fn test_flow_at_vertex_oob() {
        let f = new_flow_field(2);
        let v = flow_at_vertex(&f, 100);
        assert!((v.magnitude).abs() < 1e-5);
    }

    #[test]
    fn test_flow_vector_default_magnitude() {
        let f = new_flow_field(3);
        for v in &f.vectors {
            assert!((v.magnitude).abs() < 1e-5);
        }
    }

    #[test]
    fn test_smooth_flow_preserves_direction_unit() {
        let mut f = new_flow_field(3);
        set_flow_vector(&mut f, 0, [1.0, 0.0, 0.0], 1.0);
        set_flow_vector(&mut f, 1, [1.0, 0.0, 0.0], 1.0);
        set_flow_vector(&mut f, 2, [1.0, 0.0, 0.0], 1.0);
        let s = smooth_flow_field(&f, 2);
        let v = s.vectors[1];
        let len = (v.direction[0] * v.direction[0]
            + v.direction[1] * v.direction[1]
            + v.direction[2] * v.direction[2])
            .sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }
}
