// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Instanced rendering data management.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstanceDataV2 {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstancedRendererV2 {
    pub instances: Vec<InstanceDataV2>,
    pub mesh_id: u32,
}

#[allow(dead_code)]
pub fn new_instanced_renderer_v2(mesh_id: u32) -> InstancedRendererV2 {
    InstancedRendererV2 { instances: Vec::new(), mesh_id }
}

#[allow(dead_code)]
pub fn irv2_add_instance(
    renderer: &mut InstancedRendererV2,
    transform: [[f32; 4]; 4],
    color: [f32; 4],
) {
    renderer.instances.push(InstanceDataV2 { transform, color });
}

#[allow(dead_code)]
pub fn irv2_instance_count(renderer: &InstancedRendererV2) -> usize {
    renderer.instances.len()
}

#[allow(dead_code)]
pub fn irv2_clear(renderer: &mut InstancedRendererV2) {
    renderer.instances.clear();
}

#[allow(dead_code)]
pub fn irv2_mesh_id(renderer: &InstancedRendererV2) -> u32 {
    renderer.mesh_id
}

#[allow(dead_code)]
pub fn irv2_avg_color(renderer: &InstancedRendererV2) -> [f32; 4] {
    if renderer.instances.is_empty() {
        return [0.0; 4];
    }
    let n = renderer.instances.len() as f32;
    let mut sum = [0.0f32; 4];
    for inst in &renderer.instances {
        for (i, s) in sum.iter_mut().enumerate() {
            *s += inst.color[i];
        }
    }
    [sum[0] / n, sum[1] / n, sum[2] / n, sum[3] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_new_instance_count_zero() {
        let r = new_instanced_renderer_v2(5);
        assert_eq!(irv2_instance_count(&r), 0);
    }

    #[test]
    fn test_add_instance() {
        let mut r = new_instanced_renderer_v2(5);
        irv2_add_instance(&mut r, identity(), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(irv2_instance_count(&r), 1);
    }

    #[test]
    fn test_clear() {
        let mut r = new_instanced_renderer_v2(5);
        irv2_add_instance(&mut r, identity(), [1.0, 0.0, 0.0, 1.0]);
        irv2_clear(&mut r);
        assert_eq!(irv2_instance_count(&r), 0);
    }

    #[test]
    fn test_mesh_id() {
        let r = new_instanced_renderer_v2(42);
        assert_eq!(irv2_mesh_id(&r), 42);
    }

    #[test]
    fn test_avg_color_empty() {
        let r = new_instanced_renderer_v2(1);
        assert_eq!(irv2_avg_color(&r), [0.0; 4]);
    }

    #[test]
    fn test_avg_color_single() {
        let mut r = new_instanced_renderer_v2(1);
        irv2_add_instance(&mut r, identity(), [0.5, 0.5, 0.5, 1.0]);
        let avg = irv2_avg_color(&r);
        assert!((avg[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_avg_color_two() {
        let mut r = new_instanced_renderer_v2(1);
        irv2_add_instance(&mut r, identity(), [0.0, 0.0, 0.0, 1.0]);
        irv2_add_instance(&mut r, identity(), [1.0, 1.0, 1.0, 1.0]);
        let avg = irv2_avg_color(&r);
        assert!((avg[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_multiple_instances_count() {
        let mut r = new_instanced_renderer_v2(3);
        for _ in 0..5 {
            irv2_add_instance(&mut r, identity(), [1.0; 4]);
        }
        assert_eq!(irv2_instance_count(&r), 5);
    }
}
