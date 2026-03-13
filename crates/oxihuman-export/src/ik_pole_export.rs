// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! IK pole vector export: IK chain pole targets and constraints.

/// An IK pole target.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkPoleTarget {
    pub chain_name: String,
    pub pole_position: [f32; 3],
    pub influence: f32,
    pub use_local_space: bool,
}

/// IK pole export bundle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IkPoleExport {
    pub poles: Vec<IkPoleTarget>,
}

/// Create a new IK pole export.
#[allow(dead_code)]
pub fn new_ik_pole_export() -> IkPoleExport {
    IkPoleExport { poles: Vec::new() }
}

/// Add a pole target.
#[allow(dead_code)]
pub fn add_ik_pole(exp: &mut IkPoleExport, pole: IkPoleTarget) {
    exp.poles.push(pole);
}

/// Pole count.
#[allow(dead_code)]
pub fn ik_pole_count(exp: &IkPoleExport) -> usize {
    exp.poles.len()
}

/// Find pole by chain name.
#[allow(dead_code)]
pub fn find_ik_pole<'a>(exp: &'a IkPoleExport, chain: &str) -> Option<&'a IkPoleTarget> {
    exp.poles.iter().find(|p| p.chain_name == chain)
}

/// Average influence.
#[allow(dead_code)]
pub fn avg_pole_influence(exp: &IkPoleExport) -> f32 {
    if exp.poles.is_empty() {
        return 0.0;
    }
    exp.poles.iter().map(|p| p.influence).sum::<f32>() / exp.poles.len() as f32
}

/// Count poles using local space.
#[allow(dead_code)]
pub fn local_space_pole_count(exp: &IkPoleExport) -> usize {
    exp.poles.iter().filter(|p| p.use_local_space).count()
}

/// Validate: influence in `[0,1]`.
#[allow(dead_code)]
pub fn validate_ik_poles(exp: &IkPoleExport) -> bool {
    exp.poles.iter().all(|p| (0.0..=1.0).contains(&p.influence))
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn ik_pole_to_json(exp: &IkPoleExport) -> String {
    format!(
        "{{\"pole_count\":{},\"avg_influence\":{}}}",
        ik_pole_count(exp),
        avg_pole_influence(exp)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pole(chain: &str, inf: f32) -> IkPoleTarget {
        IkPoleTarget {
            chain_name: chain.to_string(),
            pole_position: [0.0, 1.0, 0.5],
            influence: inf,
            use_local_space: false,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_ik_pole_export();
        assert_eq!(ik_pole_count(&exp), 0);
    }

    #[test]
    fn add_pole_increments() {
        let mut exp = new_ik_pole_export();
        add_ik_pole(&mut exp, pole("left_leg", 1.0));
        assert_eq!(ik_pole_count(&exp), 1);
    }

    #[test]
    fn find_existing() {
        let mut exp = new_ik_pole_export();
        add_ik_pole(&mut exp, pole("right_arm", 0.8));
        assert!(find_ik_pole(&exp, "right_arm").is_some());
    }

    #[test]
    fn find_missing_none() {
        let exp = new_ik_pole_export();
        assert!(find_ik_pole(&exp, "ghost").is_none());
    }

    #[test]
    fn avg_influence_correct() {
        let mut exp = new_ik_pole_export();
        add_ik_pole(&mut exp, pole("a", 0.4));
        add_ik_pole(&mut exp, pole("b", 0.8));
        assert!((avg_pole_influence(&exp) - 0.6).abs() < 1e-5);
    }

    #[test]
    fn local_space_count() {
        let mut exp = new_ik_pole_export();
        add_ik_pole(
            &mut exp,
            IkPoleTarget {
                use_local_space: true,
                ..pole("x", 1.0)
            },
        );
        assert_eq!(local_space_pole_count(&exp), 1);
    }

    #[test]
    fn validate_valid() {
        let mut exp = new_ik_pole_export();
        add_ik_pole(&mut exp, pole("a", 0.5));
        assert!(validate_ik_poles(&exp));
    }

    #[test]
    fn json_contains_pole_count() {
        let exp = new_ik_pole_export();
        let j = ik_pole_to_json(&exp);
        assert!(j.contains("pole_count"));
    }

    #[test]
    fn influence_in_range() {
        let p = pole("t", 0.75);
        assert!((0.0..=1.0).contains(&p.influence));
    }

    #[test]
    fn empty_avg_influence_zero() {
        let exp = new_ik_pole_export();
        assert!((avg_pole_influence(&exp)).abs() < 1e-6);
    }
}
