// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Decal projector: projects 2D textures onto 3D surfaces.

/// A decal projector box.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecalProjector {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub up: [f32; 3],
    pub half_extents: [f32; 3],
    pub opacity: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn new_decal_projector(position: [f32; 3], direction: [f32; 3]) -> DecalProjector {
    DecalProjector {
        position,
        direction,
        up: [0.0, 1.0, 0.0],
        half_extents: [0.5, 0.5, 0.5],
        opacity: 1.0,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn set_decal_opacity(proj: &mut DecalProjector, v: f32) {
    proj.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_decal_extents(proj: &mut DecalProjector, half_extents: [f32; 3]) {
    proj.half_extents = half_extents;
}

#[allow(dead_code)]
pub fn set_decal_enabled(proj: &mut DecalProjector, enabled: bool) {
    proj.enabled = enabled;
}

/// Test if a point is inside the decal projection box.
#[allow(dead_code)]
pub fn point_in_decal_box(proj: &DecalProjector, point: [f32; 3]) -> bool {
    let dx = (point[0] - proj.position[0]).abs();
    let dy = (point[1] - proj.position[1]).abs();
    let dz = (point[2] - proj.position[2]).abs();
    dx <= proj.half_extents[0] && dy <= proj.half_extents[1] && dz <= proj.half_extents[2]
}

/// Project a world-space point into decal UV space (simplified).
#[allow(dead_code)]
pub fn project_to_decal_uv(proj: &DecalProjector, point: [f32; 3]) -> Option<[f32; 2]> {
    if !point_in_decal_box(proj, point) {
        return None;
    }
    let hx = proj.half_extents[0];
    let hy = proj.half_extents[1];
    if hx.abs() < 1e-9 || hy.abs() < 1e-9 {
        return None;
    }
    let u = ((point[0] - proj.position[0]) / hx + 1.0) * 0.5;
    let v = ((point[1] - proj.position[1]) / hy + 1.0) * 0.5;
    Some([u.clamp(0.0, 1.0), v.clamp(0.0, 1.0)])
}

#[allow(dead_code)]
pub fn decal_volume(proj: &DecalProjector) -> f32 {
    8.0 * proj.half_extents[0] * proj.half_extents[1] * proj.half_extents[2]
}

#[allow(dead_code)]
pub fn decal_to_json(proj: &DecalProjector) -> String {
    format!(
        r#"{{"position":[{:.4},{:.4},{:.4}],"opacity":{:.4},"enabled":{}}}"#,
        proj.position[0], proj.position[1], proj.position[2], proj.opacity, proj.enabled
    )
}

#[allow(dead_code)]
pub fn reset_decal(proj: &mut DecalProjector) {
    proj.opacity = 1.0;
    proj.enabled = true;
    proj.half_extents = [0.5, 0.5, 0.5];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_decal_projector() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        assert!((p.opacity - 1.0).abs() < 1e-6);
        assert!(p.enabled);
    }

    #[test]
    fn test_set_opacity() {
        let mut p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        set_decal_opacity(&mut p, 0.5);
        assert!((p.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_opacity_clamps() {
        let mut p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        set_decal_opacity(&mut p, 2.0);
        assert!((p.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_point_in_box_inside() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        assert!(point_in_decal_box(&p, [0.0, 0.0, 0.0]));
    }

    #[test]
    fn test_point_in_box_outside() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        assert!(!point_in_decal_box(&p, [10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_project_to_uv() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        let uv = project_to_decal_uv(&p, [0.0, 0.0, 0.0]);
        assert!(uv.is_some());
        let uv = uv.expect("should succeed");
        assert!((uv[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_project_to_uv_outside() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        assert!(project_to_decal_uv(&p, [10.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_decal_volume() {
        let p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        assert!((decal_volume(&p) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decal_to_json() {
        let p = new_decal_projector([1.0, 2.0, 3.0], [0.0, 0.0, -1.0]);
        let j = decal_to_json(&p);
        assert!(j.contains("position"));
    }

    #[test]
    fn test_reset_decal() {
        let mut p = new_decal_projector([0.0; 3], [0.0, 0.0, -1.0]);
        set_decal_opacity(&mut p, 0.2);
        reset_decal(&mut p);
        assert!((p.opacity - 1.0).abs() < 1e-6);
    }
}
