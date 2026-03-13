// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Gizmo (3D widget) manager.

#[allow(dead_code)]
pub enum GizmoKind {
    Translate,
    Rotate,
    Scale,
    Universal,
}

#[allow(dead_code)]
pub struct Gizmo {
    pub kind: GizmoKind,
    pub position: [f32; 3],
    pub scale: f32,
    pub active: bool,
}

#[allow(dead_code)]
pub struct GizmoManager {
    pub gizmos: Vec<Gizmo>,
}

#[allow(dead_code)]
pub fn new_gizmo_manager() -> GizmoManager {
    GizmoManager { gizmos: Vec::new() }
}

#[allow(dead_code)]
pub fn gm_add_gizmo(m: &mut GizmoManager, kind: GizmoKind, pos: [f32; 3]) -> usize {
    let idx = m.gizmos.len();
    m.gizmos.push(Gizmo { kind, position: pos, scale: 1.0, active: false });
    idx
}

#[allow(dead_code)]
pub fn gm_activate(m: &mut GizmoManager, idx: usize) {
    if idx < m.gizmos.len() {
        m.gizmos[idx].active = true;
    }
}

#[allow(dead_code)]
pub fn gm_deactivate(m: &mut GizmoManager, idx: usize) {
    if idx < m.gizmos.len() {
        m.gizmos[idx].active = false;
    }
}

#[allow(dead_code)]
pub fn gm_count(m: &GizmoManager) -> usize {
    m.gizmos.len()
}

#[allow(dead_code)]
pub fn gm_active_count(m: &GizmoManager) -> usize {
    m.gizmos.iter().filter(|g| g.active).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_gizmo() {
        let mut m = new_gizmo_manager();
        let idx = gm_add_gizmo(&mut m, GizmoKind::Translate, [0.0; 3]);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_count() {
        let mut m = new_gizmo_manager();
        gm_add_gizmo(&mut m, GizmoKind::Translate, [0.0; 3]);
        gm_add_gizmo(&mut m, GizmoKind::Rotate, [1.0; 3]);
        assert_eq!(gm_count(&m), 2);
    }

    #[test]
    fn test_activate() {
        let mut m = new_gizmo_manager();
        gm_add_gizmo(&mut m, GizmoKind::Scale, [0.0; 3]);
        gm_activate(&mut m, 0);
        assert!(m.gizmos[0].active);
    }

    #[test]
    fn test_deactivate() {
        let mut m = new_gizmo_manager();
        gm_add_gizmo(&mut m, GizmoKind::Universal, [0.0; 3]);
        gm_activate(&mut m, 0);
        gm_deactivate(&mut m, 0);
        assert!(!m.gizmos[0].active);
    }

    #[test]
    fn test_active_count() {
        let mut m = new_gizmo_manager();
        gm_add_gizmo(&mut m, GizmoKind::Translate, [0.0; 3]);
        gm_add_gizmo(&mut m, GizmoKind::Rotate, [0.0; 3]);
        gm_activate(&mut m, 0);
        assert_eq!(gm_active_count(&m), 1);
    }

    #[test]
    fn test_active_count_none() {
        let mut m = new_gizmo_manager();
        gm_add_gizmo(&mut m, GizmoKind::Translate, [0.0; 3]);
        assert_eq!(gm_active_count(&m), 0);
    }

    #[test]
    fn test_activate_out_of_bounds_safe() {
        let mut m = new_gizmo_manager();
        gm_activate(&mut m, 99); /* should not panic */
    }

    #[test]
    fn test_deactivate_out_of_bounds_safe() {
        let mut m = new_gizmo_manager();
        gm_deactivate(&mut m, 99); /* should not panic */
    }
}
