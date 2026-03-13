// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Header bar UI state.

#![allow(dead_code)]

/// Header bar UI state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HeaderBarState {
    pub active_menu: Option<u32>,
    pub breadcrumb: Vec<String>,
    pub show_search: bool,
    pub search_query: String,
}

/// Returns a default `HeaderBarState`.
#[allow(dead_code)]
pub fn default_header_bar() -> HeaderBarState {
    HeaderBarState {
        active_menu: None,
        breadcrumb: Vec::new(),
        show_search: false,
        search_query: String::new(),
    }
}

/// Opens the menu with the given id.
#[allow(dead_code)]
pub fn open_menu(bar: &mut HeaderBarState, id: u32) {
    bar.active_menu = Some(id);
}

/// Closes any open menu.
#[allow(dead_code)]
pub fn close_menu(bar: &mut HeaderBarState) {
    bar.active_menu = None;
}

/// Appends a label to the breadcrumb trail.
#[allow(dead_code)]
pub fn push_breadcrumb(bar: &mut HeaderBarState, label: &str) {
    bar.breadcrumb.push(label.to_string());
}

/// Removes and returns the last breadcrumb label, if any.
#[allow(dead_code)]
pub fn pop_breadcrumb(bar: &mut HeaderBarState) -> Option<String> {
    bar.breadcrumb.pop()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_header_bar() {
        let b = default_header_bar();
        assert!(b.active_menu.is_none());
        assert!(b.breadcrumb.is_empty());
        assert!(!b.show_search);
        assert!(b.search_query.is_empty());
    }

    #[test]
    fn test_open_menu() {
        let mut b = default_header_bar();
        open_menu(&mut b, 42);
        assert_eq!(b.active_menu, Some(42));
    }

    #[test]
    fn test_close_menu() {
        let mut b = default_header_bar();
        open_menu(&mut b, 1);
        close_menu(&mut b);
        assert!(b.active_menu.is_none());
    }

    #[test]
    fn test_push_breadcrumb() {
        let mut b = default_header_bar();
        push_breadcrumb(&mut b, "Root");
        push_breadcrumb(&mut b, "Scene");
        assert_eq!(b.breadcrumb.len(), 2);
        assert_eq!(b.breadcrumb[0], "Root");
    }

    #[test]
    fn test_pop_breadcrumb_returns_last() {
        let mut b = default_header_bar();
        push_breadcrumb(&mut b, "Root");
        push_breadcrumb(&mut b, "Scene");
        let popped = pop_breadcrumb(&mut b);
        assert_eq!(popped, Some("Scene".to_string()));
        assert_eq!(b.breadcrumb.len(), 1);
    }

    #[test]
    fn test_pop_breadcrumb_empty() {
        let mut b = default_header_bar();
        let popped = pop_breadcrumb(&mut b);
        assert!(popped.is_none());
    }

    #[test]
    fn test_open_menu_replaces() {
        let mut b = default_header_bar();
        open_menu(&mut b, 1);
        open_menu(&mut b, 2);
        assert_eq!(b.active_menu, Some(2));
    }

    #[test]
    fn test_push_pop_roundtrip() {
        let mut b = default_header_bar();
        push_breadcrumb(&mut b, "A");
        let _ = pop_breadcrumb(&mut b);
        assert!(b.breadcrumb.is_empty());
    }

    #[test]
    fn test_search_query_default_empty() {
        let b = default_header_bar();
        assert_eq!(b.search_query, "");
    }

    #[test]
    fn test_breadcrumb_multiple_pops() {
        let mut b = default_header_bar();
        push_breadcrumb(&mut b, "A");
        push_breadcrumb(&mut b, "B");
        push_breadcrumb(&mut b, "C");
        let _ = pop_breadcrumb(&mut b);
        let _ = pop_breadcrumb(&mut b);
        assert_eq!(b.breadcrumb.len(), 1);
        assert_eq!(b.breadcrumb[0], "A");
    }
}
