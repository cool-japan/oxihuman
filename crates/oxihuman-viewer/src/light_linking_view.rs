// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Light linking and exclusion panel view.

/// A light-to-object link entry.
#[derive(Debug, Clone)]
pub struct LightLink {
    pub light_id: u32,
    pub object_id: u32,
    pub excluded: bool,
}

/// Light linking view state.
#[derive(Debug, Clone)]
pub struct LightLinkingView {
    pub links: Vec<LightLink>,
    pub visible: bool,
}

impl Default for LightLinkingView {
    fn default() -> Self {
        Self {
            links: Vec::new(),
            visible: true,
        }
    }
}

/// Create a new LightLinkingView.
pub fn new_light_linking_view() -> LightLinkingView {
    LightLinkingView::default()
}

/// Add a link between a light and an object.
pub fn light_link_add(view: &mut LightLinkingView, light_id: u32, object_id: u32, excluded: bool) {
    view.links.push(LightLink {
        light_id,
        object_id,
        excluded,
    });
}

/// Remove all links for a given light.
pub fn light_link_remove_light(view: &mut LightLinkingView, light_id: u32) {
    view.links.retain(|l| l.light_id != light_id);
}

/// Count links for a given light.
pub fn light_link_count(view: &LightLinkingView, light_id: u32) -> usize {
    view.links.iter().filter(|l| l.light_id == light_id).count()
}

/// Count excluded links.
pub fn light_link_excluded_count(view: &LightLinkingView) -> usize {
    view.links.iter().filter(|l| l.excluded).count()
}

/// Serialize to JSON.
pub fn light_linking_to_json(view: &LightLinkingView) -> String {
    format!(
        r#"{{"visible":{},"link_count":{}}}"#,
        view.visible,
        view.links.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_light_linking_view();
        assert!(v.links.is_empty() /* no links */);
    }

    #[test]
    fn test_add_link() {
        let mut v = new_light_linking_view();
        light_link_add(&mut v, 1, 2, false);
        assert_eq!(v.links.len(), 1 /* one link */);
    }

    #[test]
    fn test_add_multiple() {
        let mut v = new_light_linking_view();
        light_link_add(&mut v, 1, 2, false);
        light_link_add(&mut v, 1, 3, true);
        assert_eq!(light_link_count(&v, 1), 2 /* two for light 1 */);
    }

    #[test]
    fn test_remove_light() {
        let mut v = new_light_linking_view();
        light_link_add(&mut v, 1, 2, false);
        light_link_add(&mut v, 2, 3, false);
        light_link_remove_light(&mut v, 1);
        assert_eq!(v.links.len(), 1 /* one remaining */);
    }

    #[test]
    fn test_count_zero() {
        let v = new_light_linking_view();
        assert_eq!(light_link_count(&v, 99), 0 /* no links */);
    }

    #[test]
    fn test_excluded_count() {
        let mut v = new_light_linking_view();
        light_link_add(&mut v, 1, 2, true);
        light_link_add(&mut v, 1, 3, false);
        assert_eq!(light_link_excluded_count(&v), 1 /* one excluded */);
    }

    #[test]
    fn test_json_key() {
        let v = new_light_linking_view();
        let j = light_linking_to_json(&v);
        assert!(j.contains("link_count") /* key */);
    }

    #[test]
    fn test_default_visible() {
        let v = LightLinkingView::default();
        assert!(v.visible /* visible */);
    }

    #[test]
    fn test_clone() {
        let mut v = new_light_linking_view();
        light_link_add(&mut v, 1, 2, false);
        let c = v.clone();
        assert_eq!(c.links.len(), v.links.len() /* equal */);
    }
}
