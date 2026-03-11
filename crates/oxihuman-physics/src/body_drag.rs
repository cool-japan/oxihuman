#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// Drag properties for a physics body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyDrag {
    coefficient: f32,
    area: f32,
    velocity: [f32; 3],
    fluid_density: f32,
}

#[allow(dead_code)]
pub fn new_body_drag(coefficient: f32, area: f32) -> BodyDrag {
    BodyDrag {
        coefficient,
        area,
        velocity: [0.0; 3],
        fluid_density: 1.225, // air density kg/m^3
    }
}

#[allow(dead_code)]
pub fn compute_drag(drag: &BodyDrag) -> f32 {
    let speed_sq = drag.velocity[0] * drag.velocity[0]
        + drag.velocity[1] * drag.velocity[1]
        + drag.velocity[2] * drag.velocity[2];
    0.5 * drag.fluid_density * speed_sq * drag.coefficient * drag.area
}

#[allow(dead_code)]
pub fn drag_coefficient(drag: &BodyDrag) -> f32 {
    drag.coefficient
}

#[allow(dead_code)]
pub fn drag_area(drag: &BodyDrag) -> f32 {
    drag.area
}

#[allow(dead_code)]
pub fn drag_velocity(drag: &BodyDrag) -> [f32; 3] {
    drag.velocity
}

#[allow(dead_code)]
pub fn drag_force_vector(drag: &BodyDrag) -> [f32; 3] {
    let speed_sq = drag.velocity[0] * drag.velocity[0]
        + drag.velocity[1] * drag.velocity[1]
        + drag.velocity[2] * drag.velocity[2];
    let speed = speed_sq.sqrt();
    if speed < 1e-6 {
        return [0.0; 3];
    }
    let magnitude = 0.5 * drag.fluid_density * speed_sq * drag.coefficient * drag.area;
    let scale = -magnitude / speed;
    [
        drag.velocity[0] * scale,
        drag.velocity[1] * scale,
        drag.velocity[2] * scale,
    ]
}

#[allow(dead_code)]
pub fn drag_to_json(drag: &BodyDrag) -> String {
    format!(
        "{{\"coefficient\":{:.6},\"area\":{:.6},\"density\":{:.6}}}",
        drag.coefficient, drag.area, drag.fluid_density
    )
}

#[allow(dead_code)]
pub fn drag_reset(drag: &mut BodyDrag) {
    drag.velocity = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_body_drag() {
        let d = new_body_drag(0.47, 1.0);
        assert!((drag_coefficient(&d) - 0.47).abs() < 1e-6);
    }

    #[test]
    fn test_compute_drag_zero_velocity() {
        let d = new_body_drag(0.47, 1.0);
        assert!(compute_drag(&d).abs() < 1e-6);
    }

    #[test]
    fn test_compute_drag_nonzero() {
        let mut d = new_body_drag(1.0, 1.0);
        d.velocity = [10.0, 0.0, 0.0];
        assert!(compute_drag(&d) > 0.0);
    }

    #[test]
    fn test_drag_coefficient() {
        let d = new_body_drag(0.5, 2.0);
        assert!((drag_coefficient(&d) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_drag_area() {
        let d = new_body_drag(0.5, 2.0);
        assert!((drag_area(&d) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_drag_velocity() {
        let d = new_body_drag(0.5, 1.0);
        let v = drag_velocity(&d);
        assert!(v[0].abs() < 1e-6);
    }

    #[test]
    fn test_drag_force_vector_zero() {
        let d = new_body_drag(0.5, 1.0);
        let f = drag_force_vector(&d);
        assert!(f[0].abs() < 1e-6);
    }

    #[test]
    fn test_drag_force_vector_nonzero() {
        let mut d = new_body_drag(1.0, 1.0);
        d.velocity = [10.0, 0.0, 0.0];
        let f = drag_force_vector(&d);
        assert!(f[0] < 0.0); // opposing direction
    }

    #[test]
    fn test_drag_to_json() {
        let d = new_body_drag(0.47, 1.0);
        let json = drag_to_json(&d);
        assert!(json.contains("\"coefficient\""));
    }

    #[test]
    fn test_drag_reset() {
        let mut d = new_body_drag(0.47, 1.0);
        d.velocity = [5.0, 5.0, 5.0];
        drag_reset(&mut d);
        assert!(d.velocity[0].abs() < 1e-6);
    }
}
