// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct MhMorphParam {
    pub name: String,
    pub value: f32,
}

pub struct MhExport {
    pub version: String,
    pub params: Vec<MhMorphParam>,
}

pub fn new_mh_export() -> MhExport {
    MhExport {
        version: "1.2.0".to_string(),
        params: Vec::new(),
    }
}

pub fn mh_push_param(e: &mut MhExport, name: &str, value: f32) {
    e.params.push(MhMorphParam {
        name: name.to_string(),
        value,
    });
}

pub fn mh_to_mhm_string(e: &MhExport) -> String {
    let mut lines = vec![format!("version {}", e.version)];
    for p in &e.params {
        lines.push(format!("{} {}", p.name, p.value));
    }
    lines.join("\n")
}

pub fn mh_param_count(e: &MhExport) -> usize {
    e.params.len()
}

pub fn mh_find_param(e: &MhExport, name: &str) -> Option<f32> {
    e.params.iter().find(|p| p.name == name).map(|p| p.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mh_export_empty() {
        /* starts empty */
        let e = new_mh_export();
        assert_eq!(mh_param_count(&e), 0);
    }

    #[test]
    fn test_mh_push_param() {
        /* push param */
        let mut e = new_mh_export();
        mh_push_param(&mut e, "macrodetails/Age", 0.5);
        assert_eq!(mh_param_count(&e), 1);
    }

    #[test]
    fn test_mh_find_param_found() {
        /* find existing param */
        let mut e = new_mh_export();
        mh_push_param(&mut e, "Age", 0.7);
        let v = mh_find_param(&e, "Age");
        assert!((v.expect("should succeed") - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_mh_find_param_none() {
        /* missing param */
        let e = new_mh_export();
        assert!(mh_find_param(&e, "nonexistent").is_none());
    }

    #[test]
    fn test_mh_to_mhm_contains_version() {
        /* mhm string contains version */
        let e = new_mh_export();
        let s = mh_to_mhm_string(&e);
        assert!(s.contains("version"));
    }
}
