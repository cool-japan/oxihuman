#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DebugShape {
    Sphere { center: [f32; 3], radius: f32 },
    Box { center: [f32; 3], half_extents: [f32; 3] },
    Capsule { start: [f32; 3], end: [f32; 3], radius: f32 },
    Contact { point: [f32; 3], normal: [f32; 3] },
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PhysDebugDraw {
    shapes: Vec<DebugShape>,
}

#[allow(dead_code)]
pub fn new_phys_debug_draw() -> PhysDebugDraw {
    PhysDebugDraw {
        shapes: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn draw_sphere_dd(dd: &mut PhysDebugDraw, center: [f32; 3], radius: f32) {
    dd.shapes.push(DebugShape::Sphere { center, radius });
}

#[allow(dead_code)]
pub fn draw_box_dd(dd: &mut PhysDebugDraw, center: [f32; 3], half_extents: [f32; 3]) {
    dd.shapes.push(DebugShape::Box { center, half_extents });
}

#[allow(dead_code)]
pub fn draw_capsule_dd(dd: &mut PhysDebugDraw, start: [f32; 3], end: [f32; 3], radius: f32) {
    dd.shapes.push(DebugShape::Capsule { start, end, radius });
}

#[allow(dead_code)]
pub fn draw_contact_dd(dd: &mut PhysDebugDraw, point: [f32; 3], normal: [f32; 3]) {
    dd.shapes.push(DebugShape::Contact { point, normal });
}

#[allow(dead_code)]
pub fn debug_shape_count_dd(dd: &PhysDebugDraw) -> usize {
    dd.shapes.len()
}

#[allow(dead_code)]
pub fn clear_phys_debug_draw(dd: &mut PhysDebugDraw) {
    dd.shapes.clear();
}

#[allow(dead_code)]
pub fn debug_draw_to_json(dd: &PhysDebugDraw) -> String {
    let entries: Vec<String> = dd.shapes.iter().map(|s| match s {
        DebugShape::Sphere { center, radius } => {
            format!("{{\"type\":\"sphere\",\"center\":[{},{},{}],\"radius\":{}}}", center[0], center[1], center[2], radius)
        }
        DebugShape::Box { center, half_extents } => {
            format!("{{\"type\":\"box\",\"center\":[{},{},{}],\"half_extents\":[{},{},{}]}}", center[0], center[1], center[2], half_extents[0], half_extents[1], half_extents[2])
        }
        DebugShape::Capsule { start, end, radius } => {
            format!("{{\"type\":\"capsule\",\"start\":[{},{},{}],\"end\":[{},{},{}],\"radius\":{}}}", start[0], start[1], start[2], end[0], end[1], end[2], radius)
        }
        DebugShape::Contact { point, normal } => {
            format!("{{\"type\":\"contact\",\"point\":[{},{},{}],\"normal\":[{},{},{}]}}", point[0], point[1], point[2], normal[0], normal[1], normal[2])
        }
    }).collect();
    format!("[{}]", entries.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dd = new_phys_debug_draw();
        assert_eq!(debug_shape_count_dd(&dd), 0);
    }

    #[test]
    fn test_draw_sphere() {
        let mut dd = new_phys_debug_draw();
        draw_sphere_dd(&mut dd, [0.0; 3], 1.0);
        assert_eq!(debug_shape_count_dd(&dd), 1);
    }

    #[test]
    fn test_draw_box() {
        let mut dd = new_phys_debug_draw();
        draw_box_dd(&mut dd, [0.0; 3], [1.0, 1.0, 1.0]);
        assert_eq!(debug_shape_count_dd(&dd), 1);
    }

    #[test]
    fn test_draw_capsule() {
        let mut dd = new_phys_debug_draw();
        draw_capsule_dd(&mut dd, [0.0; 3], [0.0, 1.0, 0.0], 0.5);
        assert_eq!(debug_shape_count_dd(&dd), 1);
    }

    #[test]
    fn test_draw_contact() {
        let mut dd = new_phys_debug_draw();
        draw_contact_dd(&mut dd, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(debug_shape_count_dd(&dd), 1);
    }

    #[test]
    fn test_clear() {
        let mut dd = new_phys_debug_draw();
        draw_sphere_dd(&mut dd, [0.0; 3], 1.0);
        clear_phys_debug_draw(&mut dd);
        assert_eq!(debug_shape_count_dd(&dd), 0);
    }

    #[test]
    fn test_multiple_shapes() {
        let mut dd = new_phys_debug_draw();
        draw_sphere_dd(&mut dd, [0.0; 3], 1.0);
        draw_box_dd(&mut dd, [1.0; 3], [0.5; 3]);
        assert_eq!(debug_shape_count_dd(&dd), 2);
    }

    #[test]
    fn test_to_json_empty() {
        let dd = new_phys_debug_draw();
        assert_eq!(debug_draw_to_json(&dd), "[]");
    }

    #[test]
    fn test_to_json_sphere() {
        let mut dd = new_phys_debug_draw();
        draw_sphere_dd(&mut dd, [0.0; 3], 1.0);
        let j = debug_draw_to_json(&dd);
        assert!(j.contains("\"type\":\"sphere\""));
    }

    #[test]
    fn test_to_json_contact() {
        let mut dd = new_phys_debug_draw();
        draw_contact_dd(&mut dd, [0.0; 3], [0.0, 1.0, 0.0]);
        let j = debug_draw_to_json(&dd);
        assert!(j.contains("\"type\":\"contact\""));
    }
}
