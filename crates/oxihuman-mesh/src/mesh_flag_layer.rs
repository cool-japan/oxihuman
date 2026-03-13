// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Boolean flag layer per mesh element (vertex, edge, or face).

/// A named boolean flag layer.
pub struct FlagLayer {
    pub name: String,
    pub flags: Vec<bool>,
}

/// A set of named boolean flag layers.
pub struct FlagLayerSet {
    pub layers: Vec<FlagLayer>,
}

/// Create a new empty flag layer set.
pub fn new_flag_layer_set() -> FlagLayerSet {
    FlagLayerSet { layers: Vec::new() }
}

/// Add a named flag layer with given element count, initialised to false.
pub fn add_flag_layer(set: &mut FlagLayerSet, name: &str, element_count: usize) {
    set.layers.push(FlagLayer {
        name: name.to_string(),
        flags: vec![false; element_count],
    });
}

/// Set a flag at a given element index in the named layer.
pub fn set_flag(set: &mut FlagLayerSet, name: &str, index: usize, value: bool) -> bool {
    if let Some(layer) = set.layers.iter_mut().find(|l| l.name == name) {
        if index < layer.flags.len() {
            layer.flags[index] = value;
            return true;
        }
    }
    false
}

/// Get the flag at an element index in the named layer.
pub fn get_flag(set: &FlagLayerSet, name: &str, index: usize) -> Option<bool> {
    let layer = set.layers.iter().find(|l| l.name == name)?;
    layer.flags.get(index).copied()
}

/// Count of set (true) flags in the named layer.
pub fn flag_set_count(set: &FlagLayerSet, name: &str) -> usize {
    set.layers
        .iter()
        .find(|l| l.name == name)
        .map(|l| l.flags.iter().filter(|&&f| f).count())
        .unwrap_or(0)
}

/// Clear all flags (set to false) in the named layer.
pub fn clear_flags(set: &mut FlagLayerSet, name: &str) -> bool {
    if let Some(layer) = set.layers.iter_mut().find(|l| l.name == name) {
        for f in layer.flags.iter_mut() {
            *f = false;
        }
        true
    } else {
        false
    }
}

/// Number of flag layers.
pub fn flag_layer_count(set: &FlagLayerSet) -> usize {
    set.layers.len()
}

/// Invert all flags in the named layer.
pub fn invert_flags(set: &mut FlagLayerSet, name: &str) -> bool {
    if let Some(layer) = set.layers.iter_mut().find(|l| l.name == name) {
        for f in layer.flags.iter_mut() {
            *f = !*f;
        }
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_empty() {
        let s = new_flag_layer_set();
        assert_eq!(flag_layer_count(&s), 0 /* empty */);
    }

    #[test]
    fn add_layer_increments() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "selected", 5);
        assert_eq!(flag_layer_count(&s), 1 /* one layer */);
    }

    #[test]
    fn flags_default_false() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "vis", 3);
        assert_eq!(get_flag(&s, "vis", 0), Some(false) /* default false */);
    }

    #[test]
    fn set_and_get_flag() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "sel", 4);
        set_flag(&mut s, "sel", 2, true);
        assert_eq!(get_flag(&s, "sel", 2), Some(true) /* set to true */);
    }

    #[test]
    fn set_out_of_bounds_returns_false() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "x", 2);
        assert!(!set_flag(&mut s, "x", 99, true) /* out of bounds */);
    }

    #[test]
    fn flag_set_count_correct() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "f", 5);
        set_flag(&mut s, "f", 1, true);
        set_flag(&mut s, "f", 3, true);
        assert_eq!(flag_set_count(&s, "f"), 2 /* two set */);
    }

    #[test]
    fn clear_flags_resets_all() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "c", 3);
        set_flag(&mut s, "c", 0, true);
        clear_flags(&mut s, "c");
        assert_eq!(flag_set_count(&s, "c"), 0 /* all cleared */);
    }

    #[test]
    fn invert_flags_toggles() {
        let mut s = new_flag_layer_set();
        add_flag_layer(&mut s, "inv", 4);
        set_flag(&mut s, "inv", 0, true);
        invert_flags(&mut s, "inv");
        assert_eq!(
            get_flag(&s, "inv", 0),
            Some(false) /* inverted from true */
        );
        assert_eq!(
            get_flag(&s, "inv", 1),
            Some(true) /* inverted from false */
        );
    }

    #[test]
    fn get_flag_missing_layer_none() {
        let s = new_flag_layer_set();
        assert!(get_flag(&s, "none", 0).is_none() /* not found */);
    }
}
