#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderDebugBox {
    center: [f32; 3],
    extents: [f32; 3],
    color: [f32; 4],
    visible: bool,
}

#[allow(dead_code)]
pub fn new_debug_box(center: [f32; 3], extents: [f32; 3]) -> RenderDebugBox {
    RenderDebugBox { center, extents, color: [1.0, 1.0, 1.0, 1.0], visible: true }
}

#[allow(dead_code)]
pub fn box_center_rdb(b: &RenderDebugBox) -> [f32; 3] { b.center }

#[allow(dead_code)]
pub fn box_extents_rdb(b: &RenderDebugBox) -> [f32; 3] { b.extents }

#[allow(dead_code)]
pub fn box_color_rdb(b: &RenderDebugBox) -> [f32; 4] { b.color }

#[allow(dead_code)]
pub fn box_to_vertices(b: &RenderDebugBox) -> Vec<[f32; 3]> {
    let c = b.center; let e = b.extents;
    vec![
        [c[0]-e[0], c[1]-e[1], c[2]-e[2]], [c[0]+e[0], c[1]-e[1], c[2]-e[2]],
        [c[0]-e[0], c[1]+e[1], c[2]-e[2]], [c[0]+e[0], c[1]+e[1], c[2]-e[2]],
        [c[0]-e[0], c[1]-e[1], c[2]+e[2]], [c[0]+e[0], c[1]-e[1], c[2]+e[2]],
        [c[0]-e[0], c[1]+e[1], c[2]+e[2]], [c[0]+e[0], c[1]+e[1], c[2]+e[2]],
    ]
}

#[allow(dead_code)]
pub fn box_to_json_rdb(b: &RenderDebugBox) -> String {
    format!("{{\"center\":[{:.2},{:.2},{:.2}]}}", b.center[0], b.center[1], b.center[2])
}

#[allow(dead_code)]
pub fn box_is_visible_rdb(b: &RenderDebugBox) -> bool { b.visible }

#[allow(dead_code)]
pub fn box_transform(b: &mut RenderDebugBox, offset: [f32; 3]) {
    b.center[0] += offset[0]; b.center[1] += offset[1]; b.center[2] += offset[2];
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let b = new_debug_box([0.0; 3], [1.0; 3]); assert!(box_is_visible_rdb(&b)); }
    #[test] fn test_center() { let b = new_debug_box([1.0, 2.0, 3.0], [1.0; 3]); assert!((box_center_rdb(&b)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_extents() { let b = new_debug_box([0.0; 3], [2.0, 3.0, 4.0]); assert!((box_extents_rdb(&b)[1] - 3.0).abs() < 1e-6); }
    #[test] fn test_color() { let b = new_debug_box([0.0; 3], [1.0; 3]); assert!((box_color_rdb(&b)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_vertices() { let b = new_debug_box([0.0; 3], [1.0; 3]); assert_eq!(box_to_vertices(&b).len(), 8); }
    #[test] fn test_json() { let b = new_debug_box([0.0; 3], [1.0; 3]); assert!(box_to_json_rdb(&b).contains("center")); }
    #[test] fn test_visible() { let b = new_debug_box([0.0; 3], [1.0; 3]); assert!(box_is_visible_rdb(&b)); }
    #[test] fn test_transform() { let mut b = new_debug_box([0.0; 3], [1.0; 3]); box_transform(&mut b, [1.0, 0.0, 0.0]); assert!((box_center_rdb(&b)[0] - 1.0).abs() < 1e-6); }
    #[test] fn test_vertices_offset() { let b = new_debug_box([1.0, 0.0, 0.0], [1.0; 3]); let v = box_to_vertices(&b); assert!((v[0][0]).abs() < 1e-6); }
    #[test] fn test_transform_twice() { let mut b = new_debug_box([0.0; 3], [1.0; 3]); box_transform(&mut b, [1.0, 1.0, 1.0]); box_transform(&mut b, [1.0, 1.0, 1.0]); assert!((box_center_rdb(&b)[0] - 2.0).abs() < 1e-6); }
}
