// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3D cursor state.

#![allow(dead_code)]

/// 3D cursor state (position + rotation quaternion).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Cursor3d {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub visible: bool,
}

/// Returns a default 3D cursor at the origin with identity rotation.
#[allow(dead_code)]
pub fn default_cursor3d() -> Cursor3d {
    Cursor3d {
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        visible: true,
    }
}

/// Moves the cursor to the given position.
#[allow(dead_code)]
pub fn move_cursor(cursor: &mut Cursor3d, pos: [f32; 3]) {
    cursor.position = pos;
}

/// Snaps the cursor position to the nearest grid vertex.
#[allow(dead_code)]
pub fn snap_cursor_to_grid(cursor: &mut Cursor3d, grid_size: f32) {
    let gs = if grid_size.abs() < f32::EPSILON { 1.0 } else { grid_size };
    cursor.position[0] = (cursor.position[0] / gs).round() * gs;
    cursor.position[1] = (cursor.position[1] / gs).round() * gs;
    cursor.position[2] = (cursor.position[2] / gs).round() * gs;
}

/// Computes Euclidean distance between two cursor positions.
#[allow(dead_code)]
pub fn cursor_distance(a: &Cursor3d, b: &Cursor3d) -> f32 {
    let dx = a.position[0] - b.position[0];
    let dy = a.position[1] - b.position[1];
    let dz = a.position[2] - b.position[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Resets the cursor to the default state.
#[allow(dead_code)]
pub fn reset_cursor(cursor: &mut Cursor3d) {
    *cursor = default_cursor3d();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cursor_at_origin() {
        let c = default_cursor3d();
        assert!((c.position[0]).abs() < 1e-6);
        assert!((c.position[1]).abs() < 1e-6);
        assert!((c.position[2]).abs() < 1e-6);
    }

    #[test]
    fn test_default_rotation_identity() {
        let c = default_cursor3d();
        assert!((c.rotation[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_move_cursor() {
        let mut c = default_cursor3d();
        move_cursor(&mut c, [1.0, 2.0, 3.0]);
        assert!((c.position[0] - 1.0).abs() < 1e-6);
        assert!((c.position[1] - 2.0).abs() < 1e-6);
        assert!((c.position[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_snap_to_grid() {
        let mut c = default_cursor3d();
        c.position = [0.6, 1.4, -0.4];
        snap_cursor_to_grid(&mut c, 1.0);
        assert!((c.position[0] - 1.0).abs() < 1e-5);
        assert!((c.position[1] - 1.0).abs() < 1e-5);
        assert!((c.position[2] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_snap_zero_grid_size() {
        let mut c = default_cursor3d();
        c.position = [1.5, 1.5, 1.5];
        snap_cursor_to_grid(&mut c, 0.0); // fallback to 1.0
        assert!((c.position[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_cursor_distance_zero() {
        let a = default_cursor3d();
        let b = default_cursor3d();
        assert!((cursor_distance(&a, &b)).abs() < 1e-6);
    }

    #[test]
    fn test_cursor_distance_unit() {
        let a = default_cursor3d();
        let mut b = default_cursor3d();
        b.position = [1.0, 0.0, 0.0];
        assert!((cursor_distance(&a, &b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_reset_cursor() {
        let mut c = default_cursor3d();
        c.position = [5.0, 5.0, 5.0];
        c.visible = false;
        reset_cursor(&mut c);
        assert!((c.position[0]).abs() < 1e-6);
        assert!(c.visible);
    }

    #[test]
    fn test_visible_default_true() {
        let c = default_cursor3d();
        assert!(c.visible);
    }
}
