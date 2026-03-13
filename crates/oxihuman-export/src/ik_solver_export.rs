// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Export IK solver configuration data.
#[allow(dead_code)]
pub enum IkSolverType {
    Fabrik,
    TwoBone,
    CcdIk,
    JacobianTranspose,
}

#[allow(dead_code)]
pub struct IkJointEntry {
    pub name: String,
    pub min_angle: [f32; 3],
    pub max_angle: [f32; 3],
    pub stiffness: f32,
    pub stretch: f32,
}

#[allow(dead_code)]
pub struct IkSolverExport {
    pub name: String,
    pub solver_type: IkSolverType,
    pub root_bone: String,
    pub tip_bone: String,
    pub target_name: String,
    pub pole_target: Option<String>,
    pub iterations: u32,
    pub tolerance: f32,
    pub chain_length: u32,
    pub joints: Vec<IkJointEntry>,
}

#[allow(dead_code)]
pub struct IkSolverBundle {
    pub solvers: Vec<IkSolverExport>,
}

#[allow(dead_code)]
pub fn new_ik_solver_bundle() -> IkSolverBundle {
    IkSolverBundle { solvers: vec![] }
}

#[allow(dead_code)]
pub fn add_solver(bundle: &mut IkSolverBundle, solver: IkSolverExport) {
    bundle.solvers.push(solver);
}

#[allow(dead_code)]
pub fn solver_count(bundle: &IkSolverBundle) -> usize {
    bundle.solvers.len()
}

#[allow(dead_code)]
pub fn default_ik_solver(name: &str, root: &str, tip: &str, target: &str) -> IkSolverExport {
    IkSolverExport {
        name: name.to_string(),
        solver_type: IkSolverType::Fabrik,
        root_bone: root.to_string(),
        tip_bone: tip.to_string(),
        target_name: target.to_string(),
        pole_target: None,
        iterations: 10,
        tolerance: 0.001,
        chain_length: 2,
        joints: vec![],
    }
}

#[allow(dead_code)]
pub fn validate_ik_solver(solver: &IkSolverExport) -> bool {
    !solver.name.is_empty()
        && !solver.root_bone.is_empty()
        && !solver.tip_bone.is_empty()
        && solver.iterations > 0
        && solver.tolerance > 0.0
        && solver.chain_length > 0
}

#[allow(dead_code)]
pub fn solver_type_name(st: &IkSolverType) -> &'static str {
    match st {
        IkSolverType::Fabrik => "fabrik",
        IkSolverType::TwoBone => "two_bone",
        IkSolverType::CcdIk => "ccd_ik",
        IkSolverType::JacobianTranspose => "jacobian_transpose",
    }
}

#[allow(dead_code)]
pub fn ik_solver_to_json(solver: &IkSolverExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"type\":\"{}\",\"root\":\"{}\",\"tip\":\"{}\",\"iterations\":{}}}",
        solver.name,
        solver_type_name(&solver.solver_type),
        solver.root_bone,
        solver.tip_bone,
        solver.iterations
    )
}

#[allow(dead_code)]
pub fn ik_bundle_to_json(bundle: &IkSolverBundle) -> String {
    format!("{{\"solver_count\":{}}}", bundle.solvers.len())
}

#[allow(dead_code)]
pub fn find_solver<'a>(bundle: &'a IkSolverBundle, name: &str) -> Option<&'a IkSolverExport> {
    bundle.solvers.iter().find(|s| s.name == name)
}

#[allow(dead_code)]
pub fn total_ik_joints(bundle: &IkSolverBundle) -> usize {
    bundle.solvers.iter().map(|s| s.joints.len()).sum()
}

#[allow(dead_code)]
pub fn solvers_with_pole(bundle: &IkSolverBundle) -> Vec<&IkSolverExport> {
    bundle
        .solvers
        .iter()
        .filter(|s| s.pole_target.is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm_ik() -> IkSolverExport {
        default_ik_solver("arm_ik", "upper_arm", "hand", "hand_target")
    }

    #[test]
    fn test_add_solver() {
        let mut b = new_ik_solver_bundle();
        add_solver(&mut b, arm_ik());
        assert_eq!(solver_count(&b), 1);
    }

    #[test]
    fn test_validate_default() {
        let s = arm_ik();
        assert!(validate_ik_solver(&s));
    }

    #[test]
    fn test_validate_zero_iterations_fails() {
        let mut s = arm_ik();
        s.iterations = 0;
        assert!(!validate_ik_solver(&s));
    }

    #[test]
    fn test_find_solver_found() {
        let mut b = new_ik_solver_bundle();
        add_solver(&mut b, arm_ik());
        assert!(find_solver(&b, "arm_ik").is_some());
    }

    #[test]
    fn test_find_solver_missing() {
        let b = new_ik_solver_bundle();
        assert!(find_solver(&b, "leg_ik").is_none());
    }

    #[test]
    fn test_solver_type_name() {
        assert_eq!(solver_type_name(&IkSolverType::Fabrik), "fabrik");
        assert_eq!(solver_type_name(&IkSolverType::TwoBone), "two_bone");
    }

    #[test]
    fn test_solvers_with_pole_empty() {
        let mut b = new_ik_solver_bundle();
        add_solver(&mut b, arm_ik());
        assert_eq!(solvers_with_pole(&b).len(), 0);
    }

    #[test]
    fn test_solvers_with_pole_found() {
        let mut b = new_ik_solver_bundle();
        let mut s = arm_ik();
        s.pole_target = Some("elbow_pole".to_string());
        add_solver(&mut b, s);
        assert_eq!(solvers_with_pole(&b).len(), 1);
    }

    #[test]
    fn test_to_json() {
        let s = arm_ik();
        let j = ik_solver_to_json(&s);
        assert!(j.contains("arm_ik"));
    }

    #[test]
    fn test_bundle_to_json() {
        let mut b = new_ik_solver_bundle();
        add_solver(&mut b, arm_ik());
        let j = ik_bundle_to_json(&b);
        assert!(j.contains("solver_count"));
    }
}
