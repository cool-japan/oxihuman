#![allow(dead_code)]

//! GPU instanced draw data (transform + per-instance data).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceData {
    pub transform: [[f32; 4]; 4],
    pub color_tint: [f32; 4],
    pub custom: [f32; 4],
}

#[allow(dead_code)]
pub fn identity_transform() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstancedDraw {
    pub mesh_id: u32,
    pub material_id: u32,
    pub instances: Vec<InstanceData>,
    pub max_instances: u32,
}

#[allow(dead_code)]
pub fn new_instanced_draw(mesh_id: u32, material_id: u32, max_instances: u32) -> InstancedDraw {
    InstancedDraw {
        mesh_id,
        material_id,
        instances: Vec::new(),
        max_instances,
    }
}

#[allow(dead_code)]
pub fn id_add_instance(draw: &mut InstancedDraw, transform: [[f32; 4]; 4], color_tint: [f32; 4], custom: [f32; 4]) -> bool {
    if draw.instances.len() >= draw.max_instances as usize {
        return false;
    }
    draw.instances.push(InstanceData { transform, color_tint, custom });
    true
}

#[allow(dead_code)]
pub fn id_instance_count(draw: &InstancedDraw) -> usize {
    draw.instances.len()
}

#[allow(dead_code)]
pub fn id_is_full(draw: &InstancedDraw) -> bool {
    draw.instances.len() >= draw.max_instances as usize
}

#[allow(dead_code)]
pub fn id_clear(draw: &mut InstancedDraw) {
    draw.instances.clear();
}

#[allow(dead_code)]
pub fn id_remove_last(draw: &mut InstancedDraw) {
    draw.instances.pop();
}

#[allow(dead_code)]
pub fn id_capacity_remaining(draw: &InstancedDraw) -> u32 {
    draw.max_instances.saturating_sub(draw.instances.len() as u32)
}

#[allow(dead_code)]
pub fn id_update_tint(draw: &mut InstancedDraw, index: usize, tint: [f32; 4]) {
    if let Some(inst) = draw.instances.get_mut(index) {
        inst.color_tint = tint;
    }
}

#[allow(dead_code)]
pub fn id_to_json(draw: &InstancedDraw) -> String {
    format!(
        "{{\"mesh_id\":{},\"material_id\":{},\"instance_count\":{},\"max_instances\":{}}}",
        draw.mesh_id,
        draw.material_id,
        draw.instances.len(),
        draw.max_instances
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_instanced_draw() {
        let d = new_instanced_draw(0, 1, 100);
        assert_eq!(id_instance_count(&d), 0);
    }

    #[test]
    fn test_add_instance() {
        let mut d = new_instanced_draw(0, 1, 100);
        let added = id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        assert!(added);
        assert_eq!(id_instance_count(&d), 1);
    }

    #[test]
    fn test_is_full() {
        let mut d = new_instanced_draw(0, 1, 1);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        assert!(id_is_full(&d));
    }

    #[test]
    fn test_add_when_full_fails() {
        let mut d = new_instanced_draw(0, 1, 1);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        let added = id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        assert!(!added);
    }

    #[test]
    fn test_clear() {
        let mut d = new_instanced_draw(0, 1, 10);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        id_clear(&mut d);
        assert_eq!(id_instance_count(&d), 0);
    }

    #[test]
    fn test_remove_last() {
        let mut d = new_instanced_draw(0, 1, 10);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        id_add_instance(&mut d, identity_transform(), [0.5; 4], [0.0; 4]);
        id_remove_last(&mut d);
        assert_eq!(id_instance_count(&d), 1);
    }

    #[test]
    fn test_capacity_remaining() {
        let mut d = new_instanced_draw(0, 1, 5);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        assert_eq!(id_capacity_remaining(&d), 4);
    }

    #[test]
    fn test_update_tint() {
        let mut d = new_instanced_draw(0, 1, 10);
        id_add_instance(&mut d, identity_transform(), [1.0; 4], [0.0; 4]);
        id_update_tint(&mut d, 0, [0.5; 4]);
        assert!((d.instances[0].color_tint[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let d = new_instanced_draw(2, 3, 50);
        let json = id_to_json(&d);
        assert!(json.contains("mesh_id"));
    }

    #[test]
    fn test_identity_transform() {
        let t = identity_transform();
        assert!((t[0][0] - 1.0).abs() < 1e-6);
        assert!((t[1][2] - 0.0).abs() < 1e-6);
    }
}
