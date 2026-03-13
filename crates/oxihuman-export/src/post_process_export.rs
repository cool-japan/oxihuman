// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Post-processing settings export.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostProcessEffect {
    pub name: String,
    pub enabled: bool,
    pub strength: f32,
    pub order: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostProcessExport {
    pub effects: Vec<PostProcessEffect>,
}

#[allow(dead_code)]
pub fn new_post_process_export() -> PostProcessExport {
    PostProcessExport { effects: Vec::new() }
}

#[allow(dead_code)]
pub fn pp_add_effect(export: &mut PostProcessExport, effect: PostProcessEffect) {
    export.effects.push(effect);
}

#[allow(dead_code)]
pub fn pp_remove_effect(export: &mut PostProcessExport, name: &str) {
    export.effects.retain(|e| e.name != name);
}

#[allow(dead_code)]
pub fn pp_effect_count(export: &PostProcessExport) -> usize {
    export.effects.len()
}

#[allow(dead_code)]
pub fn pp_get_effect<'a>(export: &'a PostProcessExport, name: &str) -> Option<&'a PostProcessEffect> {
    export.effects.iter().find(|e| e.name == name)
}

#[allow(dead_code)]
pub fn pp_set_enabled(export: &mut PostProcessExport, name: &str, enabled: bool) {
    if let Some(e) = export.effects.iter_mut().find(|e| e.name == name) {
        e.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn pp_sort_by_order(export: &mut PostProcessExport) {
    export.effects.sort_by_key(|e| e.order);
}

#[allow(dead_code)]
pub fn pp_enabled_count(export: &PostProcessExport) -> usize {
    export.effects.iter().filter(|e| e.enabled).count()
}

#[allow(dead_code)]
pub fn pp_to_json(export: &PostProcessExport) -> String {
    format!(
        "{{\"effect_count\":{},\"enabled_count\":{}}}",
        export.effects.len(),
        pp_enabled_count(export)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_effect(name: &str, order: u32, enabled: bool) -> PostProcessEffect {
        PostProcessEffect { name: name.to_string(), enabled, strength: 1.0, order }
    }

    #[test]
    fn test_new_export() {
        let exp = new_post_process_export();
        assert_eq!(pp_effect_count(&exp), 0);
    }

    #[test]
    fn test_add_effect() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("bloom", 1, true));
        assert_eq!(pp_effect_count(&exp), 1);
    }

    #[test]
    fn test_remove_effect() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("ssao", 0, true));
        pp_add_effect(&mut exp, make_effect("bloom", 1, true));
        pp_remove_effect(&mut exp, "ssao");
        assert_eq!(pp_effect_count(&exp), 1);
    }

    #[test]
    fn test_get_effect() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("dof", 2, false));
        assert!(pp_get_effect(&exp, "dof").is_some());
        assert!(pp_get_effect(&exp, "none").is_none());
    }

    #[test]
    fn test_set_enabled() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("vignette", 0, false));
        pp_set_enabled(&mut exp, "vignette", true);
        assert!(pp_get_effect(&exp, "vignette").expect("should succeed").enabled);
    }

    #[test]
    fn test_sort_by_order() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("c", 3, true));
        pp_add_effect(&mut exp, make_effect("a", 1, true));
        pp_add_effect(&mut exp, make_effect("b", 2, true));
        pp_sort_by_order(&mut exp);
        assert_eq!(exp.effects[0].name, "a");
    }

    #[test]
    fn test_enabled_count() {
        let mut exp = new_post_process_export();
        pp_add_effect(&mut exp, make_effect("e1", 0, true));
        pp_add_effect(&mut exp, make_effect("e2", 1, false));
        pp_add_effect(&mut exp, make_effect("e3", 2, true));
        assert_eq!(pp_enabled_count(&exp), 2);
    }

    #[test]
    fn test_to_json() {
        let exp = new_post_process_export();
        let j = pp_to_json(&exp);
        assert!(j.contains("effect_count"));
    }
}
