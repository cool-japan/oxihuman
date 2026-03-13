// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct DazFigureExport {
    pub figure_name: String,
    pub morph_values: Vec<(String, f32)>,
    pub bone_count: usize,
}

pub fn new_daz_figure(name: &str, bone_count: usize) -> DazFigureExport {
    DazFigureExport {
        figure_name: name.to_string(),
        morph_values: Vec::new(),
        bone_count,
    }
}

pub fn daz_push_morph(f: &mut DazFigureExport, name: &str, value: f32) {
    f.morph_values.push((name.to_string(), value));
}

pub fn daz_to_json(f: &DazFigureExport) -> String {
    let morphs: Vec<_> = f
        .morph_values
        .iter()
        .map(|(n, v)| format!(r#"{{"name":"{}","value":{}}}"#, n, v))
        .collect();
    format!(
        r#"{{"figure":"{}","bones":{},"morphs":[{}]}}"#,
        f.figure_name,
        f.bone_count,
        morphs.join(",")
    )
}

pub fn daz_morph_count(f: &DazFigureExport) -> usize {
    f.morph_values.len()
}

pub fn daz_is_genesis8(f: &DazFigureExport) -> bool {
    f.bone_count >= 200
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_daz_figure() {
        /* construction */
        let f = new_daz_figure("Genesis8", 230);
        assert_eq!(f.figure_name, "Genesis8");
    }

    #[test]
    fn test_daz_push_morph() {
        /* push morph */
        let mut f = new_daz_figure("G8", 230);
        daz_push_morph(&mut f, "Head Size", 0.5);
        assert_eq!(daz_morph_count(&f), 1);
    }

    #[test]
    fn test_daz_to_json_contains_name() {
        /* json contains figure name */
        let f = new_daz_figure("G8F", 230);
        let j = daz_to_json(&f);
        assert!(j.contains("G8F"));
    }

    #[test]
    fn test_daz_is_genesis8_true() {
        /* 200+ bones = genesis8 */
        let f = new_daz_figure("G8", 200);
        assert!(daz_is_genesis8(&f));
    }

    #[test]
    fn test_daz_is_genesis8_false() {
        /* less bones = not genesis8 */
        let f = new_daz_figure("Simple", 50);
        assert!(!daz_is_genesis8(&f));
    }
}
