#![allow(dead_code)]

/// A single row of a constraint system (Jacobian row + RHS + lambda).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintRow {
    pub jacobian: [f32; 6],
    pub rhs: f32,
    pub lambda: f32,
    pub cfm: f32,
    pub erp: f32,
}

/// Creates a new constraint row.
#[allow(dead_code)]
pub fn new_constraint_row(jacobian: [f32; 6], rhs: f32) -> ConstraintRow {
    ConstraintRow {
        jacobian,
        rhs,
        lambda: 0.0,
        cfm: 1e-6,
        erp: 0.2,
    }
}

/// Returns the Jacobian array.
#[allow(dead_code)]
pub fn row_jacobian(row: &ConstraintRow) -> &[f32; 6] {
    &row.jacobian
}

/// Returns the right-hand side.
#[allow(dead_code)]
pub fn row_rhs(row: &ConstraintRow) -> f32 {
    row.rhs
}

/// Returns the accumulated lambda.
#[allow(dead_code)]
pub fn row_lambda(row: &ConstraintRow) -> f32 {
    row.lambda
}

/// Solves the row for one iteration using PGS.
#[allow(dead_code, clippy::needless_range_loop)]
pub fn solve_row(row: &mut ConstraintRow, velocities: &[f32; 6], inv_mass: f32) {
    let mut jv = 0.0f32;
    for i in 0..6 {
        jv += row.jacobian[i] * velocities[i];
    }
    let effective_mass = {
        let mut em = 0.0f32;
        for i in 0..6 {
            em += row.jacobian[i] * row.jacobian[i] * inv_mass;
        }
        em + row.cfm
    };
    if effective_mass > f32::EPSILON {
        let delta_lambda = (row.rhs * row.erp - jv) / effective_mass;
        row.lambda += delta_lambda;
    }
}

/// Returns the constraint error.
#[allow(dead_code, clippy::needless_range_loop)]
pub fn row_error(row: &ConstraintRow, velocities: &[f32; 6]) -> f32 {
    let mut jv = 0.0f32;
    for i in 0..6 {
        jv += row.jacobian[i] * velocities[i];
    }
    row.rhs - jv
}

/// Returns the CFM value.
#[allow(dead_code)]
pub fn row_cfm(row: &ConstraintRow) -> f32 {
    row.cfm
}

/// Returns the ERP value.
#[allow(dead_code)]
pub fn row_erp(row: &ConstraintRow) -> f32 {
    row.erp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_row() {
        let row = new_constraint_row([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1.0);
        assert!((row_rhs(&row) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jacobian() {
        let row = new_constraint_row([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 0.0);
        assert!((row_jacobian(&row)[0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_lambda_initial() {
        let row = new_constraint_row([1.0; 6], 0.0);
        assert!((row_lambda(&row)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve_row() {
        let mut row = new_constraint_row([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1.0);
        let vels = [0.0f32; 6];
        solve_row(&mut row, &vels, 1.0);
        assert!(row_lambda(&row).abs() > 0.0);
    }

    #[test]
    fn test_row_error() {
        let row = new_constraint_row([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1.0);
        let vels = [0.5, 0.0, 0.0, 0.0, 0.0, 0.0];
        let err = row_error(&row, &vels);
        assert!((err - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cfm() {
        let row = new_constraint_row([0.0; 6], 0.0);
        assert!(row_cfm(&row) > 0.0);
    }

    #[test]
    fn test_erp() {
        let row = new_constraint_row([0.0; 6], 0.0);
        assert!((row_erp(&row) - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve_zero_jacobian() {
        let mut row = new_constraint_row([0.0; 6], 1.0);
        let vels = [0.0f32; 6];
        solve_row(&mut row, &vels, 1.0);
        // With zero jacobian, effective_mass = cfm only, lambda should change
        assert!(row_lambda(&row).abs() > 0.0);
    }

    #[test]
    fn test_error_zero_velocity() {
        let row = new_constraint_row([1.0; 6], 2.0);
        let vels = [0.0f32; 6];
        assert!((row_error(&row, &vels) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_multiple_solves() {
        let mut row = new_constraint_row([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 1.0);
        let vels = [0.0f32; 6];
        for _ in 0..10 {
            solve_row(&mut row, &vels, 1.0);
        }
        assert!(row_lambda(&row).abs() > 0.0);
    }
}
