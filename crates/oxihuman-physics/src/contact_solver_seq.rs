#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SeqContact {
    normal: [f32; 3],
    depth: f32,
    impulse: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SeqContactSolver {
    contacts: Vec<SeqContact>,
    iterations: u32,
    max_iterations: u32,
    residual: f32,
    tolerance: f32,
}

#[allow(dead_code)]
pub fn new_seq_solver(max_iterations: u32, tolerance: f32) -> SeqContactSolver {
    SeqContactSolver {
        contacts: Vec::new(),
        iterations: 0,
        max_iterations,
        residual: 0.0,
        tolerance,
    }
}

#[allow(dead_code)]
pub fn add_seq_contact(solver: &mut SeqContactSolver, normal: [f32; 3], depth: f32) {
    solver.contacts.push(SeqContact {
        normal,
        depth,
        impulse: 0.0,
    });
}

#[allow(dead_code)]
pub fn solve_seq_contacts(solver: &mut SeqContactSolver, inv_mass: f32) {
    solver.iterations = 0;
    for _ in 0..solver.max_iterations {
        solver.iterations += 1;
        solver.residual = 0.0;
        for c in solver.contacts.iter_mut() {
            let delta = c.depth * inv_mass;
            c.impulse += delta;
            if c.impulse < 0.0 {
                c.impulse = 0.0;
            }
            solver.residual += delta.abs();
        }
        if solver.residual < solver.tolerance {
            break;
        }
    }
}

#[allow(dead_code)]
pub fn seq_iteration_count(solver: &SeqContactSolver) -> u32 {
    solver.iterations
}

#[allow(dead_code)]
pub fn seq_residual(solver: &SeqContactSolver) -> f32 {
    solver.residual
}

#[allow(dead_code)]
pub fn seq_contact_count(solver: &SeqContactSolver) -> usize {
    solver.contacts.len()
}

#[allow(dead_code)]
pub fn seq_clear(solver: &mut SeqContactSolver) {
    solver.contacts.clear();
    solver.iterations = 0;
    solver.residual = 0.0;
}

#[allow(dead_code)]
pub fn seq_converged(solver: &SeqContactSolver) -> bool {
    solver.residual < solver.tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = new_seq_solver(10, 1e-6);
        assert_eq!(seq_contact_count(&s), 0);
    }

    #[test]
    fn test_add_contact() {
        let mut s = new_seq_solver(10, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        assert_eq!(seq_contact_count(&s), 1);
    }

    #[test]
    fn test_solve() {
        let mut s = new_seq_solver(10, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.5);
        solve_seq_contacts(&mut s, 1.0);
        assert!(seq_iteration_count(&s) > 0);
    }

    #[test]
    fn test_residual() {
        let mut s = new_seq_solver(1, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        solve_seq_contacts(&mut s, 1.0);
        assert!(seq_residual(&s) >= 0.0);
    }

    #[test]
    fn test_clear() {
        let mut s = new_seq_solver(10, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.1);
        seq_clear(&mut s);
        assert_eq!(seq_contact_count(&s), 0);
    }

    #[test]
    fn test_converged_empty() {
        let s = new_seq_solver(10, 1e-6);
        assert!(seq_converged(&s));
    }

    #[test]
    fn test_iteration_count() {
        let mut s = new_seq_solver(5, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.01);
        solve_seq_contacts(&mut s, 1.0);
        assert!((1..=5).contains(&seq_iteration_count(&s)));
    }

    #[test]
    fn test_multiple_contacts() {
        let mut s = new_seq_solver(10, 1e-6);
        add_seq_contact(&mut s, [1.0, 0.0, 0.0], 0.1);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.2);
        assert_eq!(seq_contact_count(&s), 2);
    }

    #[test]
    fn test_zero_depth() {
        let mut s = new_seq_solver(10, 1e-6);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 0.0);
        solve_seq_contacts(&mut s, 1.0);
        assert!(seq_converged(&s));
    }

    #[test]
    fn test_high_iterations() {
        let mut s = new_seq_solver(100, 0.0);
        add_seq_contact(&mut s, [0.0, 1.0, 0.0], 1.0);
        solve_seq_contacts(&mut s, 1.0);
        assert_eq!(seq_iteration_count(&s), 100);
    }
}
