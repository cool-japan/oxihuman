#![allow(dead_code)]

/// A constraint on morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct MorphConstraint {
    name: String,
    min: f32,
    max: f32,
}

/// Solver for morph parameter constraints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphConstraintSolver {
    constraints: Vec<MorphConstraint>,
    max_iterations: usize,
}

#[allow(dead_code)]
pub fn new_morph_constraint_solver(max_iterations: usize) -> MorphConstraintSolver {
    MorphConstraintSolver { constraints: Vec::new(), max_iterations }
}

#[allow(dead_code)]
pub fn add_morph_constraint(solver: &mut MorphConstraintSolver, name: &str, min: f32, max: f32) {
    solver.constraints.push(MorphConstraint { name: name.to_string(), min, max });
}

#[allow(dead_code)]
pub fn solve_morph_constraints(solver: &MorphConstraintSolver, values: &mut [f32], names: &[String]) {
    for _ in 0..solver.max_iterations {
        for c in &solver.constraints {
            if let Some(idx) = names.iter().position(|n| n == &c.name) {
                if idx < values.len() {
                    values[idx] = values[idx].clamp(c.min, c.max);
                }
            }
        }
    }
}

#[allow(dead_code)]
pub fn constraint_count_mcs(solver: &MorphConstraintSolver) -> usize { solver.constraints.len() }

#[allow(dead_code)]
pub fn constraint_error_mcs(solver: &MorphConstraintSolver, values: &[f32], names: &[String]) -> f32 {
    let mut err = 0.0f32;
    for c in &solver.constraints {
        if let Some(idx) = names.iter().position(|n| n == &c.name) {
            if idx < values.len() {
                let v = values[idx];
                if v < c.min { err += c.min - v; }
                if v > c.max { err += v - c.max; }
            }
        }
    }
    err
}

#[allow(dead_code)]
pub fn solver_to_json_mcs(solver: &MorphConstraintSolver) -> String {
    format!("{{\"constraint_count\":{},\"max_iterations\":{}}}", solver.constraints.len(), solver.max_iterations)
}

#[allow(dead_code)]
pub fn clear_solver_mcs(solver: &mut MorphConstraintSolver) { solver.constraints.clear(); }

#[allow(dead_code)]
pub fn solver_converged_mcs(solver: &MorphConstraintSolver, values: &[f32], names: &[String]) -> bool {
    constraint_error_mcs(solver, values, names) < 1e-6
}

#[cfg(test)]
mod tests {
    use super::*;
    fn names(n: &[&str]) -> Vec<String> { n.iter().map(|s| s.to_string()).collect() }

    #[test] fn test_new() { assert_eq!(constraint_count_mcs(&new_morph_constraint_solver(10)), 0); }
    #[test] fn test_add() {
        let mut s = new_morph_constraint_solver(10);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        assert_eq!(constraint_count_mcs(&s), 1);
    }
    #[test] fn test_solve() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        let mut v = [2.0f32];
        let n = names(&["x"]);
        solve_morph_constraints(&s, &mut v, &n);
        assert!((v[0] - 1.0).abs() < 1e-6);
    }
    #[test] fn test_error() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        let v = [2.0f32];
        let n = names(&["x"]);
        assert!((constraint_error_mcs(&s, &v, &n) - 1.0).abs() < 1e-6);
    }
    #[test] fn test_converged() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        let v = [0.5f32];
        let n = names(&["x"]);
        assert!(solver_converged_mcs(&s, &v, &n));
    }
    #[test] fn test_not_converged() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        let v = [5.0f32];
        let n = names(&["x"]);
        assert!(!solver_converged_mcs(&s, &v, &n));
    }
    #[test] fn test_clear() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        clear_solver_mcs(&mut s);
        assert_eq!(constraint_count_mcs(&s), 0);
    }
    #[test] fn test_to_json() {
        let s = new_morph_constraint_solver(5);
        assert!(solver_to_json_mcs(&s).contains("max_iterations"));
    }
    #[test] fn test_solve_within_range() {
        let mut s = new_morph_constraint_solver(1);
        add_morph_constraint(&mut s, "x", 0.0, 1.0);
        let mut v = [0.5f32];
        let n = names(&["x"]);
        solve_morph_constraints(&s, &mut v, &n);
        assert!((v[0] - 0.5).abs() < 1e-6);
    }
    #[test] fn test_error_zero() {
        let s = new_morph_constraint_solver(1);
        let v = [0.5f32];
        let n = names(&["x"]);
        assert!((constraint_error_mcs(&s, &v, &n)).abs() < 1e-6);
    }
}
