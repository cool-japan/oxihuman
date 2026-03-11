// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

pub type Params = HashMap<String, f32>;

/// Constraint types between parameters.
#[derive(Clone, Debug)]
pub enum Constraint {
    /// target must be >= source * factor + offset
    MinRelative {
        source: String,
        target: String,
        factor: f32,
        offset: f32,
    },
    /// target must be <= source * factor + offset
    MaxRelative {
        source: String,
        target: String,
        factor: f32,
        offset: f32,
    },
    /// target = source * factor + offset (hard link), clamped to [0, 1]
    Driven {
        source: String,
        target: String,
        factor: f32,
        offset: f32,
    },
    /// sum of named params must equal total (redistributes proportionally)
    SumEquals { params: Vec<String>, total: f32 },
    /// param must be in [min, max]
    Clamp { param: String, min: f32, max: f32 },
    /// param_a + param_b <= max_sum
    MaxSum {
        param_a: String,
        param_b: String,
        max_sum: f32,
    },
    /// if condition_param >= threshold, then target_param gets assigned value
    Conditional {
        condition: String,
        threshold: f32,
        target: String,
        value: f32,
    },
}

impl Constraint {
    /// Apply this constraint to params in-place, return true if any change was made.
    pub fn apply(&self, params: &mut Params) -> bool {
        const EPS: f32 = 1e-6;
        match self {
            Constraint::MinRelative {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let min_val = src * factor + offset;
                let cur = *params.get(target).unwrap_or(&0.0);
                if cur < min_val - EPS {
                    params.insert(target.clone(), min_val);
                    true
                } else {
                    false
                }
            }
            Constraint::MaxRelative {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let max_val = src * factor + offset;
                let cur = *params.get(target).unwrap_or(&0.0);
                if cur > max_val + EPS {
                    params.insert(target.clone(), max_val);
                    true
                } else {
                    false
                }
            }
            Constraint::Driven {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let new_val = (src * factor + offset).clamp(0.0, 1.0);
                let cur = *params.get(target).unwrap_or(&0.0);
                if (cur - new_val).abs() > EPS {
                    params.insert(target.clone(), new_val);
                    true
                } else {
                    false
                }
            }
            Constraint::SumEquals {
                params: keys,
                total,
            } => {
                let current_sum: f32 = keys.iter().map(|k| *params.get(k).unwrap_or(&0.0)).sum();
                if (current_sum - total).abs() <= EPS {
                    return false;
                }
                if current_sum.abs() < EPS {
                    // Distribute equally
                    let equal_share = total / keys.len() as f32;
                    for k in keys {
                        params.insert(k.clone(), equal_share);
                    }
                } else {
                    let scale = total / current_sum;
                    for k in keys {
                        let v = *params.get(k).unwrap_or(&0.0);
                        params.insert(k.clone(), v * scale);
                    }
                }
                true
            }
            Constraint::Clamp { param, min, max } => {
                let cur = *params.get(param).unwrap_or(&0.0);
                let clamped = cur.clamp(*min, *max);
                if (cur - clamped).abs() > EPS {
                    params.insert(param.clone(), clamped);
                    true
                } else {
                    false
                }
            }
            Constraint::MaxSum {
                param_a,
                param_b,
                max_sum,
            } => {
                let a = *params.get(param_a).unwrap_or(&0.0);
                let b = *params.get(param_b).unwrap_or(&0.0);
                let sum = a + b;
                if sum > max_sum + 1e-6 {
                    let scale = max_sum / sum;
                    params.insert(param_a.clone(), a * scale);
                    params.insert(param_b.clone(), b * scale);
                    true
                } else {
                    false
                }
            }
            Constraint::Conditional {
                condition,
                threshold,
                target,
                value,
            } => {
                let cond_val = *params.get(condition).unwrap_or(&0.0);
                if cond_val >= *threshold {
                    let cur = *params.get(target).unwrap_or(&0.0);
                    if (cur - value).abs() > 1e-6 {
                        params.insert(target.clone(), *value);
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Check if the constraint is currently satisfied.
    pub fn is_satisfied(&self, params: &Params) -> bool {
        const EPS: f32 = 1e-5;
        match self {
            Constraint::MinRelative {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let cur = *params.get(target).unwrap_or(&0.0);
                cur >= src * factor + offset - EPS
            }
            Constraint::MaxRelative {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let cur = *params.get(target).unwrap_or(&0.0);
                cur <= src * factor + offset + EPS
            }
            Constraint::Driven {
                source,
                target,
                factor,
                offset,
            } => {
                let src = *params.get(source).unwrap_or(&0.0);
                let expected = (src * factor + offset).clamp(0.0, 1.0);
                let cur = *params.get(target).unwrap_or(&0.0);
                (cur - expected).abs() <= EPS
            }
            Constraint::SumEquals {
                params: keys,
                total,
            } => {
                let s: f32 = keys.iter().map(|k| *params.get(k).unwrap_or(&0.0)).sum();
                (s - total).abs() <= EPS
            }
            Constraint::Clamp { param, min, max } => {
                let v = *params.get(param).unwrap_or(&0.0);
                v >= *min - EPS && v <= *max + EPS
            }
            Constraint::MaxSum {
                param_a,
                param_b,
                max_sum,
            } => {
                let a = *params.get(param_a).unwrap_or(&0.0);
                let b = *params.get(param_b).unwrap_or(&0.0);
                a + b <= max_sum + EPS
            }
            Constraint::Conditional {
                condition,
                threshold,
                target,
                value,
            } => {
                let cond_val = *params.get(condition).unwrap_or(&0.0);
                if cond_val >= *threshold {
                    let cur = *params.get(target).unwrap_or(&0.0);
                    (cur - value).abs() <= EPS
                } else {
                    true
                }
            }
        }
    }

    /// Name/description of constraint for debugging.
    pub fn describe(&self) -> String {
        match self {
            Constraint::MinRelative {
                source,
                target,
                factor,
                offset,
            } => format!(
                "MinRelative: {} >= {} * {} + {}",
                target, source, factor, offset
            ),
            Constraint::MaxRelative {
                source,
                target,
                factor,
                offset,
            } => format!(
                "MaxRelative: {} <= {} * {} + {}",
                target, source, factor, offset
            ),
            Constraint::Driven {
                source,
                target,
                factor,
                offset,
            } => format!(
                "Driven: {} = {} * {} + {} (clamped to [0,1])",
                target, source, factor, offset
            ),
            Constraint::SumEquals { params, total } => {
                format!("SumEquals: {:?} sums to {}", params, total)
            }
            Constraint::Clamp { param, min, max } => {
                format!("Clamp: {} in [{}, {}]", param, min, max)
            }
            Constraint::MaxSum {
                param_a,
                param_b,
                max_sum,
            } => format!("MaxSum: {} + {} <= {}", param_a, param_b, max_sum),
            Constraint::Conditional {
                condition,
                threshold,
                target,
                value,
            } => format!(
                "Conditional: if {} >= {} then {} = {}",
                condition, threshold, target, value
            ),
        }
    }
}

/// Iterative constraint solver.
pub struct ConstraintSolver {
    constraints: Vec<Constraint>,
    max_iterations: usize,
    tolerance: f32,
}

/// Result of a solve pass.
pub struct SolveResult {
    pub iterations: usize,
    pub converged: bool,
    pub violations_remaining: usize,
    pub changes_made: usize,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            max_iterations: 100,
            tolerance: 1e-5,
        }
    }

    pub fn with_max_iterations(mut self, n: usize) -> Self {
        self.max_iterations = n;
        self
    }

    pub fn with_tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    pub fn add(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.constraints.len() {
            self.constraints.remove(index);
        }
    }

    /// Solve all constraints iteratively until convergence or max_iterations.
    pub fn solve(&self, params: &mut Params) -> SolveResult {
        let mut total_changes = 0usize;
        let mut iterations = 0usize;
        let mut converged = false;

        for _ in 0..self.max_iterations {
            iterations += 1;
            let mut changed_this_iter = false;

            for constraint in &self.constraints {
                if constraint.apply(params) {
                    total_changes += 1;
                    changed_this_iter = true;
                }
            }

            if !changed_this_iter {
                converged = true;
                break;
            }
        }

        let violations_remaining = self.check_violations(params).len();

        SolveResult {
            iterations,
            converged,
            violations_remaining,
            changes_made: total_changes,
        }
    }

    /// Check which constraints (by index) are violated.
    pub fn check_violations(&self, params: &Params) -> Vec<usize> {
        self.constraints
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_satisfied(params))
            .map(|(i, _)| i)
            .collect()
    }

    /// True if all constraints are satisfied.
    pub fn is_satisfied(&self, params: &Params) -> bool {
        self.constraints.iter().all(|c| c.is_satisfied(params))
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Human body BMI-related constraint presets.
///
/// Encodes: weight is driven by bmi_factor; muscle is bounded relative to weight.
pub fn bmi_constraints() -> Vec<Constraint> {
    vec![
        // weight is driven directly by bmi_factor (normalized)
        Constraint::Driven {
            source: "bmi_factor".to_string(),
            target: "weight".to_string(),
            factor: 1.0,
            offset: 0.0,
        },
        // muscle must be <= weight * 0.9 + 0.1 (heavier bodies can have more muscle)
        Constraint::MaxRelative {
            source: "weight".to_string(),
            target: "muscle".to_string(),
            factor: 0.9,
            offset: 0.1,
        },
        // muscle must stay in [0, 1]
        Constraint::Clamp {
            param: "muscle".to_string(),
            min: 0.0,
            max: 1.0,
        },
        // weight must stay in [0, 1]
        Constraint::Clamp {
            param: "weight".to_string(),
            min: 0.0,
            max: 1.0,
        },
    ]
}

/// Proportion constraints linking limb dimensions to height.
pub fn proportion_constraints() -> Vec<Constraint> {
    vec![
        // shoulder_width >= height * 0.3 + 0.05
        Constraint::MinRelative {
            source: "height".to_string(),
            target: "shoulder_width".to_string(),
            factor: 0.3,
            offset: 0.05,
        },
        // shoulder_width <= height * 0.6 + 0.1
        Constraint::MaxRelative {
            source: "height".to_string(),
            target: "shoulder_width".to_string(),
            factor: 0.6,
            offset: 0.1,
        },
        // shoulder_width must stay in [0, 1]
        Constraint::Clamp {
            param: "shoulder_width".to_string(),
            min: 0.0,
            max: 1.0,
        },
        // leg_length is driven by height
        Constraint::Driven {
            source: "height".to_string(),
            target: "leg_length".to_string(),
            factor: 0.85,
            offset: 0.05,
        },
    ]
}

/// Age-related constraint presets.
///
/// Muscle mass decreases with age; body fat increases slightly.
pub fn age_constraints() -> Vec<Constraint> {
    vec![
        // muscle decreases as age increases: muscle <= 1.0 - age * 0.5
        Constraint::MaxRelative {
            source: "age".to_string(),
            target: "muscle".to_string(),
            factor: -0.5,
            offset: 1.0,
        },
        // at advanced age (>= 0.8), assign lower muscle cap
        Constraint::Conditional {
            condition: "age".to_string(),
            threshold: 0.8,
            target: "elderly_flag".to_string(),
            value: 1.0,
        },
        // muscle clamped
        Constraint::Clamp {
            param: "muscle".to_string(),
            min: 0.0,
            max: 1.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(pairs: &[(&str, f32)]) -> Params {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_clamp_constraint() {
        let c = Constraint::Clamp {
            param: "height".to_string(),
            min: 0.0,
            max: 1.0,
        };
        let mut p = make_params(&[("height", 1.5)]);
        assert!(c.apply(&mut p));
        assert!((p["height"] - 1.0).abs() < 1e-6);

        let mut p2 = make_params(&[("height", 0.5)]);
        assert!(!c.apply(&mut p2));
        assert!((p2["height"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_min_relative_constraint() {
        let c = Constraint::MinRelative {
            source: "height".to_string(),
            target: "shoulder_width".to_string(),
            factor: 0.3,
            offset: 0.0,
        };
        // height=0.8 → min shoulder=0.24; shoulder=0.1 should be raised
        let mut p = make_params(&[("height", 0.8), ("shoulder_width", 0.1)]);
        assert!(c.apply(&mut p));
        assert!((p["shoulder_width"] - 0.24).abs() < 1e-5);

        // shoulder already satisfies: no change
        let mut p2 = make_params(&[("height", 0.8), ("shoulder_width", 0.5)]);
        assert!(!c.apply(&mut p2));
    }

    #[test]
    fn test_max_relative_constraint() {
        let c = Constraint::MaxRelative {
            source: "weight".to_string(),
            target: "muscle".to_string(),
            factor: 0.9,
            offset: 0.1,
        };
        // weight=0.5 → max muscle=0.55; muscle=0.8 should be capped
        let mut p = make_params(&[("weight", 0.5), ("muscle", 0.8)]);
        assert!(c.apply(&mut p));
        assert!((p["muscle"] - 0.55).abs() < 1e-5);

        // muscle already satisfies: no change
        let mut p2 = make_params(&[("weight", 0.5), ("muscle", 0.3)]);
        assert!(!c.apply(&mut p2));
    }

    #[test]
    fn test_driven_constraint() {
        let c = Constraint::Driven {
            source: "bmi_factor".to_string(),
            target: "weight".to_string(),
            factor: 1.0,
            offset: 0.0,
        };
        let mut p = make_params(&[("bmi_factor", 0.7), ("weight", 0.0)]);
        assert!(c.apply(&mut p));
        assert!((p["weight"] - 0.7).abs() < 1e-6);

        // Already set correctly
        let mut p2 = make_params(&[("bmi_factor", 0.7), ("weight", 0.7)]);
        assert!(!c.apply(&mut p2));

        // Clamped to [0,1]
        let mut p3 = make_params(&[("bmi_factor", 1.5), ("weight", 0.0)]);
        assert!(c.apply(&mut p3));
        assert!((p3["weight"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sum_equals_constraint() {
        let c = Constraint::SumEquals {
            params: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            total: 1.0,
        };
        let mut p = make_params(&[("a", 0.2), ("b", 0.3), ("c", 0.5)]);
        // sum = 1.0, already satisfied
        assert!(!c.apply(&mut p));

        let mut p2 = make_params(&[("a", 0.5), ("b", 0.5), ("c", 0.5)]);
        // sum = 1.5, scale to 1.0
        assert!(c.apply(&mut p2));
        let new_sum: f32 = ["a", "b", "c"].iter().map(|k| p2[*k]).sum();
        assert!((new_sum - 1.0).abs() < 1e-5);

        // Zero-sum case: distribute equally
        let mut p3 = make_params(&[("a", 0.0), ("b", 0.0), ("c", 0.0)]);
        assert!(c.apply(&mut p3));
        assert!((p3["a"] - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_sum_constraint() {
        let c = Constraint::MaxSum {
            param_a: "muscle".to_string(),
            param_b: "fat".to_string(),
            max_sum: 1.0,
        };
        let mut p = make_params(&[("muscle", 0.7), ("fat", 0.6)]);
        assert!(c.apply(&mut p));
        let s = p["muscle"] + p["fat"];
        assert!((s - 1.0).abs() < 1e-5);

        let mut p2 = make_params(&[("muscle", 0.4), ("fat", 0.4)]);
        assert!(!c.apply(&mut p2));
    }

    #[test]
    fn test_conditional_constraint() {
        let c = Constraint::Conditional {
            condition: "age".to_string(),
            threshold: 0.8,
            target: "elderly_flag".to_string(),
            value: 1.0,
        };
        // age >= 0.8 → elderly_flag should be set to 1.0
        let mut p = make_params(&[("age", 0.9), ("elderly_flag", 0.0)]);
        assert!(c.apply(&mut p));
        assert!((p["elderly_flag"] - 1.0).abs() < 1e-6);

        // age < 0.8 → no change
        let mut p2 = make_params(&[("age", 0.5), ("elderly_flag", 0.0)]);
        assert!(!c.apply(&mut p2));
    }

    #[test]
    fn test_constraint_is_satisfied() {
        let c = Constraint::Clamp {
            param: "x".to_string(),
            min: 0.0,
            max: 1.0,
        };
        let p_ok = make_params(&[("x", 0.5)]);
        assert!(c.is_satisfied(&p_ok));

        let p_bad = make_params(&[("x", 1.5)]);
        assert!(!c.is_satisfied(&p_bad));

        // Driven
        let d = Constraint::Driven {
            source: "s".to_string(),
            target: "t".to_string(),
            factor: 2.0,
            offset: 0.0,
        };
        // s=0.4 → t should be 0.8
        let p_driven_ok = make_params(&[("s", 0.4), ("t", 0.8)]);
        assert!(d.is_satisfied(&p_driven_ok));

        let p_driven_bad = make_params(&[("s", 0.4), ("t", 0.5)]);
        assert!(!d.is_satisfied(&p_driven_bad));
    }

    #[test]
    fn test_solver_new() {
        let s = ConstraintSolver::new();
        assert_eq!(s.constraint_count(), 0);
        assert_eq!(s.max_iterations, 100);
    }

    #[test]
    fn test_solver_add_and_count() {
        let mut s = ConstraintSolver::new();
        s.add(Constraint::Clamp {
            param: "x".to_string(),
            min: 0.0,
            max: 1.0,
        });
        s.add(Constraint::Clamp {
            param: "y".to_string(),
            min: 0.0,
            max: 1.0,
        });
        assert_eq!(s.constraint_count(), 2);
        s.remove(0);
        assert_eq!(s.constraint_count(), 1);
    }

    #[test]
    fn test_solver_solve_converges() {
        let mut s = ConstraintSolver::new();
        s.add(Constraint::Clamp {
            param: "height".to_string(),
            min: 0.0,
            max: 1.0,
        });
        s.add(Constraint::Clamp {
            param: "weight".to_string(),
            min: 0.0,
            max: 1.0,
        });

        let mut p = make_params(&[("height", 1.5), ("weight", -0.2)]);
        let result = s.solve(&mut p);

        assert!(result.converged);
        assert!(result.violations_remaining == 0);
        assert!((p["height"] - 1.0).abs() < 1e-5);
        assert!((p["weight"] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_solver_check_violations() {
        let mut s = ConstraintSolver::new();
        s.add(Constraint::Clamp {
            param: "x".to_string(),
            min: 0.0,
            max: 1.0,
        });
        s.add(Constraint::Clamp {
            param: "y".to_string(),
            min: 0.0,
            max: 1.0,
        });

        let p = make_params(&[("x", 1.5), ("y", 0.5)]);
        let violations = s.check_violations(&p);
        assert_eq!(violations, vec![0]);

        let p2 = make_params(&[("x", 0.5), ("y", 0.5)]);
        assert!(s.check_violations(&p2).is_empty());
        assert!(s.is_satisfied(&p2));
    }

    #[test]
    fn test_bmi_constraints() {
        let constraints = bmi_constraints();
        assert!(!constraints.is_empty());

        let mut s = ConstraintSolver::new();
        for c in constraints {
            s.add(c);
        }

        let mut p = make_params(&[("bmi_factor", 0.6), ("weight", 0.0), ("muscle", 0.9)]);
        let result = s.solve(&mut p);
        assert!(result.converged);
        // weight driven to bmi_factor
        assert!((p["weight"] - 0.6).abs() < 1e-5);
        // muscle <= weight * 0.9 + 0.1 = 0.64
        assert!(p["muscle"] <= 0.64 + 1e-4);
    }

    #[test]
    fn test_solve_result_fields() {
        let mut s = ConstraintSolver::new();
        s.add(Constraint::Clamp {
            param: "z".to_string(),
            min: 0.0,
            max: 1.0,
        });

        let mut p = make_params(&[("z", 2.0)]);
        let r = s.solve(&mut p);

        assert!(r.iterations >= 1); // At least one iteration ran
        assert!(r.converged);
        assert_eq!(r.violations_remaining, 0);
        assert_eq!(r.changes_made, 1);
    }
}
