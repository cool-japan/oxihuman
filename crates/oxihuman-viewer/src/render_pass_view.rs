// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Render pass selector/viewer stub.

/// A named render pass entry.
#[derive(Debug, Clone)]
pub struct RenderPassEntry {
    pub name: String,
    pub enabled: bool,
    pub order: usize,
}

/// Render pass view state.
#[derive(Debug, Clone, Default)]
pub struct RenderPassView {
    pub passes: Vec<RenderPassEntry>,
    pub selected: Option<usize>,
}

/// Create a new render pass view.
pub fn new_render_pass_view() -> RenderPassView {
    RenderPassView::default()
}

/// Add a pass.
pub fn rpv_add_pass(view: &mut RenderPassView, name: &str) {
    let order = view.passes.len();
    view.passes.push(RenderPassEntry {
        name: name.to_string(),
        enabled: true,
        order,
    });
}

/// Select a pass by index.
pub fn rpv_select(view: &mut RenderPassView, index: usize) {
    if index < view.passes.len() {
        view.selected = Some(index);
    }
}

/// Toggle a pass enabled state.
pub fn rpv_toggle(view: &mut RenderPassView, index: usize) {
    if index < view.passes.len() {
        view.passes[index].enabled = !view.passes[index].enabled;
    }
}

/// Return the pass count.
pub fn rpv_pass_count(view: &RenderPassView) -> usize {
    view.passes.len()
}

/// Return enabled pass count.
pub fn rpv_enabled_count(view: &RenderPassView) -> usize {
    view.passes.iter().filter(|p| p.enabled).count()
}

/// Return a JSON-like string.
pub fn rpv_to_json(view: &RenderPassView) -> String {
    format!(
        r#"{{"passes":{},"selected":{:?}}}"#,
        view.passes.len(),
        view.selected
    )
}

/// Return the selected pass name if any.
pub fn rpv_selected_name(view: &RenderPassView) -> Option<&str> {
    view.selected
        .and_then(|i| view.passes.get(i))
        .map(|p| p.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_render_pass_view_empty() {
        let v = new_render_pass_view();
        assert_eq!(rpv_pass_count(&v), 0 /* new view should be empty */,);
    }

    #[test]
    fn test_add_pass_increases_count() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "shadow");
        assert_eq!(rpv_pass_count(&v), 1 /* count should increase */,);
    }

    #[test]
    fn test_select_pass() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "depth");
        rpv_select(&mut v, 0);
        assert_eq!(v.selected, Some(0) /* selected should be 0 */,);
    }

    #[test]
    fn test_select_out_of_bounds_ignored() {
        let mut v = new_render_pass_view();
        rpv_select(&mut v, 99);
        assert!(v.selected.is_none() /* out-of-bounds select ignored */,);
    }

    #[test]
    fn test_toggle_disables_pass() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "shadow");
        rpv_toggle(&mut v, 0);
        assert!(!v.passes[0].enabled, /* pass should be disabled after toggle */);
    }

    #[test]
    fn test_enabled_count_all_enabled() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "a");
        rpv_add_pass(&mut v, "b");
        assert_eq!(
            rpv_enabled_count(&v),
            2, /* both passes enabled initially */
        );
    }

    #[test]
    fn test_enabled_count_one_disabled() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "a");
        rpv_add_pass(&mut v, "b");
        rpv_toggle(&mut v, 0);
        assert_eq!(rpv_enabled_count(&v), 1 /* one pass disabled */,);
    }

    #[test]
    fn test_to_json_contains_passes() {
        let v = new_render_pass_view();
        let j = rpv_to_json(&v);
        assert!(j.contains("passes") /* JSON must contain passes */,);
    }

    #[test]
    fn test_selected_name_none_initially() {
        let v = new_render_pass_view();
        assert!(rpv_selected_name(&v).is_none(), /* no pass selected initially */);
    }

    #[test]
    fn test_selected_name_returns_correct() {
        let mut v = new_render_pass_view();
        rpv_add_pass(&mut v, "lighting");
        rpv_select(&mut v, 0);
        assert_eq!(
            rpv_selected_name(&v),
            Some("lighting"), /* selected name must match */
        );
    }
}
