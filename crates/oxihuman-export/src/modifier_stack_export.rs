// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ModifierType {
    Subdivision,
    Array,
    Mirror,
    Solidify,
    Bevel,
    Skin,
    Other(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModifierEntry {
    pub name: String,
    pub mod_type: ModifierType,
    pub enabled: bool,
    pub order: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModifierStackExport {
    pub object_name: String,
    pub modifiers: Vec<ModifierEntry>,
}

#[allow(dead_code)]
pub fn new_modifier_stack_export(object_name: &str) -> ModifierStackExport {
    ModifierStackExport {
        object_name: object_name.to_string(),
        modifiers: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn ms_add_modifier(exp: &mut ModifierStackExport, entry: ModifierEntry) {
    exp.modifiers.push(entry);
}

#[allow(dead_code)]
pub fn ms_modifier_count(exp: &ModifierStackExport) -> usize {
    exp.modifiers.len()
}

#[allow(dead_code)]
pub fn ms_get_modifier(exp: &ModifierStackExport, idx: usize) -> Option<&ModifierEntry> {
    exp.modifiers.get(idx)
}

#[allow(dead_code)]
pub fn ms_set_enabled(exp: &mut ModifierStackExport, idx: usize, enabled: bool) {
    if let Some(m) = exp.modifiers.get_mut(idx) {
        m.enabled = enabled;
    }
}

#[allow(dead_code)]
pub fn ms_to_json(exp: &ModifierStackExport) -> String {
    format!(
        r#"{{"object":"{}","modifier_count":{}}}"#,
        exp.object_name,
        exp.modifiers.len()
    )
}

#[allow(dead_code)]
pub fn ms_enabled_count(exp: &ModifierStackExport) -> usize {
    exp.modifiers.iter().filter(|m| m.enabled).count()
}

#[allow(dead_code)]
pub fn ms_type_name(mod_type: &ModifierType) -> &str {
    match mod_type {
        ModifierType::Subdivision => "Subdivision",
        ModifierType::Array => "Array",
        ModifierType::Mirror => "Mirror",
        ModifierType::Solidify => "Solidify",
        ModifierType::Bevel => "Bevel",
        ModifierType::Skin => "Skin",
        ModifierType::Other(s) => s.as_str(),
    }
}

#[allow(dead_code)]
pub fn ms_remove_modifier(exp: &mut ModifierStackExport, idx: usize) {
    if idx < exp.modifiers.len() {
        exp.modifiers.remove(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(name: &str, t: ModifierType) -> ModifierEntry {
        ModifierEntry { name: name.to_string(), mod_type: t, enabled: true, order: 0 }
    }

    #[test]
    fn test_new_stack_empty() {
        let e = new_modifier_stack_export("Cube");
        assert_eq!(ms_modifier_count(&e), 0);
    }

    #[test]
    fn test_add_modifier() {
        let mut e = new_modifier_stack_export("Cube");
        ms_add_modifier(&mut e, make_entry("Subsurf", ModifierType::Subdivision));
        assert_eq!(ms_modifier_count(&e), 1);
    }

    #[test]
    fn test_get_modifier() {
        let mut e = new_modifier_stack_export("Cube");
        ms_add_modifier(&mut e, make_entry("Mirror", ModifierType::Mirror));
        let m = ms_get_modifier(&e, 0).unwrap();
        assert_eq!(m.name, "Mirror");
    }

    #[test]
    fn test_set_enabled() {
        let mut e = new_modifier_stack_export("Cube");
        ms_add_modifier(&mut e, make_entry("Array", ModifierType::Array));
        ms_set_enabled(&mut e, 0, false);
        assert_eq!(ms_enabled_count(&e), 0);
    }

    #[test]
    fn test_enabled_count() {
        let mut e = new_modifier_stack_export("Cube");
        ms_add_modifier(&mut e, make_entry("A", ModifierType::Array));
        ms_add_modifier(&mut e, make_entry("B", ModifierType::Bevel));
        ms_set_enabled(&mut e, 1, false);
        assert_eq!(ms_enabled_count(&e), 1);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(ms_type_name(&ModifierType::Subdivision), "Subdivision");
        assert_eq!(ms_type_name(&ModifierType::Other("Custom".to_string())), "Custom");
    }

    #[test]
    fn test_remove_modifier() {
        let mut e = new_modifier_stack_export("Cube");
        ms_add_modifier(&mut e, make_entry("A", ModifierType::Array));
        ms_add_modifier(&mut e, make_entry("B", ModifierType::Bevel));
        ms_remove_modifier(&mut e, 0);
        assert_eq!(ms_modifier_count(&e), 1);
    }

    #[test]
    fn test_to_json() {
        let e = new_modifier_stack_export("Sphere");
        let j = ms_to_json(&e);
        assert!(j.contains("Sphere"));
        assert!(j.contains("modifier_count"));
    }
}
