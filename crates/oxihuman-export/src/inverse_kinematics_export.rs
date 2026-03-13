#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export IK chain configuration.

#[allow(dead_code)]
pub struct IkChainExport {
    pub name: String,
    pub tip_bone: String,
    pub root_bone: String,
    pub target: [f32; 3],
    pub iterations: u32,
    pub weight: f32,
}

#[allow(dead_code)]
pub struct IkExport {
    pub chains: Vec<IkChainExport>,
}

#[allow(dead_code)]
pub fn new_ik_export() -> IkExport {
    IkExport { chains: vec![] }
}

#[allow(dead_code)]
pub fn add_chain(
    exp: &mut IkExport,
    name: &str,
    tip: &str,
    root: &str,
    target: [f32; 3],
    iters: u32,
) {
    exp.chains.push(IkChainExport {
        name: name.to_string(),
        tip_bone: tip.to_string(),
        root_bone: root.to_string(),
        target,
        iterations: iters,
        weight: 1.0,
    });
}

#[allow(dead_code)]
pub fn export_ik_to_json(exp: &IkExport) -> String {
    let chains_str: Vec<String> = exp.chains.iter().map(|c| {
        format!(
            r#"{{"name":"{}","tip":"{}","root":"{}","target":[{},{},{}],"iterations":{},"weight":{}}}"#,
            c.name, c.tip_bone, c.root_bone,
            c.target[0], c.target[1], c.target[2],
            c.iterations, c.weight
        )
    }).collect();
    format!(r#"{{"chains":[{}]}}"#, chains_str.join(","))
}

#[allow(dead_code)]
pub fn chain_count(exp: &IkExport) -> usize {
    exp.chains.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_ik_export();
        assert_eq!(chain_count(&e), 0);
    }

    #[test]
    fn add_chain_increments_count() {
        let mut e = new_ik_export();
        add_chain(&mut e, "arm_ik", "hand", "shoulder", [0.0; 3], 10);
        assert_eq!(chain_count(&e), 1);
    }

    #[test]
    fn add_multiple_chains() {
        let mut e = new_ik_export();
        add_chain(&mut e, "arm", "hand", "shoulder", [0.0; 3], 5);
        add_chain(&mut e, "leg", "foot", "hip", [0.0, -1.0, 0.0], 8);
        assert_eq!(chain_count(&e), 2);
    }

    #[test]
    fn chain_name_stored() {
        let mut e = new_ik_export();
        add_chain(&mut e, "spine_ik", "head", "pelvis", [0.0; 3], 10);
        assert_eq!(e.chains[0].name, "spine_ik");
    }

    #[test]
    fn chain_iterations_stored() {
        let mut e = new_ik_export();
        add_chain(&mut e, "x", "a", "b", [0.0; 3], 20);
        assert_eq!(e.chains[0].iterations, 20);
    }

    #[test]
    fn export_ik_json_contains_name() {
        let mut e = new_ik_export();
        add_chain(&mut e, "mychain", "tip", "root", [0.0; 3], 5);
        let json = export_ik_to_json(&e);
        assert!(json.contains("mychain"));
    }

    #[test]
    fn export_ik_json_empty() {
        let e = new_ik_export();
        let json = export_ik_to_json(&e);
        assert!(json.contains("chains"));
    }

    #[test]
    fn chain_target_stored() {
        let mut e = new_ik_export();
        add_chain(&mut e, "x", "t", "r", [1.0, 2.0, 3.0], 5);
        assert!((e.chains[0].target[1] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn weight_default_one() {
        let mut e = new_ik_export();
        add_chain(&mut e, "x", "t", "r", [0.0; 3], 5);
        assert!((e.chains[0].weight - 1.0).abs() < 1e-5);
    }
}
