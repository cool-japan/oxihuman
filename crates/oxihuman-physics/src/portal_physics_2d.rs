// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D portal/warp teleportation physics stub.

#[derive(Debug, Clone, PartialEq)]
pub struct Portal2d {
    pub id: usize,
    pub position: [f32; 2],
    pub normal: [f32; 2],
    pub half_width: f32,
    pub linked_to: Option<usize>,
}

impl Portal2d {
    pub fn new(id: usize, x: f32, y: f32, nx: f32, ny: f32, half_width: f32) -> Self {
        let len = (nx * nx + ny * ny).sqrt().max(f32::EPSILON);
        Portal2d {
            id,
            position: [x, y],
            normal: [nx / len, ny / len],
            half_width,
            linked_to: None,
        }
    }

    pub fn link(&mut self, other_id: usize) {
        self.linked_to = Some(other_id);
    }

    pub fn point_in_portal(&self, p: [f32; 2]) -> bool {
        point_near_portal(p, self)
    }
}

pub fn point_near_portal(p: [f32; 2], portal: &Portal2d) -> bool {
    let dx = p[0] - portal.position[0];
    let dy = p[1] - portal.position[1];
    /* Project onto portal tangent (perpendicular to normal) */
    let tx = -portal.normal[1];
    let ty = portal.normal[0];
    let along = (dx * tx + dy * ty).abs();
    /* Check if on the portal face (close to its plane) */
    let perp = (dx * portal.normal[0] + dy * portal.normal[1]).abs();
    along <= portal.half_width && perp < 0.5
}

/// Transform a position through a portal pair.
pub fn teleport_through(
    pos: [f32; 2],
    vel: [f32; 2],
    from: &Portal2d,
    to: &Portal2d,
) -> ([f32; 2], [f32; 2]) {
    let dx = pos[0] - from.position[0];
    let dy = pos[1] - from.position[1];
    let new_pos = [to.position[0] + dx, to.position[1] + dy];
    let dot = vel[0] * from.normal[0] + vel[1] * from.normal[1];
    let new_vel = [
        vel[0] - 2.0 * dot * (from.normal[0] - to.normal[0]),
        vel[1] - 2.0 * dot * (from.normal[1] - to.normal[1]),
    ];
    (new_pos, new_vel)
}

pub fn check_portal_crossing(old_pos: [f32; 2], new_pos: [f32; 2], portal: &Portal2d) -> bool {
    let op = [
        old_pos[0] - portal.position[0],
        old_pos[1] - portal.position[1],
    ];
    let np = [
        new_pos[0] - portal.position[0],
        new_pos[1] - portal.position[1],
    ];
    let sign_old = op[0] * portal.normal[0] + op[1] * portal.normal[1];
    let sign_new = np[0] * portal.normal[0] + np[1] * portal.normal[1];
    sign_old * sign_new < 0.0 && point_near_portal(new_pos, portal)
}

#[allow(clippy::too_many_arguments)]
pub fn portal_pair(
    ax: f32,
    ay: f32,
    anx: f32,
    any: f32,
    bx: f32,
    by: f32,
    bnx: f32,
    bny: f32,
    half_width: f32,
) -> (Portal2d, Portal2d) {
    let mut a = Portal2d::new(0, ax, ay, anx, any, half_width);
    let mut b = Portal2d::new(1, bx, by, bnx, bny, half_width);
    a.link(1);
    b.link(0);
    (a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_creation() {
        let p = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 2.0);
        assert_eq!(p.id, 0);
        assert!((p.normal[0] - 1.0).abs() < 1e-5, /* normal normalized */);
    }

    #[test]
    fn test_link() {
        let mut p = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 2.0);
        p.link(1);
        assert_eq!(p.linked_to, Some(1));
    }

    #[test]
    fn test_point_in_portal() {
        let p = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 2.0);
        assert!(p.point_in_portal([0.1, 1.0]), /* within half_width along tangent */);
    }

    #[test]
    fn test_point_outside_portal() {
        let p = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 1.0);
        assert!(!p.point_in_portal([0.0, 5.0]), /* too far along tangent */);
    }

    #[test]
    fn test_teleport_position_offset() {
        let from = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 5.0);
        let to = Portal2d::new(1, 10.0, 0.0, -1.0, 0.0, 5.0);
        let (new_pos, _) = teleport_through([0.1, 0.5], [1.0, 0.0], &from, &to);
        assert!((new_pos[0] - 10.1).abs() < 1e-4 /* offset preserved */,);
    }

    #[test]
    fn test_portal_pair_linked() {
        let (a, b) = portal_pair(0.0, 0.0, 1.0, 0.0, 10.0, 0.0, -1.0, 0.0, 2.0);
        assert_eq!(a.linked_to, Some(1));
        assert_eq!(b.linked_to, Some(0));
    }

    #[test]
    fn test_crossing_detection() {
        let portal = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 2.0);
        let old_pos = [-0.1, 0.5];
        let new_pos = [0.1, 0.5];
        assert!(
            check_portal_crossing(old_pos, new_pos, &portal),
            /* crossing portal plane */
        );
    }

    #[test]
    fn test_no_crossing_same_side() {
        let portal = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 2.0);
        let old_pos = [1.0, 0.5];
        let new_pos = [2.0, 0.5];
        assert!(
            !check_portal_crossing(old_pos, new_pos, &portal),
            /* both on same side, no crossing */
        );
    }

    #[test]
    fn test_velocity_transformed() {
        let from = Portal2d::new(0, 0.0, 0.0, 1.0, 0.0, 5.0);
        let to = Portal2d::new(1, 10.0, 0.0, 1.0, 0.0, 5.0);
        let (_, new_vel) = teleport_through([0.0, 0.0], [5.0, 0.0], &from, &to);
        assert!(
            new_vel[0].is_finite() && new_vel[1].is_finite(),
            /* velocity remains finite */
        );
    }
}
