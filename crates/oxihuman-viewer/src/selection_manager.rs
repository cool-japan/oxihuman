// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Selection manager for viewport picking.

#[allow(dead_code)]
pub struct SelectionManager {
    pub selected_ids: Vec<u32>,
    pub max_selection: usize,
}

#[allow(dead_code)]
pub fn new_selection_manager(max_selection: usize) -> SelectionManager {
    SelectionManager { selected_ids: Vec::new(), max_selection }
}

#[allow(dead_code)]
pub fn selm_select(m: &mut SelectionManager, id: u32) {
    if m.selected_ids.contains(&id) {
        return;
    }
    if m.selected_ids.len() >= m.max_selection {
        m.selected_ids.remove(0);
    }
    m.selected_ids.push(id);
}

#[allow(dead_code)]
pub fn selm_deselect(m: &mut SelectionManager, id: u32) {
    m.selected_ids.retain(|&x| x != id);
}

#[allow(dead_code)]
pub fn selm_clear(m: &mut SelectionManager) {
    m.selected_ids.clear();
}

#[allow(dead_code)]
pub fn selm_is_selected(m: &SelectionManager, id: u32) -> bool {
    m.selected_ids.contains(&id)
}

#[allow(dead_code)]
pub fn selm_count(m: &SelectionManager) -> usize {
    m.selected_ids.len()
}

#[allow(dead_code)]
pub fn selm_toggle(m: &mut SelectionManager, id: u32) {
    if selm_is_selected(m, id) {
        selm_deselect(m, id);
    } else {
        selm_select(m, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select() {
        let mut m = new_selection_manager(5);
        selm_select(&mut m, 1);
        assert!(selm_is_selected(&m, 1));
    }

    #[test]
    fn test_deselect() {
        let mut m = new_selection_manager(5);
        selm_select(&mut m, 1);
        selm_deselect(&mut m, 1);
        assert!(!selm_is_selected(&m, 1));
    }

    #[test]
    fn test_clear() {
        let mut m = new_selection_manager(5);
        selm_select(&mut m, 1);
        selm_select(&mut m, 2);
        selm_clear(&mut m);
        assert_eq!(selm_count(&m), 0);
    }

    #[test]
    fn test_is_selected_false() {
        let m = new_selection_manager(5);
        assert!(!selm_is_selected(&m, 99));
    }

    #[test]
    fn test_count() {
        let mut m = new_selection_manager(5);
        selm_select(&mut m, 1);
        selm_select(&mut m, 2);
        assert_eq!(selm_count(&m), 2);
    }

    #[test]
    fn test_toggle_on() {
        let mut m = new_selection_manager(5);
        selm_toggle(&mut m, 1);
        assert!(selm_is_selected(&m, 1));
    }

    #[test]
    fn test_toggle_off() {
        let mut m = new_selection_manager(5);
        selm_select(&mut m, 1);
        selm_toggle(&mut m, 1);
        assert!(!selm_is_selected(&m, 1));
    }

    #[test]
    fn test_max_enforced() {
        let mut m = new_selection_manager(2);
        selm_select(&mut m, 1);
        selm_select(&mut m, 2);
        selm_select(&mut m, 3);
        assert_eq!(selm_count(&m), 2);
        assert!(!selm_is_selected(&m, 1));
    }
}
