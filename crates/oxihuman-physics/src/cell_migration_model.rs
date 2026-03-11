// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cell random walk / chemotaxis model.

use std::f32::consts::TAU;

pub struct Cell {
    pub pos: [f32; 2],
    pub speed: f32,
    pub persistence: f32,
    pub direction: f32,
}

pub fn new_cell(pos: [f32; 2], speed: f32) -> Cell {
    Cell {
        pos,
        speed: speed.max(0.0),
        persistence: 0.8,
        direction: 0.0,
    }
}

pub fn cell_step(c: &mut Cell, dt: f32, noise: f32) {
    /* persistent random walk: direction rotates by noise * TAU */
    let angle_change = (noise - 0.5) * TAU * (1.0 - c.persistence);
    c.direction += angle_change;
    c.pos[0] += c.speed * c.direction.cos() * dt;
    c.pos[1] += c.speed * c.direction.sin() * dt;
}

pub fn cell_distance_from_origin(c: &Cell) -> f32 {
    (c.pos[0] * c.pos[0] + c.pos[1] * c.pos[1]).sqrt()
}

pub fn cell_chemotaxis_step(c: &mut Cell, gradient_dir: f32, dt: f32) {
    /* bias movement toward gradient direction */
    c.direction = c.direction * (1.0 - c.persistence) + gradient_dir * c.persistence;
    c.pos[0] += c.speed * c.direction.cos() * dt;
    c.pos[1] += c.speed * c.direction.sin() * dt;
}

pub fn cell_position(c: &Cell) -> [f32; 2] {
    c.pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cell() {
        /* new cell is at given position */
        let c = new_cell([1.0, 2.0], 1.0);
        assert!((c.pos[0] - 1.0).abs() < 1e-5);
        assert!((c.pos[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_cell_step_moves() {
        /* cell moves after step */
        let mut c = new_cell([0.0, 0.0], 1.0);
        let initial_pos = c.pos;
        cell_step(&mut c, 1.0, 0.5);
        assert!(c.pos[0] != initial_pos[0] || c.pos[1] != initial_pos[1]);
    }

    #[test]
    fn test_cell_distance_from_origin() {
        /* distance from origin is correct */
        let c = new_cell([3.0, 4.0], 1.0);
        let d = cell_distance_from_origin(&c);
        assert!((d - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_cell_chemotaxis_step() {
        /* chemotaxis step moves cell toward gradient */
        let mut c = new_cell([0.0, 0.0], 1.0);
        c.direction = 0.0;
        cell_chemotaxis_step(&mut c, 0.0, 1.0); /* gradient toward +x */
        assert!(c.pos[0] > 0.0);
    }

    #[test]
    fn test_cell_position() {
        /* cell_position returns current position */
        let c = new_cell([5.0, 6.0], 1.0);
        let pos = cell_position(&c);
        assert!((pos[0] - 5.0).abs() < 1e-5);
        assert!((pos[1] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_cell_zero_speed() {
        /* zero speed cell does not move */
        let mut c = new_cell([0.0, 0.0], 0.0);
        cell_step(&mut c, 1.0, 0.5);
        assert!((c.pos[0]).abs() < 1e-6);
        assert!((c.pos[1]).abs() < 1e-6);
    }

    #[test]
    fn test_cell_persistence() {
        /* high persistence keeps direction more stable */
        let c = new_cell([0.0, 0.0], 1.0);
        assert!(c.persistence > 0.5);
    }
}
