#![allow(dead_code)]
//! A set of parameter constraints that can be evaluated for satisfaction.

/// A single constraint: `param` must be within `[min, max]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Constraint {
    param: String,
    min: f32,
    max: f32,
}

/// A collection of parameter constraints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParamConstraintSet {
    constraints: Vec<Constraint>,
}

/// Create a new empty constraint set.
#[allow(dead_code)]
pub fn new_param_constraint_set() -> ParamConstraintSet {
    ParamConstraintSet {
        constraints: Vec::new(),
    }
}

/// Add a constraint: `param` must be in `[min, max]`.
#[allow(dead_code)]
pub fn add_param_constraint(set: &mut ParamConstraintSet, param: &str, min: f32, max: f32) {
    set.constraints.push(Constraint {
        param: param.to_string(),
        min,
        max,
    });
}

/// Evaluate all constraints against `values`. Returns a vec of booleans.
#[allow(dead_code)]
pub fn evaluate_constraints_pc(
    set: &ParamConstraintSet,
    values: &std::collections::HashMap<String, f32>,
) -> Vec<bool> {
    set.constraints
        .iter()
        .map(|c| {
            values
                .get(&c.param)
                .is_some_and(|v| (c.min..=c.max).contains(v))
        })
        .collect()
}

/// Return the number of constraints.
#[allow(dead_code)]
pub fn constraint_count_pcs(set: &ParamConstraintSet) -> usize {
    set.constraints.len()
}

/// Check if all constraints are satisfied.
#[allow(dead_code)]
pub fn is_satisfied_pcs(
    set: &ParamConstraintSet,
    values: &std::collections::HashMap<String, f32>,
) -> bool {
    evaluate_constraints_pc(set, values).iter().all(|&b| b)
}

/// Return the number of violated constraints.
#[allow(dead_code)]
pub fn violation_count(
    set: &ParamConstraintSet,
    values: &std::collections::HashMap<String, f32>,
) -> usize {
    evaluate_constraints_pc(set, values)
        .iter()
        .filter(|&&b| !b)
        .count()
}

/// Serialize constraints to JSON-like string.
#[allow(dead_code)]
pub fn constraints_to_json(set: &ParamConstraintSet) -> String {
    let entries: Vec<String> = set
        .constraints
        .iter()
        .map(|c| format!("{{\"param\":\"{}\",\"min\":{},\"max\":{}}}", c.param, c.min, c.max))
        .collect();
    format!("{{\"constraints\":[{}]}}", entries.join(","))
}

/// Remove all constraints.
#[allow(dead_code)]
pub fn clear_constraints_pcs(set: &mut ParamConstraintSet) {
    set.constraints.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new_set() {
        let s = new_param_constraint_set();
        assert_eq!(constraint_count_pcs(&s), 0);
    }

    #[test]
    fn test_add_constraint() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "height", 0.0, 1.0);
        assert_eq!(constraint_count_pcs(&s), 1);
    }

    #[test]
    fn test_satisfied() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "x", 0.0, 1.0);
        let mut vals = HashMap::new();
        vals.insert("x".to_string(), 0.5);
        assert!(is_satisfied_pcs(&s, &vals));
    }

    #[test]
    fn test_not_satisfied() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "x", 0.0, 1.0);
        let mut vals = HashMap::new();
        vals.insert("x".to_string(), 2.0);
        assert!(!is_satisfied_pcs(&s, &vals));
    }

    #[test]
    fn test_violation_count() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "a", 0.0, 1.0);
        add_param_constraint(&mut s, "b", 0.0, 1.0);
        let mut vals = HashMap::new();
        vals.insert("a".to_string(), 0.5);
        vals.insert("b".to_string(), 5.0);
        assert_eq!(violation_count(&s, &vals), 1);
    }

    #[test]
    fn test_evaluate_constraints() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "p", 0.0, 1.0);
        let mut vals = HashMap::new();
        vals.insert("p".to_string(), 0.5);
        let results = evaluate_constraints_pc(&s, &vals);
        assert_eq!(results, vec![true]);
    }

    #[test]
    fn test_constraints_to_json() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "z", 0.0, 1.0);
        let json = constraints_to_json(&s);
        assert!(json.contains("\"param\":\"z\""));
    }

    #[test]
    fn test_clear() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "a", 0.0, 1.0);
        clear_constraints_pcs(&mut s);
        assert_eq!(constraint_count_pcs(&s), 0);
    }

    #[test]
    fn test_missing_param_not_satisfied() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "missing", 0.0, 1.0);
        let vals = HashMap::new();
        assert!(!is_satisfied_pcs(&s, &vals));
    }

    #[test]
    fn test_boundary_values() {
        let mut s = new_param_constraint_set();
        add_param_constraint(&mut s, "x", 0.0, 1.0);
        let mut vals = HashMap::new();
        vals.insert("x".to_string(), 0.0);
        assert!(is_satisfied_pcs(&s, &vals));
        vals.insert("x".to_string(), 1.0);
        assert!(is_satisfied_pcs(&s, &vals));
    }
}
