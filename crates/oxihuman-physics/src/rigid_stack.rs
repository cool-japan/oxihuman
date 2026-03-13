// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Stack of rigid boxes with stacking stability analysis.

#![allow(dead_code)]

/// A single box in the stack.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StackBox {
    pub id: u32,
    pub width: f32,
    pub depth: f32,
    pub height: f32,
    pub mass: f32,
    /// Center of mass position.
    pub position: [f32; 3],
    /// Is this box currently settled/sleeping?
    pub sleeping: bool,
}

/// A vertical stack of rigid boxes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidStack {
    pub boxes: Vec<StackBox>,
    /// Ground y position.
    pub ground_y: f32,
}

/// Create an empty rigid stack.
#[allow(dead_code)]
pub fn new_rigid_stack(ground_y: f32) -> RigidStack {
    RigidStack {
        boxes: Vec::new(),
        ground_y,
    }
}

/// Add a box to the top of the stack. Position is computed automatically.
#[allow(dead_code)]
pub fn rs_push_box(
    stack: &mut RigidStack,
    id: u32,
    width: f32,
    depth: f32,
    height: f32,
    mass: f32,
) {
    let y = stack_top_y(stack) + height * 0.5;
    stack.boxes.push(StackBox {
        id,
        width,
        depth,
        height,
        mass,
        position: [0.0, y, 0.0],
        sleeping: false,
    });
}

/// Compute the y of the top surface of the stack.
#[allow(dead_code)]
pub fn stack_top_y(stack: &RigidStack) -> f32 {
    if let Some(top) = stack.boxes.last() {
        top.position[1] + top.height * 0.5
    } else {
        stack.ground_y
    }
}

/// Number of boxes in the stack.
#[allow(dead_code)]
pub fn rs_count(stack: &RigidStack) -> usize {
    stack.boxes.len()
}

/// Total mass of the stack.
#[allow(dead_code)]
pub fn rs_total_mass(stack: &RigidStack) -> f32 {
    stack.boxes.iter().map(|b| b.mass).sum()
}

/// Compute center of mass of the entire stack.
#[allow(dead_code)]
pub fn rs_center_of_mass(stack: &RigidStack) -> [f32; 3] {
    let total = rs_total_mass(stack);
    if total < f32::EPSILON {
        return [0.0, 0.0, 0.0];
    }
    let mut com = [0.0f32; 3];
    for b in &stack.boxes {
        com[0] += b.position[0] * b.mass;
        com[1] += b.position[1] * b.mass;
        com[2] += b.position[2] * b.mass;
    }
    [com[0] / total, com[1] / total, com[2] / total]
}

/// Check if each box in the stack is supported by the box below
/// (center of mass of above box is within the footprint of the below box).
#[allow(dead_code)]
pub fn rs_is_stable(stack: &RigidStack) -> bool {
    for i in 1..stack.boxes.len() {
        let above = &stack.boxes[i];
        let below = &stack.boxes[i - 1];
        let dx = (above.position[0] - below.position[0]).abs();
        let dz = (above.position[2] - below.position[2]).abs();
        if dx > below.width * 0.5 || dz > below.depth * 0.5 {
            return false;
        }
    }
    true
}

/// Pop the topmost box from the stack.
#[allow(dead_code)]
pub fn rs_pop_box(stack: &mut RigidStack) -> Option<StackBox> {
    stack.boxes.pop()
}

/// Sleep all boxes (mark as settled).
#[allow(dead_code)]
pub fn rs_sleep_all(stack: &mut RigidStack) {
    for b in &mut stack.boxes {
        b.sleeping = true;
    }
}

/// Total height of the stack.
#[allow(dead_code)]
pub fn rs_total_height(stack: &RigidStack) -> f32 {
    stack_top_y(stack) - stack.ground_y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_stack() -> RigidStack {
        let mut s = new_rigid_stack(0.0);
        rs_push_box(&mut s, 1, 2.0, 2.0, 1.0, 10.0);
        rs_push_box(&mut s, 2, 1.5, 1.5, 1.0, 8.0);
        rs_push_box(&mut s, 3, 1.0, 1.0, 1.0, 5.0);
        s
    }

    #[test]
    fn count_correct() {
        let s = simple_stack();
        assert_eq!(rs_count(&s), 3);
    }

    #[test]
    fn total_mass() {
        let s = simple_stack();
        assert!((rs_total_mass(&s) - 23.0).abs() < 1e-4);
    }

    #[test]
    fn top_y_increases() {
        let s = simple_stack();
        assert!((stack_top_y(&s) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn stable_centered_stack() {
        let s = simple_stack();
        assert!(rs_is_stable(&s));
    }

    #[test]
    fn unstable_offset_box() {
        let mut s = new_rigid_stack(0.0);
        rs_push_box(&mut s, 1, 2.0, 2.0, 1.0, 10.0);
        let y = stack_top_y(&s) + 0.5;
        s.boxes.push(StackBox {
            id: 2,
            width: 1.0,
            depth: 1.0,
            height: 1.0,
            mass: 5.0,
            position: [5.0, y, 0.0], // way off center
            sleeping: false,
        });
        assert!(!rs_is_stable(&s));
    }

    #[test]
    fn pop_box_removes_top() {
        let mut s = simple_stack();
        let top = rs_pop_box(&mut s).expect("should succeed");
        assert_eq!(top.id, 3);
        assert_eq!(rs_count(&s), 2);
    }

    #[test]
    fn sleep_all() {
        let mut s = simple_stack();
        rs_sleep_all(&mut s);
        assert!(s.boxes.iter().all(|b| b.sleeping));
    }

    #[test]
    fn center_of_mass_y_in_range() {
        let s = simple_stack();
        let com = rs_center_of_mass(&s);
        assert!(com[1] > 0.0 && com[1] < 3.5);
    }

    #[test]
    fn total_height_correct() {
        let s = simple_stack();
        assert!((rs_total_height(&s) - 3.0).abs() < 1e-4);
    }

    #[test]
    fn empty_stack_top_at_ground() {
        let s = new_rigid_stack(1.5);
        assert!((stack_top_y(&s) - 1.5).abs() < 1e-5);
    }
}
