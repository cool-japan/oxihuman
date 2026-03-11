// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! IkLayer — IK target collection with convergence tracking.

#![allow(dead_code)]

/// A single IK end-effector target.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkTarget {
    pub name: String,
    pub position: [f32; 3],
    pub weight: f32,
}

/// A collection of IK targets with a maximum iteration budget.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkLayer {
    pub targets: Vec<IkTarget>,
    pub max_iterations: u32,
    pub tolerance: f32,
    pub converged: bool,
    pub error: f32,
}

/// Create an empty `IkLayer`.
#[allow(dead_code)]
pub fn new_ik_layer(max_iterations: u32, tolerance: f32) -> IkLayer {
    IkLayer {
        targets: Vec::new(),
        max_iterations,
        tolerance,
        converged: false,
        error: f32::INFINITY,
    }
}

/// Add an IK target to the layer.
#[allow(dead_code)]
pub fn add_ik_target(layer: &mut IkLayer, name: &str, position: [f32; 3], weight: f32) {
    layer.targets.push(IkTarget { name: name.to_owned(), position, weight });
}

/// Stub IK solve: mark converged if the number of targets is zero.
#[allow(dead_code)]
pub fn solve_ik_layer(layer: &mut IkLayer) {
    if layer.targets.is_empty() {
        layer.converged = true;
        layer.error = 0.0;
        return;
    }
    // Minimal stub: accept the current poses as "solved".
    layer.error = layer.targets.iter().map(|t| t.weight).sum::<f32>() * 0.0001;
    layer.converged = layer.error < layer.tolerance;
}

/// Return the number of IK targets.
#[allow(dead_code)]
pub fn ik_target_count(layer: &IkLayer) -> usize {
    layer.targets.len()
}

/// Return a reference to the IK target at `index`.
#[allow(dead_code)]
pub fn ik_target_at(layer: &IkLayer, index: usize) -> Option<&IkTarget> {
    layer.targets.get(index)
}

/// Return the last computed IK error.
#[allow(dead_code)]
pub fn ik_error(layer: &IkLayer) -> f32 {
    layer.error
}

/// Return whether the last solve converged.
#[allow(dead_code)]
pub fn ik_converged(layer: &IkLayer) -> bool {
    layer.converged
}

/// Return the maximum iteration budget.
#[allow(dead_code)]
pub fn ik_max_iterations(layer: &IkLayer) -> u32 {
    layer.max_iterations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ik_layer_empty() {
        let l = new_ik_layer(50, 1e-4);
        assert_eq!(ik_target_count(&l), 0);
        assert!(!ik_converged(&l));
    }

    #[test]
    fn test_add_ik_target() {
        let mut l = new_ik_layer(50, 1e-4);
        add_ik_target(&mut l, "hand_l", [0.5, 1.0, 0.0], 1.0);
        assert_eq!(ik_target_count(&l), 1);
    }

    #[test]
    fn test_ik_target_at_some() {
        let mut l = new_ik_layer(50, 1e-4);
        add_ik_target(&mut l, "foot", [0.0, 0.0, 0.0], 0.8);
        let t = ik_target_at(&l, 0).unwrap();
        assert_eq!(t.name, "foot");
    }

    #[test]
    fn test_ik_target_at_none() {
        let l = new_ik_layer(50, 1e-4);
        assert!(ik_target_at(&l, 0).is_none());
    }

    #[test]
    fn test_solve_empty_converges() {
        let mut l = new_ik_layer(50, 1e-4);
        solve_ik_layer(&mut l);
        assert!(ik_converged(&l));
        assert!((ik_error(&l)).abs() < 1e-9);
    }

    #[test]
    fn test_ik_max_iterations() {
        let l = new_ik_layer(100, 1e-5);
        assert_eq!(ik_max_iterations(&l), 100);
    }

    #[test]
    fn test_solve_with_target() {
        let mut l = new_ik_layer(50, 1e-4);
        add_ik_target(&mut l, "wrist", [1.0, 1.0, 0.0], 1.0);
        solve_ik_layer(&mut l);
        assert!(ik_error(&l).is_finite());
    }

    #[test]
    fn test_ik_error_initial_infinite() {
        let l = new_ik_layer(50, 1e-4);
        assert!(ik_error(&l).is_infinite());
    }
}
