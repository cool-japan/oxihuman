// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ordered list of named commands with priority and enabled flag.

/// Priority level for a command.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CmdPriority {
    Low,
    Normal,
    High,
}

/// A single entry in the command list.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CmdEntry {
    pub name: String,
    pub priority: CmdPriority,
    pub enabled: bool,
    pub tag: Option<String>,
}

/// Ordered, prioritised list of named commands.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CommandList {
    entries: Vec<CmdEntry>,
}

#[allow(dead_code)]
impl CommandList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, name: &str, priority: CmdPriority) {
        self.entries.push(CmdEntry {
            name: name.to_string(),
            priority,
            enabled: true,
            tag: None,
        });
    }

    pub fn push_tagged(&mut self, name: &str, priority: CmdPriority, tag: &str) {
        self.entries.push(CmdEntry {
            name: name.to_string(),
            priority,
            enabled: true,
            tag: Some(tag.to_string()),
        });
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&CmdEntry> {
        self.entries.get(index)
    }

    pub fn enable(&mut self, name: &str) {
        for e in &mut self.entries {
            if e.name == name {
                e.enabled = true;
            }
        }
    }

    pub fn disable(&mut self, name: &str) {
        for e in &mut self.entries {
            if e.name == name {
                e.enabled = false;
            }
        }
    }

    pub fn enabled_names(&self) -> Vec<&str> {
        self.entries
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.name.as_str())
            .collect()
    }

    pub fn sorted_by_priority(&self) -> Vec<&CmdEntry> {
        let mut v: Vec<&CmdEntry> = self.entries.iter().collect();
        v.sort_by(|a, b| b.priority.cmp(&a.priority));
        v
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&CmdEntry> {
        self.entries
            .iter()
            .filter(|e| e.tag.as_deref() == Some(tag))
            .collect()
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < before
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let cl = CommandList::new();
        assert!(cl.is_empty());
        assert_eq!(cl.len(), 0);
    }

    #[test]
    fn push_and_get() {
        let mut cl = CommandList::new();
        cl.push("draw", CmdPriority::Normal);
        assert_eq!(cl.len(), 1);
        assert_eq!(cl.get(0).expect("should succeed").name, "draw");
        assert!(cl.get(0).expect("should succeed").enabled);
    }

    #[test]
    fn contains_works() {
        let mut cl = CommandList::new();
        cl.push("update", CmdPriority::High);
        assert!(cl.contains("update"));
        assert!(!cl.contains("missing"));
    }

    #[test]
    fn enable_disable() {
        let mut cl = CommandList::new();
        cl.push("tick", CmdPriority::Normal);
        cl.disable("tick");
        assert!(!cl.get(0).expect("should succeed").enabled);
        cl.enable("tick");
        assert!(cl.get(0).expect("should succeed").enabled);
    }

    #[test]
    fn enabled_names_filters() {
        let mut cl = CommandList::new();
        cl.push("a", CmdPriority::Normal);
        cl.push("b", CmdPriority::Low);
        cl.disable("b");
        let names = cl.enabled_names();
        assert_eq!(names, vec!["a"]);
    }

    #[test]
    fn sorted_by_priority_descending() {
        let mut cl = CommandList::new();
        cl.push("lo", CmdPriority::Low);
        cl.push("hi", CmdPriority::High);
        cl.push("nm", CmdPriority::Normal);
        let sorted = cl.sorted_by_priority();
        assert_eq!(sorted[0].priority, CmdPriority::High);
        assert_eq!(sorted[2].priority, CmdPriority::Low);
    }

    #[test]
    fn push_tagged_and_by_tag() {
        let mut cl = CommandList::new();
        cl.push_tagged("render_a", CmdPriority::Normal, "render");
        cl.push_tagged("render_b", CmdPriority::Normal, "render");
        cl.push("logic", CmdPriority::Normal);
        let tagged = cl.by_tag("render");
        assert_eq!(tagged.len(), 2);
    }

    #[test]
    fn remove_entry() {
        let mut cl = CommandList::new();
        cl.push("x", CmdPriority::Normal);
        assert!(cl.remove("x"));
        assert!(cl.is_empty());
        assert!(!cl.remove("x"));
    }

    #[test]
    fn clear_empties_list() {
        let mut cl = CommandList::new();
        cl.push("a", CmdPriority::Normal);
        cl.push("b", CmdPriority::High);
        cl.clear();
        assert!(cl.is_empty());
    }

    #[test]
    fn multiple_same_name_priority() {
        let mut cl = CommandList::new();
        cl.push("dup", CmdPriority::Normal);
        cl.push("dup", CmdPriority::High);
        assert_eq!(cl.len(), 2);
        cl.disable("dup");
        assert!(cl.enabled_names().is_empty());
    }
}
