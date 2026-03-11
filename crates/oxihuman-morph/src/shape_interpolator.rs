#![allow(dead_code)]

//! Multi-shape interpolation with barycentric coords.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapePoint {
    pub name: String,
    pub coords: [f32; 2],
    pub vertices: Vec<[f32; 3]>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShapeInterpolator {
    pub shapes: Vec<ShapePoint>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_shape_interpolator(vertex_count: usize) -> ShapeInterpolator {
    ShapeInterpolator {
        shapes: Vec::new(),
        vertex_count,
    }
}

#[allow(dead_code)]
pub fn si_add_shape(interp: &mut ShapeInterpolator, name: &str, coords: [f32; 2], vertices: Vec<[f32; 3]>) {
    interp.shapes.push(ShapePoint {
        name: name.to_string(),
        coords,
        vertices,
    });
}

fn barycentric_weights(shapes: &[ShapePoint], query: [f32; 2]) -> Vec<f32> {
    if shapes.is_empty() {
        return Vec::new();
    }
    let inv_dists: Vec<f32> = shapes
        .iter()
        .map(|s| {
            let dx = s.coords[0] - query[0];
            let dy = s.coords[1] - query[1];
            let d2 = dx * dx + dy * dy;
            if d2 < 1e-12 { 1e12 } else { 1.0 / d2 }
        })
        .collect();
    let total: f32 = inv_dists.iter().sum();
    if total < 1e-12 {
        let n = shapes.len() as f32;
        return vec![1.0 / n; shapes.len()];
    }
    inv_dists.iter().map(|&w| w / total).collect()
}

#[allow(dead_code)]
pub fn si_interpolate(interp: &ShapeInterpolator, coords: [f32; 2]) -> Vec<[f32; 3]> {
    if interp.shapes.is_empty() {
        return vec![[0.0; 3]; interp.vertex_count];
    }
    let weights = barycentric_weights(&interp.shapes, coords);
    let mut result = vec![[0.0_f32; 3]; interp.vertex_count];
    for (shape, &w) in interp.shapes.iter().zip(weights.iter()) {
        for (i, r) in result.iter_mut().enumerate() {
            if let Some(v) = shape.vertices.get(i) {
                for k in 0..3 {
                    r[k] += v[k] * w;
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn si_shape_count(interp: &ShapeInterpolator) -> usize {
    interp.shapes.len()
}

#[allow(dead_code)]
pub fn si_remove_shape(interp: &mut ShapeInterpolator, name: &str) {
    interp.shapes.retain(|s| s.name != name);
}

#[allow(dead_code)]
pub fn si_clear(interp: &mut ShapeInterpolator) {
    interp.shapes.clear();
}

#[allow(dead_code)]
pub fn si_has_shape(interp: &ShapeInterpolator, name: &str) -> bool {
    interp.shapes.iter().any(|s| s.name == name)
}

#[allow(dead_code)]
pub fn si_to_json(interp: &ShapeInterpolator) -> String {
    format!(
        "{{\"shape_count\":{},\"vertex_count\":{}}}",
        interp.shapes.len(),
        interp.vertex_count
    )
}

#[allow(dead_code)]
pub fn si_nearest_shape(interp: &ShapeInterpolator, coords: [f32; 2]) -> Option<&str> {
    interp
        .shapes
        .iter()
        .min_by(|a, b| {
            let da = (a.coords[0] - coords[0]).powi(2) + (a.coords[1] - coords[1]).powi(2);
            let db = (b.coords[0] - coords[0]).powi(2) + (b.coords[1] - coords[1]).powi(2);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|s| s.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_interpolator() {
        let si = new_shape_interpolator(100);
        assert_eq!(si_shape_count(&si), 0);
    }

    #[test]
    fn test_add_shape() {
        let mut si = new_shape_interpolator(4);
        si_add_shape(&mut si, "neutral", [0.0, 0.0], vec![[0.0; 3]; 4]);
        assert_eq!(si_shape_count(&si), 1);
    }

    #[test]
    fn test_interpolate_single_shape() {
        let mut si = new_shape_interpolator(1);
        si_add_shape(&mut si, "a", [0.0, 0.0], vec![[1.0, 2.0, 3.0]]);
        let result = si_interpolate(&si, [0.0, 0.0]);
        assert!((result[0][0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_interpolate_two_shapes() {
        let mut si = new_shape_interpolator(1);
        si_add_shape(&mut si, "a", [-1.0, 0.0], vec![[0.0, 0.0, 0.0]]);
        si_add_shape(&mut si, "b", [1.0, 0.0], vec![[2.0, 0.0, 0.0]]);
        let result = si_interpolate(&si, [0.0, 0.0]);
        assert!(result[0][0] >= 0.0 && result[0][0] <= 2.0);
    }

    #[test]
    fn test_remove_shape() {
        let mut si = new_shape_interpolator(4);
        si_add_shape(&mut si, "x", [0.0, 0.0], vec![]);
        si_remove_shape(&mut si, "x");
        assert!(!si_has_shape(&si, "x"));
    }

    #[test]
    fn test_clear() {
        let mut si = new_shape_interpolator(4);
        si_add_shape(&mut si, "a", [0.0, 0.0], vec![]);
        si_clear(&mut si);
        assert_eq!(si_shape_count(&si), 0);
    }

    #[test]
    fn test_has_shape() {
        let mut si = new_shape_interpolator(4);
        si_add_shape(&mut si, "happy", [1.0, 0.0], vec![]);
        assert!(si_has_shape(&si, "happy"));
        assert!(!si_has_shape(&si, "sad"));
    }

    #[test]
    fn test_nearest_shape() {
        let mut si = new_shape_interpolator(1);
        si_add_shape(&mut si, "a", [0.0, 0.0], vec![]);
        si_add_shape(&mut si, "b", [10.0, 0.0], vec![]);
        let nearest = si_nearest_shape(&si, [0.1, 0.0]);
        assert_eq!(nearest, Some("a"));
    }

    #[test]
    fn test_interpolate_empty() {
        let si = new_shape_interpolator(3);
        let result = si_interpolate(&si, [0.0, 0.0]);
        assert_eq!(result.len(), 3);
        assert!((result[0][0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let si = new_shape_interpolator(50);
        let json = si_to_json(&si);
        assert!(json.contains("vertex_count"));
    }
}
