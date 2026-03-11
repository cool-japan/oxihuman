#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyDeformLayer {
    deltas: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn new_body_deform_layer(vertex_count: usize) -> BodyDeformLayer {
    BodyDeformLayer { deltas: vec![[0.0; 3]; vertex_count] }
}

#[allow(dead_code)]
pub fn set_deform_delta(layer: &mut BodyDeformLayer, idx: usize, delta: [f32; 3]) {
    if idx < layer.deltas.len() { layer.deltas[idx] = delta; }
}

#[allow(dead_code)]
pub fn deform_delta_at(layer: &BodyDeformLayer, idx: usize) -> [f32; 3] {
    if idx < layer.deltas.len() { layer.deltas[idx] } else { [0.0; 3] }
}

#[allow(dead_code)]
pub fn deform_vertex_count(layer: &BodyDeformLayer) -> usize { layer.deltas.len() }

#[allow(dead_code)]
pub fn apply_deform_layer(layer: &BodyDeformLayer, positions: &mut [[f32; 3]]) {
    for (i, pos) in positions.iter_mut().enumerate() {
        if i < layer.deltas.len() {
            pos[0] += layer.deltas[i][0];
            pos[1] += layer.deltas[i][1];
            pos[2] += layer.deltas[i][2];
        }
    }
}

#[allow(dead_code)]
pub fn deform_to_json(layer: &BodyDeformLayer) -> String {
    format!("{{\"vertex_count\":{}}}", layer.deltas.len())
}

#[allow(dead_code)]
pub fn deform_clear(layer: &mut BodyDeformLayer) {
    for d in layer.deltas.iter_mut() { *d = [0.0; 3]; }
}

#[allow(dead_code)]
pub fn deform_magnitude(layer: &BodyDeformLayer) -> f32 {
    layer.deltas.iter().map(|d| (d[0]*d[0] + d[1]*d[1] + d[2]*d[2]).sqrt()).fold(0.0_f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let l = new_body_deform_layer(10); assert_eq!(deform_vertex_count(&l), 10); }
    #[test] fn test_set_get() { let mut l = new_body_deform_layer(5); set_deform_delta(&mut l, 0, [1.0, 2.0, 3.0]); assert!((deform_delta_at(&l, 0)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_oob() { let l = new_body_deform_layer(2); assert!((deform_delta_at(&l, 10)[0]).abs() < 1e-6); }
    #[test] fn test_apply() { let mut l = new_body_deform_layer(1); set_deform_delta(&mut l, 0, [1.0, 0.0, 0.0]); let mut p = [[0.0, 0.0, 0.0]]; apply_deform_layer(&l, &mut p); assert!((p[0][0] - 1.0).abs() < 1e-6); }
    #[test] fn test_json() { let l = new_body_deform_layer(3); assert!(deform_to_json(&l).contains("vertex_count")); }
    #[test] fn test_clear() { let mut l = new_body_deform_layer(2); set_deform_delta(&mut l, 0, [1.0, 1.0, 1.0]); deform_clear(&mut l); assert!((deform_delta_at(&l, 0)[0]).abs() < 1e-6); }
    #[test] fn test_magnitude_zero() { let l = new_body_deform_layer(3); assert!(deform_magnitude(&l) < 1e-6); }
    #[test] fn test_magnitude() { let mut l = new_body_deform_layer(2); set_deform_delta(&mut l, 1, [3.0, 4.0, 0.0]); assert!((deform_magnitude(&l) - 5.0).abs() < 1e-4); }
    #[test] fn test_set_oob() { let mut l = new_body_deform_layer(1); set_deform_delta(&mut l, 5, [1.0, 0.0, 0.0]); assert_eq!(deform_vertex_count(&l), 1); }
    #[test] fn test_empty() { let l = new_body_deform_layer(0); assert_eq!(deform_vertex_count(&l), 0); }
}
