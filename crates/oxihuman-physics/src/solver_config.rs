#![allow(dead_code)]

/// Configuration for the physics solver.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SolverConfiguration {
    pub iterations: u32,
    pub substeps: u32,
    pub tolerance: f32,
    pub warmstart: bool,
}

/// Creates a new solver configuration with custom iterations.
#[allow(dead_code)]
pub fn new_solver_config(iterations: u32) -> SolverConfiguration {
    SolverConfiguration {
        iterations,
        substeps: 1,
        tolerance: 1e-6,
        warmstart: true,
    }
}

/// Creates a default solver configuration.
#[allow(dead_code)]
pub fn default_solver_config() -> SolverConfiguration {
    SolverConfiguration {
        iterations: 10,
        substeps: 1,
        tolerance: 1e-6,
        warmstart: true,
    }
}

/// Sets the number of solver iterations.
#[allow(dead_code)]
pub fn set_iterations(config: &mut SolverConfiguration, iterations: u32) {
    config.iterations = iterations;
}

/// Enables or disables warmstarting.
#[allow(dead_code)]
pub fn set_warmstart_enabled(config: &mut SolverConfiguration, enabled: bool) {
    config.warmstart = enabled;
}

/// Returns the number of iterations.
#[allow(dead_code)]
pub fn solver_iterations(config: &SolverConfiguration) -> u32 {
    config.iterations
}

/// Returns the tolerance.
#[allow(dead_code)]
pub fn solver_tolerance(config: &SolverConfiguration) -> f32 {
    config.tolerance
}

/// Serializes to JSON.
#[allow(dead_code)]
pub fn solver_to_json(config: &SolverConfiguration) -> String {
    format!(
        "{{\"iterations\":{},\"substeps\":{},\"tolerance\":{},\"warmstart\":{}}}",
        config.iterations, config.substeps, config.tolerance, config.warmstart
    )
}

/// Returns the number of substeps.
#[allow(dead_code)]
pub fn solver_substeps(config: &SolverConfiguration) -> u32 {
    config.substeps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        let c = new_solver_config(20);
        assert_eq!(solver_iterations(&c), 20);
    }

    #[test]
    fn test_default_config() {
        let c = default_solver_config();
        assert_eq!(solver_iterations(&c), 10);
    }

    #[test]
    fn test_set_iterations() {
        let mut c = default_solver_config();
        set_iterations(&mut c, 50);
        assert_eq!(solver_iterations(&c), 50);
    }

    #[test]
    fn test_warmstart() {
        let mut c = default_solver_config();
        assert!(c.warmstart);
        set_warmstart_enabled(&mut c, false);
        assert!(!c.warmstart);
    }

    #[test]
    fn test_tolerance() {
        let c = default_solver_config();
        assert!(solver_tolerance(&c) > 0.0);
    }

    #[test]
    fn test_to_json() {
        let c = default_solver_config();
        let json = solver_to_json(&c);
        assert!(json.contains("\"iterations\":10"));
    }

    #[test]
    fn test_substeps() {
        let c = default_solver_config();
        assert_eq!(solver_substeps(&c), 1);
    }

    #[test]
    fn test_custom_substeps() {
        let mut c = default_solver_config();
        c.substeps = 4;
        assert_eq!(solver_substeps(&c), 4);
    }

    #[test]
    fn test_custom_tolerance() {
        let mut c = default_solver_config();
        c.tolerance = 1e-4;
        assert!((solver_tolerance(&c) - 1e-4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_clone() {
        let c = default_solver_config();
        let c2 = c.clone();
        assert_eq!(solver_iterations(&c), solver_iterations(&c2));
    }
}
