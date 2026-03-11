// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Render scope — hierarchical named render sections with timing budgets.

/// A render scope entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderScope {
    pub id: u32,
    pub name: String,
    /// Parent scope id (0 = root).
    pub parent_id: u32,
    /// CPU time budget in microseconds.
    pub budget_us: u64,
    /// Last measured CPU time in microseconds.
    pub last_us: u64,
    pub enabled: bool,
}

impl RenderScope {
    #[allow(dead_code)]
    pub fn new(id: u32, name: &str, parent_id: u32, budget_us: u64) -> Self {
        Self {
            id,
            name: name.to_string(),
            parent_id,
            budget_us,
            last_us: 0,
            enabled: true,
        }
    }

    /// Whether the last measured time exceeded the budget.
    #[allow(dead_code)]
    pub fn is_over_budget(&self) -> bool {
        self.last_us > self.budget_us && self.budget_us > 0
    }
}

/// Scope registry.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct RenderScopeRegistry {
    scopes: Vec<RenderScope>,
}

#[allow(dead_code)]
pub fn new_render_scope_registry() -> RenderScopeRegistry {
    RenderScopeRegistry::default()
}

#[allow(dead_code)]
pub fn rs_add(reg: &mut RenderScopeRegistry, scope: RenderScope) {
    reg.scopes.push(scope);
}

#[allow(dead_code)]
pub fn rs_remove(reg: &mut RenderScopeRegistry, id: u32) {
    reg.scopes.retain(|s| s.id != id);
}

#[allow(dead_code)]
pub fn rs_record_time(reg: &mut RenderScopeRegistry, id: u32, us: u64) {
    for s in reg.scopes.iter_mut() {
        if s.id == id {
            s.last_us = us;
        }
    }
}

#[allow(dead_code)]
pub fn rs_set_enabled(reg: &mut RenderScopeRegistry, id: u32, en: bool) {
    for s in reg.scopes.iter_mut() {
        if s.id == id {
            s.enabled = en;
        }
    }
}

#[allow(dead_code)]
pub fn rs_count(reg: &RenderScopeRegistry) -> usize {
    reg.scopes.len()
}

#[allow(dead_code)]
pub fn rs_enabled_count(reg: &RenderScopeRegistry) -> usize {
    reg.scopes.iter().filter(|s| s.enabled).count()
}

#[allow(dead_code)]
pub fn rs_over_budget_count(reg: &RenderScopeRegistry) -> usize {
    reg.scopes
        .iter()
        .filter(|s| s.enabled && s.is_over_budget())
        .count()
}

#[allow(dead_code)]
pub fn rs_total_time_us(reg: &RenderScopeRegistry) -> u64 {
    reg.scopes
        .iter()
        .filter(|s| s.enabled && s.parent_id == 0)
        .map(|s| s.last_us)
        .sum()
}

#[allow(dead_code)]
pub fn rs_get(reg: &RenderScopeRegistry, id: u32) -> Option<&RenderScope> {
    reg.scopes.iter().find(|s| s.id == id)
}

#[allow(dead_code)]
pub fn rs_clear(reg: &mut RenderScopeRegistry) {
    reg.scopes.clear();
}

#[allow(dead_code)]
pub fn rs_to_json(reg: &RenderScopeRegistry) -> String {
    format!(
        "{{\"count\":{},\"enabled\":{},\"over_budget\":{}}}",
        rs_count(reg),
        rs_enabled_count(reg),
        rs_over_budget_count(reg)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scope(id: u32, parent: u32) -> RenderScope {
        RenderScope::new(id, "test", parent, 1000)
    }

    #[test]
    fn empty_registry() {
        assert_eq!(rs_count(&new_render_scope_registry()), 0);
    }

    #[test]
    fn add_and_count() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        assert_eq!(rs_count(&r), 1);
    }

    #[test]
    fn remove_by_id() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_remove(&mut r, 1);
        assert_eq!(rs_count(&r), 0);
    }

    #[test]
    fn record_time_updates() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_record_time(&mut r, 1, 500);
        assert_eq!(rs_get(&r, 1).unwrap().last_us, 500);
    }

    #[test]
    fn over_budget_detected() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_record_time(&mut r, 1, 2000);
        assert_eq!(rs_over_budget_count(&r), 1);
    }

    #[test]
    fn not_over_budget_within() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_record_time(&mut r, 1, 999);
        assert_eq!(rs_over_budget_count(&r), 0);
    }

    #[test]
    fn disabled_not_counted() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_set_enabled(&mut r, 1, false);
        assert_eq!(rs_enabled_count(&r), 0);
    }

    #[test]
    fn total_time_root_only() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_add(&mut r, make_scope(2, 1)); // child — not counted
        rs_record_time(&mut r, 1, 300);
        rs_record_time(&mut r, 2, 100);
        assert_eq!(rs_total_time_us(&r), 300);
    }

    #[test]
    fn clear_empties() {
        let mut r = new_render_scope_registry();
        rs_add(&mut r, make_scope(1, 0));
        rs_clear(&mut r);
        assert_eq!(rs_count(&r), 0);
    }

    #[test]
    fn json_has_count() {
        let j = rs_to_json(&new_render_scope_registry());
        assert!(j.contains("count"));
    }
}
