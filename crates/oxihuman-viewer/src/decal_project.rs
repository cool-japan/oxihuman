// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Decal projection — projects decal quads onto surfaces using a projection matrix.

/// Decal blend mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecalBlend {
    Multiply,
    Additive,
    AlphaBlend,
}

/// A projected decal instance.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalInstance {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub size: [f32; 2],
    pub rotation_rad: f32,
    pub blend: DecalBlend,
    pub opacity: f32,
    pub material_id: u32,
}

/// A collection of active decal instances.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DecalProjector {
    pub instances: Vec<DecalInstance>,
}

#[allow(dead_code)]
pub fn new_decal_instance(
    position: [f32; 3],
    normal: [f32; 3],
    size: [f32; 2],
    material_id: u32,
) -> DecalInstance {
    DecalInstance {
        position,
        normal,
        size,
        rotation_rad: 0.0,
        blend: DecalBlend::AlphaBlend,
        opacity: 1.0,
        material_id,
    }
}

#[allow(dead_code)]
pub fn dp_add(proj: &mut DecalProjector, inst: DecalInstance) {
    proj.instances.push(inst);
}

#[allow(dead_code)]
pub fn dp_remove(proj: &mut DecalProjector, index: usize) {
    if index < proj.instances.len() {
        proj.instances.remove(index);
    }
}

#[allow(dead_code)]
pub fn dp_count(proj: &DecalProjector) -> usize {
    proj.instances.len()
}

#[allow(dead_code)]
pub fn dp_clear(proj: &mut DecalProjector) {
    proj.instances.clear();
}

#[allow(dead_code)]
pub fn dp_set_opacity(inst: &mut DecalInstance, v: f32) {
    inst.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn dp_set_blend(inst: &mut DecalInstance, blend: DecalBlend) {
    inst.blend = blend;
}

#[allow(dead_code)]
pub fn dp_area(inst: &DecalInstance) -> f32 {
    inst.size[0] * inst.size[1]
}

/// Project a world point onto the decal's local UV space.
/// Returns `None` if the point is outside the decal quad.
#[allow(dead_code)]
pub fn dp_project_point(inst: &DecalInstance, point: [f32; 3]) -> Option<[f32; 2]> {
    let dx = point[0] - inst.position[0];
    let dz = point[2] - inst.position[2];
    let cos_r = inst.rotation_rad.cos();
    let sin_r = inst.rotation_rad.sin();
    let local_x = cos_r * dx + sin_r * dz;
    let local_z = -sin_r * dx + cos_r * dz;
    let half_w = inst.size[0] * 0.5;
    let half_h = inst.size[1] * 0.5;
    if local_x.abs() <= half_w && local_z.abs() <= half_h {
        let u = (local_x + half_w) / inst.size[0];
        let v = (local_z + half_h) / inst.size[1];
        Some([u, v])
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn dp_count_by_blend(proj: &DecalProjector, blend: DecalBlend) -> usize {
    proj.instances.iter().filter(|i| i.blend == blend).count()
}

#[allow(dead_code)]
pub fn dp_to_json(proj: &DecalProjector) -> String {
    format!(r#"{{"count":{}}}"#, dp_count(proj))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_instance_defaults() {
        let inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0);
        assert!((inst.opacity - 1.0).abs() < 1e-6);
        assert_eq!(inst.blend, DecalBlend::AlphaBlend);
    }

    #[test]
    fn add_and_count() {
        let mut proj = DecalProjector::default();
        dp_add(
            &mut proj,
            new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0),
        );
        assert_eq!(dp_count(&proj), 1);
    }

    #[test]
    fn remove_instance() {
        let mut proj = DecalProjector::default();
        dp_add(
            &mut proj,
            new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0),
        );
        dp_remove(&mut proj, 0);
        assert_eq!(dp_count(&proj), 0);
    }

    #[test]
    fn clear() {
        let mut proj = DecalProjector::default();
        dp_add(
            &mut proj,
            new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [2.0, 2.0], 0),
        );
        dp_clear(&mut proj);
        assert_eq!(dp_count(&proj), 0);
    }

    #[test]
    fn project_center_returns_half_uv() {
        let inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [2.0, 2.0], 0);
        let uv = dp_project_point(&inst, [0.0, 0.0, 0.0]);
        let uv = uv.expect("should succeed");
        assert!((uv[0] - 0.5).abs() < 1e-5);
        assert!((uv[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn project_outside_returns_none() {
        let inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0);
        let uv = dp_project_point(&inst, [5.0, 0.0, 0.0]);
        assert!(uv.is_none());
    }

    #[test]
    fn set_opacity_clamps() {
        let mut inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0);
        dp_set_opacity(&mut inst, 5.0);
        assert!((inst.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn area_computed() {
        let inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [3.0, 4.0], 0);
        assert!((dp_area(&inst) - 12.0).abs() < 1e-5);
    }

    #[test]
    fn count_by_blend() {
        let mut proj = DecalProjector::default();
        let mut inst = new_decal_instance([0.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 0);
        dp_set_blend(&mut inst, DecalBlend::Additive);
        dp_add(&mut proj, inst);
        dp_add(
            &mut proj,
            new_decal_instance([1.0; 3], [0.0, 1.0, 0.0], [1.0, 1.0], 1),
        );
        assert_eq!(dp_count_by_blend(&proj, DecalBlend::Additive), 1);
    }

    #[test]
    fn to_json_count() {
        let proj = DecalProjector::default();
        let j = dp_to_json(&proj);
        assert!(j.contains("count"));
    }
}
