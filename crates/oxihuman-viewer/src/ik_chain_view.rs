// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! IK chain debug visualization view stub.

/// IK solver type indicator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IkSolverType {
    Ccd,
    Fabrik,
    TwoBone,
    Jacobian,
}

/// IK chain view configuration.
#[derive(Debug, Clone)]
pub struct IkChainView {
    pub solver_type: IkSolverType,
    pub chain_color: [f32; 4],
    pub target_color: [f32; 4],
    pub show_pole_vector: bool,
    pub show_iterations: bool,
    pub enabled: bool,
}

impl IkChainView {
    pub fn new() -> Self {
        IkChainView {
            solver_type: IkSolverType::Fabrik,
            chain_color: [0.0, 1.0, 0.5, 1.0],
            target_color: [1.0, 0.8, 0.0, 1.0],
            show_pole_vector: true,
            show_iterations: false,
            enabled: true,
        }
    }
}

impl Default for IkChainView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new IK chain view.
pub fn new_ik_chain_view() -> IkChainView {
    IkChainView::new()
}

/// Set solver type.
pub fn ikv_set_solver(view: &mut IkChainView, solver: IkSolverType) {
    view.solver_type = solver;
}

/// Set chain visualization color.
pub fn ikv_set_chain_color(view: &mut IkChainView, color: [f32; 4]) {
    view.chain_color = color;
}

/// Set target marker color.
pub fn ikv_set_target_color(view: &mut IkChainView, color: [f32; 4]) {
    view.target_color = color;
}

/// Toggle pole vector display.
pub fn ikv_show_pole_vector(view: &mut IkChainView, show: bool) {
    view.show_pole_vector = show;
}

/// Toggle iteration count display.
pub fn ikv_show_iterations(view: &mut IkChainView, show: bool) {
    view.show_iterations = show;
}

/// Enable or disable.
pub fn ikv_set_enabled(view: &mut IkChainView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn ikv_to_json(view: &IkChainView) -> String {
    let solver = match view.solver_type {
        IkSolverType::Ccd => "ccd",
        IkSolverType::Fabrik => "fabrik",
        IkSolverType::TwoBone => "two_bone",
        IkSolverType::Jacobian => "jacobian",
    };
    format!(
        r#"{{"solver":"{}","show_pole_vector":{},"show_iterations":{},"enabled":{}}}"#,
        solver, view.show_pole_vector, view.show_iterations, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_solver() {
        let v = new_ik_chain_view();
        assert_eq!(
            v.solver_type,
            IkSolverType::Fabrik /* default solver must be Fabrik */
        );
    }

    #[test]
    fn test_set_solver() {
        let mut v = new_ik_chain_view();
        ikv_set_solver(&mut v, IkSolverType::TwoBone);
        assert_eq!(
            v.solver_type,
            IkSolverType::TwoBone /* solver must be set */
        );
    }

    #[test]
    fn test_show_pole_vector() {
        let mut v = new_ik_chain_view();
        ikv_show_pole_vector(&mut v, false);
        assert!(!v.show_pole_vector /* pole vector must be hidden */);
    }

    #[test]
    fn test_show_iterations() {
        let mut v = new_ik_chain_view();
        ikv_show_iterations(&mut v, true);
        assert!(v.show_iterations /* iterations must be shown */);
    }

    #[test]
    fn test_set_chain_color() {
        let mut v = new_ik_chain_view();
        ikv_set_chain_color(&mut v, [1.0, 0.0, 0.0, 1.0]);
        assert!((v.chain_color[0] - 1.0).abs() < 1e-6 /* red channel must be 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_ik_chain_view();
        ikv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_solver() {
        let v = new_ik_chain_view();
        let j = ikv_to_json(&v);
        assert!(j.contains("\"solver\"") /* JSON must have solver */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_ik_chain_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_pole_vector_default_true() {
        let v = new_ik_chain_view();
        assert!(v.show_pole_vector /* pole vector must be shown by default */);
    }
}
