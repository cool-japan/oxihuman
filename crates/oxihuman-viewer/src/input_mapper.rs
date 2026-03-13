// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Input key/action mapping.

#[allow(dead_code)]
pub struct InputAction {
    pub name: String,
    pub key_code: u32,
    pub modifiers: u32,
}

#[allow(dead_code)]
pub struct InputMapper {
    pub actions: Vec<InputAction>,
}

#[allow(dead_code)]
pub fn new_input_mapper() -> InputMapper {
    InputMapper { actions: Vec::new() }
}

#[allow(dead_code)]
pub fn im_bind(mapper: &mut InputMapper, name: &str, key_code: u32, modifiers: u32) {
    /* Replace existing binding for this name */
    if let Some(a) = mapper.actions.iter_mut().find(|a| a.name == name) {
        a.key_code = key_code;
        a.modifiers = modifiers;
    } else {
        mapper.actions.push(InputAction { name: name.to_string(), key_code, modifiers });
    }
}

#[allow(dead_code)]
pub fn im_unbind(mapper: &mut InputMapper, name: &str) -> bool {
    if let Some(idx) = mapper.actions.iter().position(|a| a.name == name) {
        mapper.actions.remove(idx);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn im_is_bound(mapper: &InputMapper, key_code: u32, modifiers: u32) -> bool {
    mapper.actions.iter().any(|a| a.key_code == key_code && a.modifiers == modifiers)
}

#[allow(dead_code)]
pub fn im_action_name(mapper: &InputMapper, key_code: u32, modifiers: u32) -> Option<&str> {
    mapper.actions.iter().find(|a| a.key_code == key_code && a.modifiers == modifiers).map(|a| a.name.as_str())
}

#[allow(dead_code)]
pub fn im_count(mapper: &InputMapper) -> usize {
    mapper.actions.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind() {
        let mut m = new_input_mapper();
        im_bind(&mut m, "jump", 32, 0);
        assert_eq!(im_count(&m), 1);
    }

    #[test]
    fn test_unbind() {
        let mut m = new_input_mapper();
        im_bind(&mut m, "jump", 32, 0);
        assert!(im_unbind(&mut m, "jump"));
        assert_eq!(im_count(&m), 0);
    }

    #[test]
    fn test_unbind_missing() {
        let mut m = new_input_mapper();
        assert!(!im_unbind(&mut m, "none"));
    }

    #[test]
    fn test_is_bound() {
        let mut m = new_input_mapper();
        im_bind(&mut m, "fire", 65, 0);
        assert!(im_is_bound(&m, 65, 0));
    }

    #[test]
    fn test_is_not_bound() {
        let m = new_input_mapper();
        assert!(!im_is_bound(&m, 65, 0));
    }

    #[test]
    fn test_action_name() {
        let mut m = new_input_mapper();
        im_bind(&mut m, "reload", 82, 0);
        assert_eq!(im_action_name(&m, 82, 0), Some("reload"));
    }

    #[test]
    fn test_action_name_missing() {
        let m = new_input_mapper();
        assert_eq!(im_action_name(&m, 99, 0), None);
    }

    #[test]
    fn test_count() {
        let mut m = new_input_mapper();
        im_bind(&mut m, "a", 1, 0);
        im_bind(&mut m, "b", 2, 0);
        assert_eq!(im_count(&m), 2);
    }
}
